//! The drawing (ADR 0010): finished moves go into a pixel buffer drawn by
//! vello_cpu and shown as 256 × 256 tiles; the move in progress and the
//! turtle are painted as paths each frame.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use gpui_kit::*;
use pendown_logo::Point;
use vello_cpu::{
    RasterizerSettings, RenderContext, RenderSettings, Resources, TargetInit,
    kurbo::{BezPath, Cap, Stroke},
};

use spike_04_end_to_end_animator::animator::Segment;

/// The turtle area is 1000 × 1000 steps, from -500 to 500 (ADR 0003).
pub const TURTLE_AREA: f64 = 1000.0;
const TILE: u32 = 256;
/// Opaque black, UCBLogo's default background.
const BACKGROUND: [u8; 4] = [0, 0, 0, 255];

pub struct PixelCanvas {
    width: u32,
    height: u32,
    /// Premultiplied pixels, drawn with red and blue swapped so the bytes
    /// are already the BGRA that gpui wants (spike 02).
    data: Vec<u8>,
    /// Every segment since the run started, to re-render on resize (ADR 0003).
    display: Vec<Segment>,
    pending: Vec<Segment>,
    tiles: Vec<Option<Arc<RenderImage>>>,
    dirty: Vec<bool>,
    columns: u32,
    context: RenderContext,
    resources: Resources,
    threads: u16,
    /// Recent drawing cost per segment, for sizing Instant mode's frames.
    pub per_segment: Duration,
    pub last_draw: Duration,
    pub last_upload: Duration,
    pub last_paint: Duration,
    pub rerender: Option<(usize, Duration)>,
}

fn settings(threads: u16) -> RenderSettings {
    RenderSettings { num_threads: threads, ..RenderSettings::default() }
}

impl PixelCanvas {
    pub fn new() -> PixelCanvas {
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get().saturating_sub(1).max(1)) as u16;
        PixelCanvas {
            width: 0,
            height: 0,
            data: Vec::new(),
            display: Vec::new(),
            pending: Vec::new(),
            tiles: Vec::new(),
            dirty: Vec::new(),
            columns: 0,
            context: RenderContext::new_with(1, 1, settings(threads)),
            resources: Resources::new(),
            threads,
            per_segment: Duration::from_micros(2),
            last_draw: Duration::ZERO,
            last_upload: Duration::ZERO,
            last_paint: Duration::ZERO,
            rerender: None,
        }
    }

    /// A new run starts with a clear screen.
    pub fn clean(&mut self) {
        self.display.clear();
        self.pending.clear();
        self.fill_background();
    }

    pub fn segments(&self) -> usize {
        self.display.len()
    }

    pub fn push(&mut self, segments: &[Segment]) {
        self.display.extend_from_slice(segments);
        self.pending.extend_from_slice(segments);
    }

    fn fill_background(&mut self) {
        for pixel in self.data.as_chunks_mut::<4>().0 {
            pixel.copy_from_slice(&BACKGROUND);
        }
        self.dirty.iter_mut().for_each(|d| *d = true);
    }

    /// Device pixels per turtle step, and the device position of 0, 0.
    fn view(&self) -> (f64, f64, f64) {
        let scale = self.width.min(self.height) as f64 / TURTLE_AREA;
        (scale, self.width as f64 / 2.0, self.height as f64 / 2.0)
    }

    fn to_device(&self, p: Point) -> (f64, f64) {
        let (scale, cx, cy) = self.view();
        (cx + p.x * scale, cy - p.y * scale)
    }

    fn resize(&mut self, width: u32, height: u32, window: &mut Window) {
        self.width = width;
        self.height = height;
        self.data = vec![0; width as usize * height as usize * 4];
        self.context = RenderContext::new_with(width as u16, height as u16, settings(self.threads));
        self.columns = width.div_ceil(TILE);
        for tile in self.tiles.drain(..).flatten() {
            window.drop_image(tile).ok();
        }
        let count = (self.columns * height.div_ceil(TILE)) as usize;
        self.tiles = vec![None; count];
        self.dirty = vec![true; count];
        self.fill_background();
        let started = Instant::now();
        let display = std::mem::take(&mut self.display);
        self.draw(&display);
        if !display.is_empty() {
            self.rerender = Some((display.len(), started.elapsed()));
        }
        self.display = display;
        self.pending.clear();
    }

    fn draw(&mut self, segments: &[Segment]) {
        if segments.is_empty() {
            return;
        }
        let started = Instant::now();
        let (scale, ..) = self.view();
        self.context.reset();
        for segment in segments {
            let (fx, fy) = self.to_device(segment.from);
            let (tx, ty) = self.to_device(segment.to);
            let mut path = BezPath::new();
            path.move_to((fx, fy));
            path.line_to((tx, ty));
            let [r, g, b, a] = segment.pen.color;
            // Red and blue swapped, so the buffer is BGRA.
            self.context.set_paint(vello_cpu::color::AlphaColor::from_rgba8(b, g, r, a));
            let width = (segment.pen.size * scale).max(1.0);
            self.context.set_stroke(Stroke::new(width).with_caps(Cap::Round));
            self.context.stroke_path(&path);
            self.mark_dirty(fx.min(tx), fy.min(ty), fx.max(tx), fy.max(ty), width / 2.0 + 2.0);
        }
        self.context.flush();
        let target = vello_cpu::PixmapMut::new(self.width as u16, self.height as u16, &mut self.data).expect("size matches");
        let settings = RasterizerSettings { target_init: TargetInit::SrcOver, ..RasterizerSettings::default() };
        self.context.render_with(target, &mut self.resources, settings);
        self.last_draw = started.elapsed();
        // Smooth the estimate, which sizes Instant mode's frames.
        let each = self.last_draw / segments.len() as u32;
        self.per_segment = (self.per_segment * 3 + each) / 4;
    }

    fn mark_dirty(&mut self, left: f64, top: f64, right: f64, bottom: f64, margin: f64) {
        if self.columns == 0 {
            return;
        }
        let rows = self.dirty.len() as u32 / self.columns;
        let clamp = |v: f64, max: u32| (v.max(0.0) as u32).min(max.saturating_sub(1));
        let (x0, x1) = (clamp(left - margin, self.width) / TILE, clamp(right + margin, self.width) / TILE);
        let (y0, y1) = (clamp(top - margin, self.height) / TILE, clamp(bottom + margin, self.height) / TILE);
        for ty in y0..=y1.min(rows - 1) {
            for tx in x0..=x1.min(self.columns - 1) {
                self.dirty[(ty * self.columns + tx) as usize] = true;
            }
        }
    }

    fn upload(&mut self, window: &mut Window) {
        let started = Instant::now();
        let stride = self.width as usize * 4;
        for index in 0..self.dirty.len() {
            if !std::mem::take(&mut self.dirty[index]) {
                continue;
            }
            let (tx, ty) = (index as u32 % self.columns * TILE, index as u32 / self.columns * TILE);
            let (w, h) = (TILE.min(self.width - tx), TILE.min(self.height - ty));
            let mut bytes = Vec::with_capacity(w as usize * h as usize * 4);
            for y in ty..ty + h {
                let start = y as usize * stride + tx as usize * 4;
                bytes.extend_from_slice(&self.data[start..start + w as usize * 4]);
            }
            let buffer = image::RgbaImage::from_raw(w, h, bytes).expect("buffer matches size");
            let image = Arc::new(RenderImage::new([image::Frame::new(buffer)]));
            // The old tile isn't painted this frame, so it can go now.
            if let Some(old) = self.tiles[index].replace(image) {
                window.drop_image(old).ok();
            }
        }
        self.last_upload = started.elapsed();
    }

    /// Paints the canvas: the buffer's tiles, then the move in progress and
    /// the turtle.
    pub fn paint(&mut self, bounds: Bounds<Pixels>, window: &mut Window, partial: Option<Segment>, turtle: Option<(Point, f64)>) {
        let started = Instant::now();
        self.paint_inner(bounds, window, partial, turtle);
        self.last_paint = started.elapsed();
    }

    fn paint_inner(&mut self, bounds: Bounds<Pixels>, window: &mut Window, partial: Option<Segment>, turtle: Option<(Point, f64)>) {
        let per_device = window.scale_factor();
        let device = (
            (bounds.size.width.as_f32() * per_device).round() as u32,
            (bounds.size.height.as_f32() * per_device).round() as u32,
        );
        if device.0 == 0 || device.1 == 0 {
            return;
        }
        if (self.width, self.height) != device {
            self.resize(device.0, device.1, window);
        }
        let pending = std::mem::take(&mut self.pending);
        self.draw(&pending);
        self.upload(window);

        let per_tile = TILE as f32 / per_device;
        for (index, tile) in self.tiles.iter().enumerate() {
            let Some(tile) = tile else { continue };
            let (tx, ty) = (index as u32 % self.columns, index as u32 / self.columns);
            let size = tile.size(0);
            let tile_bounds = Bounds::new(
                point(bounds.origin.x + px(tx as f32 * per_tile), bounds.origin.y + px(ty as f32 * per_tile)),
                gpui_kit::size(px(size.width.0 as f32 / per_device), px(size.height.0 as f32 / per_device)),
            );
            window.paint_image(tile_bounds, tile_bounds, Corners::default(), tile.clone(), 0, false).ok();
        }

        let (scale, ..) = self.view();
        let logical = |p: Point| {
            let (x, y) = self.to_device(p);
            point(bounds.origin.x + px(x as f32 / per_device), bounds.origin.y + px(y as f32 / per_device))
        };
        if let Some(segment) = partial {
            let width = ((segment.pen.size * scale).max(1.0) as f32) / per_device;
            let mut path = PathBuilder::stroke(px(width));
            path.move_to(logical(segment.from));
            path.line_to(logical(segment.to));
            let [r, g, b, a] = segment.pen.color;
            let color = Rgba { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: a as f32 / 255.0 };
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        }
        if let Some((pos, heading)) = turtle {
            // An isosceles triangle, its short side's midpoint at the turtle's
            // position (the plan's "Turtle sprite").
            let size = 18.0 / scale.max(0.01) * (per_device as f64);
            let (s, c) = heading.to_radians().sin_cos();
            let at = |forward: f64, right: f64| Point { x: pos.x + forward * s + right * c, y: pos.y + forward * c - right * s };
            let mut tri = PathBuilder::fill();
            tri.move_to(logical(at(size, 0.0)));
            tri.line_to(logical(at(0.0, size * 0.4)));
            tri.line_to(logical(at(0.0, -size * 0.4)));
            tri.close();
            if let Ok(tri) = tri.build() {
                window.paint_path(tri, rgb(0x4caf50));
            }
        }
    }
}
