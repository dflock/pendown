//! Spike 02: streaming a full-resolution pixel buffer into gpui.
//!
//! A random-walk turtle draws a batch of anti-aliased segments into a
//! tiny-skia pixmap every frame, and the pixmap is shown with
//! `window.paint_image`. Each phase in `PHASES` tries one way of getting the
//! pixels to the screen.
//!
//! Run with `cargo run --release`. Set `SPIKE_AUTOTEST=1` to run the timed
//! phases, print measurements and save screenshots, or `SPIKE_SOAK=<seconds>`
//! to stream 2560 × 1600 frames drawn with vello_cpu for that long and print memory every 10 s.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui_kit::*;
use spike_02_pixel_buffer_streaming::tiny_skia::{IntRect, Pixmap};
use spike_02_pixel_buffer_streaming::*;

const TITLE: &str = "Pendown spike 02";
const TILE: u32 = 256;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Mode {
    /// Convert the whole pixmap into a new image each frame it changes.
    Full { drop_old: bool },
    /// Split the pixmap into tiles, and replace only the tiles that changed.
    Tiles,
    /// Paint new segments as vector paths, and only flatten them into the
    /// pixmap every so many frames.
    Vector { flatten_every: usize },
    /// A test pattern comparing tiny-skia's blending with gpui's.
    Blend,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Rasterizer {
    TinySkia,
    Vello,
}

struct Phase {
    name: &'static str,
    secs: u64,
    mode: Mode,
    rasterizer: Rasterizer,
    /// Pixmap pixels per device pixel. 2 on this 1× display gives the
    /// 2560 × 1600 buffer a HiDPI display would need.
    oversample: u32,
    /// Segments drawn at the start of the phase.
    initial: usize,
    per_frame: usize,
    resize: bool,
    screenshot: bool,
}

const FULL: Mode = Mode::Full { drop_old: true };
const SKIA: Rasterizer = Rasterizer::TinySkia;
const VELLO: Rasterizer = Rasterizer::Vello;

const PHASES: &[Phase] = &[
    Phase { name: "warm-up", secs: 2, mode: FULL, rasterizer: SKIA, oversample: 1, initial: 0, per_frame: 500, resize: false, screenshot: false },
    Phase { name: "static 2x: 20k segments, then nothing changes", secs: 3, mode: FULL, rasterizer: SKIA, oversample: 2, initial: 20_000, per_frame: 0, resize: false, screenshot: false },
    Phase { name: "full upload 1x, tiny-skia, 500 segments a frame", secs: 5, mode: FULL, rasterizer: SKIA, oversample: 1, initial: 0, per_frame: 500, resize: false, screenshot: false },
    Phase { name: "full upload 2x, tiny-skia, 20 segments a frame", secs: 5, mode: FULL, rasterizer: SKIA, oversample: 2, initial: 0, per_frame: 20, resize: false, screenshot: false },
    Phase { name: "full upload 2x, tiny-skia, 500 segments a frame", secs: 5, mode: FULL, rasterizer: SKIA, oversample: 2, initial: 0, per_frame: 500, resize: false, screenshot: true },
    Phase { name: "full upload 2x, vello_cpu, 500 segments a frame", secs: 5, mode: FULL, rasterizer: VELLO, oversample: 2, initial: 0, per_frame: 500, resize: false, screenshot: false },
    Phase { name: "full upload 2x, vello_cpu, 2000 segments a frame", secs: 5, mode: FULL, rasterizer: VELLO, oversample: 2, initial: 0, per_frame: 2000, resize: false, screenshot: false },
    Phase { name: "tiles 2x, vello_cpu, 500 segments a frame", secs: 5, mode: Mode::Tiles, rasterizer: VELLO, oversample: 2, initial: 0, per_frame: 500, resize: false, screenshot: false },
    Phase { name: "tiles 1x, tiny-skia, 500 segments a frame", secs: 5, mode: Mode::Tiles, rasterizer: SKIA, oversample: 1, initial: 0, per_frame: 500, resize: false, screenshot: true },
    Phase { name: "vector 2x, tiny-skia, 20 segments a frame, flatten every 30 frames", secs: 5, mode: Mode::Vector { flatten_every: 30 }, rasterizer: SKIA, oversample: 2, initial: 0, per_frame: 20, resize: false, screenshot: false },
    Phase { name: "resize every second 2x, vello_cpu, 500 segments a frame", secs: 7, mode: FULL, rasterizer: VELLO, oversample: 2, initial: 0, per_frame: 500, resize: true, screenshot: false },
    Phase { name: "resize every second 2x, tiny-skia, 500 segments a frame", secs: 7, mode: FULL, rasterizer: SKIA, oversample: 2, initial: 0, per_frame: 500, resize: true, screenshot: false },
    Phase { name: "full upload 1x without drop_image", secs: 2, mode: Mode::Full { drop_old: false }, rasterizer: SKIA, oversample: 1, initial: 0, per_frame: 500, resize: false, screenshot: false },
    Phase { name: "after dropping the leaked images", secs: 2, mode: FULL, rasterizer: SKIA, oversample: 1, initial: 0, per_frame: 500, resize: false, screenshot: false },
    Phase { name: "blend: tiny-skia above, gpui paths below", secs: 2, mode: Mode::Blend, rasterizer: SKIA, oversample: 1, initial: 0, per_frame: 0, resize: false, screenshot: true },
    Phase { name: "finished", secs: 1, mode: FULL, rasterizer: SKIA, oversample: 1, initial: 0, per_frame: 0, resize: false, screenshot: false },
];

/// The phase that runs without `SPIKE_AUTOTEST`, and for `SPIKE_SOAK`.
const DEFAULT_PHASE: usize = 5;

/// What the canvas paints with, shared with its paint callback.
struct Canvas {
    mode: Mode,
    rasterizer: Rasterizer,
    vello: vello::VelloDrawer,
    oversample: u32,
    pixmap: Option<Pixmap>,
    /// Every segment since the last clean, for re-rendering on resize.
    display: Vec<Segment>,
    /// Segments added since the last paint.
    pending: Vec<Segment>,
    /// Vector mode: segments painted as paths, not yet in the pixmap.
    recent: Vec<Segment>,
    frames_since_flatten: usize,
    image: Option<Arc<RenderImage>>,
    image_stale: bool,
    tiles: Vec<Option<Arc<RenderImage>>>,
    tile_columns: u32,
    dirty: Vec<bool>,
    /// Images that were never dropped, kept so the leak can be undone.
    leaked: Vec<Arc<RenderImage>>,
    drop_leaked: bool,
    blend_image: Option<Arc<RenderImage>>,
    draw_cost: Samples,
    convert_cost: Samples,
    paint_cost: Samples,
    rerenders: Vec<(usize, (u32, u32), Duration)>,
    flatten_frames: usize,
}

impl Canvas {
    fn new() -> Self {
        Self {
            mode: FULL,
            rasterizer: SKIA,
            vello: vello::VelloDrawer::new(7),
            oversample: 1,
            pixmap: None,
            display: vec![],
            pending: vec![],
            recent: vec![],
            frames_since_flatten: 0,
            image: None,
            image_stale: true,
            tiles: vec![],
            tile_columns: 0,
            dirty: vec![],
            leaked: vec![],
            drop_leaked: false,
            blend_image: None,
            draw_cost: Samples::default(),
            convert_cost: Samples::default(),
            paint_cost: Samples::default(),
            rerenders: vec![],
            flatten_frames: 0,
        }
    }

    /// Draws segments with red and blue swapped, so that the pixmap's bytes
    /// are already in the BGRA order gpui wants. (The white background is
    /// the same either way.)
    fn draw(&mut self, segments: &[Segment]) -> Vec<IntRect> {
        let pixmap = self.pixmap.as_mut().expect("fitted first");
        match self.rasterizer {
            Rasterizer::Vello => {
                self.vello.draw(pixmap, segments, true);
                // The tiles need the touched rectangles; work them out without drawing.
                let view = TurtleView::fit(pixmap.width(), pixmap.height());
                segments.iter().filter_map(|s| touched(pixmap, &view, s)).collect()
            }
            Rasterizer::TinySkia => {
                let view = TurtleView::fit(pixmap.width(), pixmap.height());
                segments
                    .iter()
                    .filter_map(|s| draw_segment(pixmap, &view, &Segment { color: swap_red_blue(s.color), ..*s }))
                    .collect()
            }
        }
    }

    fn clean(&mut self) {
        self.display.clear();
        self.pending.clear();
        self.recent.clear();
        if let Some(pixmap) = &mut self.pixmap {
            clear(pixmap);
        }
        self.mark_all_dirty();
    }

    fn mark_all_dirty(&mut self) {
        self.image_stale = true;
        self.dirty.iter_mut().for_each(|d| *d = true);
    }

    fn mark_dirty(&mut self, rect: IntRect) {
        if self.tile_columns == 0 {
            return;
        }
        let rows = self.dirty.len() as u32 / self.tile_columns;
        for ty in (rect.top() as u32 / TILE)..=((rect.bottom() as u32 - 1) / TILE).min(rows - 1) {
            for tx in (rect.left() as u32 / TILE)..=((rect.right() as u32 - 1) / TILE).min(self.tile_columns - 1) {
                self.dirty[(ty * self.tile_columns + tx) as usize] = true;
            }
        }
    }

    /// Makes the pixmap match the canvas, re-rendering from the display list
    /// if the size changed.
    fn fit(&mut self, device: (u32, u32), window: &mut Window) {
        if self.pixmap.as_ref().is_some_and(|p| (p.width(), p.height()) == device) {
            return;
        }
        let Some(mut pixmap) = Pixmap::new(device.0, device.1) else { return };
        clear(&mut pixmap);
        let resized = self.pixmap.is_some();
        self.pixmap = Some(pixmap);
        let started = Instant::now();
        let display = std::mem::take(&mut self.display);
        self.draw(&display);
        self.display = display;
        if resized {
            self.rerenders.push((self.display.len(), device, started.elapsed()));
        }
        self.tile_columns = device.0.div_ceil(TILE);
        let tile_count = (self.tile_columns * device.1.div_ceil(TILE)) as usize;
        for tile in self.tiles.drain(..).flatten() {
            window.drop_image(tile).ok();
        }
        self.tiles = vec![None; tile_count];
        self.dirty = vec![true; tile_count];
        self.image_stale = true;
    }

    fn retire(&mut self, image: Arc<RenderImage>, window: &mut Window) {
        match self.mode {
            Mode::Full { drop_old: false } => self.leaked.push(image),
            _ => {
                window.drop_image(image).ok();
            }
        }
    }

    fn paint(&mut self, bounds: Bounds<Pixels>, window: &mut Window) {
        if self.drop_leaked {
            self.drop_leaked = false;
            for image in self.leaked.drain(..) {
                window.drop_image(image).ok();
            }
        }
        let per_device = window.scale_factor() * self.oversample as f32;
        let device = (
            (bounds.size.width.as_f32() * per_device).round() as u32,
            (bounds.size.height.as_f32() * per_device).round() as u32,
        );
        if device.0 == 0 || device.1 == 0 {
            return;
        }
        self.fit(device, window);

        let started = Instant::now();
        let pending = std::mem::take(&mut self.pending);
        let mut drew = false;
        match self.mode {
            Mode::Vector { flatten_every } => {
                self.recent.extend_from_slice(&pending);
                self.frames_since_flatten += 1;
                if self.frames_since_flatten >= flatten_every && !self.recent.is_empty() {
                    self.frames_since_flatten = 0;
                    self.flatten_frames += 1;
                    let recent = std::mem::take(&mut self.recent);
                    self.draw(&recent);
                    self.image_stale = true;
                    drew = true;
                }
            }
            Mode::Blend => {}
            _ => {
                let touched = self.draw(&pending);
                for rect in touched {
                    self.mark_dirty(rect);
                }
                if !pending.is_empty() {
                    self.image_stale = true;
                    drew = true;
                }
            }
        }
        if drew {
            self.draw_cost.push(started.elapsed());
        }

        let started = Instant::now();
        let mut converted = false;
        let pixmap = self.pixmap.as_ref().expect("fitted above");
        match self.mode {
            Mode::Tiles => {
                let rows = self.dirty.len() as u32 / self.tile_columns;
                for index in 0..self.dirty.len() {
                    if !self.dirty[index] {
                        continue;
                    }
                    self.dirty[index] = false;
                    let (tx, ty) = (index as u32 % self.tile_columns, index as u32 / self.tile_columns);
                    let rect = IntRect::from_xywh(
                        (tx * TILE) as i32,
                        (ty * TILE) as i32,
                        TILE.min(pixmap.width() - tx * TILE),
                        TILE.min(pixmap.height() - ty * TILE),
                    )
                    .expect("tile inside pixmap");
                    let bgra = copy_tile(pixmap, rect);
                    let image = render_image(rect.width(), rect.height(), bgra);
                    if let Some(old) = self.tiles[index].replace(image) {
                        window.drop_image(old).ok();
                    }
                    converted = true;
                }
                debug_assert!(rows > 0);
            }
            _ if self.image_stale || self.image.is_none() => {
                self.image_stale = false;
                // Already BGRA: see `draw`.
                let image = render_image(pixmap.width(), pixmap.height(), pixmap.data().to_vec());
                if let Some(old) = self.image.replace(image) {
                    self.retire(old, window);
                }
                converted = true;
            }
            _ => {}
        }
        if converted {
            self.convert_cost.push(started.elapsed());
        }

        let started = Instant::now();
        match self.mode {
            Mode::Tiles => {
                let per_tile = TILE as f32 / per_device;
                for (index, tile) in self.tiles.iter().enumerate() {
                    let Some(tile) = tile else { continue };
                    let (tx, ty) = (index as u32 % self.tile_columns, index as u32 / self.tile_columns);
                    let size = tile.size(0);
                    let tile_bounds = Bounds::new(
                        point(bounds.origin.x + px(tx as f32 * per_tile), bounds.origin.y + px(ty as f32 * per_tile)),
                        gpui_kit::size(px(size.width.0 as f32 / per_device), px(size.height.0 as f32 / per_device)),
                    );
                    window.paint_image(tile_bounds, tile_bounds, Corners::default(), tile.clone(), 0, false).ok();
                }
            }
            Mode::Blend => paint_blend_pattern(bounds, &mut self.blend_image, window),
            _ => {
                if let Some(image) = &self.image {
                    window.paint_image(bounds, bounds, Corners::default(), image.clone(), 0, false).ok();
                }
                if let Mode::Vector { .. } = self.mode {
                    let pixmap = self.pixmap.as_ref().expect("fitted above");
                    paint_segments(&self.recent, TurtleView::fit(pixmap.width(), pixmap.height()), per_device, bounds, window);
                }
            }
        }
        self.paint_cost.push(started.elapsed());
    }
}

/// The rectangle a segment would touch, without drawing it.
fn touched(pixmap: &Pixmap, view: &TurtleView, segment: &Segment) -> Option<IntRect> {
    let (from, to) = (view.map(segment.from), view.map(segment.to));
    let margin = (segment.width * view.scale).max(1.0) / 2.0 + 2.0;
    IntRect::from_ltrb(
        (from.0.min(to.0) - margin).floor().max(0.0) as i32,
        (from.1.min(to.1) - margin).floor().max(0.0) as i32,
        ((from.0.max(to.0) + margin).ceil() as i32).min(pixmap.width() as i32),
        ((from.1.max(to.1) + margin).ceil() as i32).min(pixmap.height() as i32),
    )
}

fn copy_tile(pixmap: &Pixmap, rect: IntRect) -> Vec<u8> {
    let stride = pixmap.width() as usize * 4;
    let mut bytes = Vec::with_capacity(rect.width() as usize * rect.height() as usize * 4);
    for y in rect.top()..rect.bottom() {
        let start = y as usize * stride + rect.left() as usize * 4;
        bytes.extend_from_slice(&pixmap.data()[start..start + rect.width() as usize * 4]);
    }
    bytes
}

fn render_image(width: u32, height: u32, bgra: Vec<u8>) -> Arc<RenderImage> {
    // gpui stores frames as `RgbaImage`, but expects the bytes in BGRA order.
    let buffer = image::RgbaImage::from_raw(width, height, bgra).expect("buffer matches size");
    Arc::new(RenderImage::new([image::Frame::new(buffer)]))
}

fn gpui_colour(color: [u8; 4]) -> Rgba {
    Rgba {
        r: color[0] as f32 / 255.0,
        g: color[1] as f32 / 255.0,
        b: color[2] as f32 / 255.0,
        a: color[3] as f32 / 255.0,
    }
}

/// Paints segments as gpui paths, one path per run of the same colour.
fn paint_segments(segments: &[Segment], view: TurtleView, per_device: f32, bounds: Bounds<Pixels>, window: &mut Window) {
    let to_window = |p: (f32, f32)| {
        let (x, y) = view.map(p);
        point(bounds.origin.x + px(x / per_device), bounds.origin.y + px(y / per_device))
    };
    let mut start = 0;
    while start < segments.len() {
        let colour = segments[start].color;
        let mut end = start;
        while end < segments.len() && segments[end].color == colour {
            end += 1;
        }
        let width = (segments[start].width * view.scale).max(1.0) / per_device;
        let mut path = PathBuilder::stroke(px(width));
        let mut at = None;
        for segment in &segments[start..end] {
            if at != Some(segment.from) {
                path.move_to(to_window(segment.from));
            }
            path.line_to(to_window(segment.to));
            at = Some(segment.to);
        }
        if let Ok(path) = path.build() {
            window.paint_path(path, gpui_colour(colour));
        }
        start = end;
    }
}

const BLEND_RED: [u8; 4] = [255, 0, 0, 128];
const BLEND_BLUE: [u8; 4] = [0, 0, 255, 255];

/// Question 6: the same half-transparent red strokes over blue, drawn by
/// tiny-skia (top) and by gpui (bottom), to compare the blended pixels.
/// Each row has a single stroke, and two strokes that meet at a corner.
fn paint_blend_pattern(bounds: Bounds<Pixels>, cached: &mut Option<Arc<RenderImage>>, window: &mut Window) {
    let origin = bounds.origin;
    let at = |x: f32, y: f32| point(origin.x + px(x), origin.y + px(y));
    window.paint_quad(fill(Bounds::new(origin, bounds.size), gpui_kit::white()));
    window.paint_quad(fill(Bounds::new(at(20., 20.), gpui_kit::size(px(400.), px(320.))), gpui_colour(BLEND_BLUE)));

    let image = cached.get_or_insert_with(|| blend_image(window.scale_factor())).clone();
    let image_bounds = Bounds::new(at(0., 0.), gpui_kit::size(px(440.), px(160.)));
    // Not dropped here: dropping an image in the frame that paints it
    // removes it from the atlas before the frame is drawn.
    window.paint_image(image_bounds, image_bounds, Corners::default(), image, 0, false).ok();

    let paint = |window: &mut Window, points: &[(f32, f32)]| {
        let mut path = PathBuilder::stroke(px(20.));
        path.move_to(at(points[0].0, points[0].1 + 160.));
        for p in &points[1..] {
            path.line_to(at(p.0, p.1 + 160.));
        }
        if let Ok(path) = path.build() {
            window.paint_path(path, gpui_colour(BLEND_RED));
        }
    };
    paint(window, &[(40., 40.), (200., 40.)]);
    paint(window, &[(240., 40.), (380., 40.)]);
    paint(window, &[(380., 40.), (380., 140.)]);
}

fn blend_image(scale: f32) -> Arc<RenderImage> {
    let width = (440.0 * scale) as u32;
    let height = (160.0 * scale) as u32;
    let mut pixmap = Pixmap::new(width, height).expect("non-zero size");
    let [r, g, b, a] = BLEND_BLUE;
    pixmap.fill(tiny_skia::Color::from_rgba8(r, g, b, a));
    let stroke = tiny_skia::Stroke { width: 20.0 * scale, ..Default::default() };
    let line = |pixmap: &mut Pixmap, points: &[(f32, f32)]| {
        let mut path = tiny_skia::PathBuilder::new();
        path.move_to(points[0].0 * scale, points[0].1 * scale);
        for p in &points[1..] {
            path.line_to(p.0 * scale, p.1 * scale);
        }
        if let Some(path) = path.finish() {
            pixmap.stroke_path(&path, &paint_for(BLEND_RED), &stroke, tiny_skia::Transform::identity(), None);
        }
    };
    line(&mut pixmap, &[(40., 40.), (200., 40.)]);
    // Two separate segments, as two Logo moves draw them.
    line(&mut pixmap, &[(240., 40.), (380., 40.)]);
    line(&mut pixmap, &[(380., 40.), (380., 140.)]);
    let mut bgra = vec![];
    to_bgra_opaque(pixmap.data(), &mut bgra);
    render_image(width, height, bgra)
}

struct Spike {
    canvas: Rc<RefCell<Canvas>>,
    turtle: Turtle,
    auto: Option<usize>,
    soak: Option<Duration>,
    started: Instant,
    phase_started: Instant,
    last_memory: Instant,
    last_resize: Instant,
    small: bool,
    last_frame: Option<Instant>,
    frame_intervals: Samples,
    screenshots: usize,
}

impl Spike {
    fn new() -> Self {
        let auto = std::env::var_os("SPIKE_AUTOTEST").map(|_| 0);
        let soak = std::env::var("SPIKE_SOAK").ok().and_then(|s| s.parse().ok()).map(Duration::from_secs);
        let mut this = Self {
            canvas: Rc::new(RefCell::new(Canvas::new())),
            turtle: Turtle::new(42),
            auto,
            soak,
            started: Instant::now(),
            phase_started: Instant::now(),
            last_memory: Instant::now(),
            last_resize: Instant::now(),
            small: false,
            last_frame: None,
            frame_intervals: Samples::default(),
            screenshots: 0,
        };
        if this.soak.is_some() {
            this.auto = None;
            this.start_phase(DEFAULT_PHASE);
        } else if this.auto.is_some() {
            this.start_phase(0);
        } else {
            this.start_phase(DEFAULT_PHASE);
        }
        this
    }

    fn phase(&self) -> &'static Phase {
        &PHASES[self.auto.unwrap_or(DEFAULT_PHASE)]
    }

    fn start_phase(&mut self, index: usize) {
        let phase = &PHASES[index];
        let mut canvas = self.canvas.borrow_mut();
        canvas.clean();
        canvas.mode = phase.mode;
        canvas.rasterizer = phase.rasterizer;
        canvas.oversample = phase.oversample;
        canvas.frames_since_flatten = 0;
        if index > 0 && PHASES[index - 1].mode == (Mode::Full { drop_old: false }) {
            canvas.drop_leaked = true;
        }
        for _ in 0..phase.initial {
            let segment = self.turtle.step();
            canvas.display.push(segment);
            canvas.pending.push(segment);
        }
        self.phase_started = Instant::now();
    }

    fn report(&mut self) {
        let mut canvas = self.canvas.borrow_mut();
        let (rss, gpu) = memory_mib();
        let size = canvas.pixmap.as_ref().map(|p| (p.width(), p.height()));
        println!("  Pixmap: {size:?}, display list {} segments", canvas.display.len());
        println!("  Frame intervals: {}", self.frame_intervals.summary());
        println!("  Draw segments: {}", canvas.draw_cost.summary());
        println!("  Convert + new image: {}", canvas.convert_cost.summary());
        println!("  Paint calls: {}", canvas.paint_cost.summary());
        if canvas.flatten_frames > 0 {
            println!("  Flattened {} times", canvas.flatten_frames);
        }
        for (count, size, took) in canvas.rerenders.drain(..) {
            println!("  Re-rendered {count} segments at {size:?} in {:.1}ms", took.as_secs_f64() * 1000.0);
        }
        println!("  Memory: RSS {rss:.0} MiB, GPU buffers {gpu:.0} MiB, leaked images {}", canvas.leaked.len());
        canvas.draw_cost = Samples::default();
        canvas.convert_cost = Samples::default();
        canvas.paint_cost = Samples::default();
        canvas.flatten_frames = 0;
        self.frame_intervals = Samples::default();
        self.last_frame = None;
    }

    fn screenshot(&mut self, name: &str) {
        self.screenshots += 1;
        let path = format!("screenshot-{}-{name}.png", self.screenshots);
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("xwd -silent -name '{TITLE}' | convert xwd:- '{path}'"))
            .status();
        println!("Screenshot {path}: {:?}", status.map(|s| s.success()));
    }

    fn tick(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let phase = self.phase();
        for _ in 0..phase.per_frame {
            let segment = self.turtle.step();
            let mut canvas = self.canvas.borrow_mut();
            canvas.display.push(segment);
            canvas.pending.push(segment);
        }
        if phase.resize && self.last_resize.elapsed() >= Duration::from_secs(1) {
            self.last_resize = Instant::now();
            self.small = !self.small;
            let (w, h) = if self.small { (1000., 640.) } else { (1280., 800.) };
            window.resize(gpui_kit::size(px(w), px(h)));
        }

        if let Some(soak) = self.soak {
            if self.last_memory.elapsed() >= Duration::from_secs(10) {
                self.last_memory = Instant::now();
                println!("--- {:.0} s:", self.started.elapsed().as_secs_f64());
                self.report();
                // CLEAN every 10 s, so the display list doesn't grow without bound.
                self.canvas.borrow_mut().clean();
            }
            if self.started.elapsed() >= soak {
                cx.quit();
            }
            return;
        }

        let Some(index) = self.auto else { return };
        if self.phase_started.elapsed() < Duration::from_secs(phase.secs) {
            return;
        }
        println!("--- {}:", phase.name);
        self.report();
        if phase.screenshot {
            self.screenshot(phase.name.split([' ', ',', ':']).next().unwrap_or("phase"));
        }
        if phase.resize {
            window.resize(gpui_kit::size(px(1280.), px(800.)));
        }
        if index + 1 == PHASES.len() {
            self.auto = None;
            cx.quit();
            return;
        }
        self.auto = Some(index + 1);
        self.start_phase(index + 1);
    }
}

impl Render for Spike {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let now = Instant::now();
        if let Some(last) = self.last_frame {
            self.frame_intervals.push(now - last);
        }
        self.last_frame = Some(now);
        self.tick(window, cx);
        window.request_animation_frame();

        let canvas = self.canvas.clone();
        div().size_full().bg(gpui_kit::white()).child(
            gpui_kit::canvas(|bounds, _, _| bounds, move |_, bounds, window, _| canvas.borrow_mut().paint(bounds, window))
                .size_full(),
        )
    }
}

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions { title: Some(TITLE.into()), ..Default::default() }),
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, size(px(1280.), px(800.)), cx))),
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| Spike::new()))
            .expect("failed to open window");
        cx.activate(true);
    });
}
