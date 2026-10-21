//! The drawing (ADRs 0003 and 0010): a display list of everything drawn
//! since the last `CLEAN`, rendered into a pixel buffer by vello_cpu. The GUI
//! shows the buffer as 256 × 256 tiles, uploading only the ones that
//! changed; the CLI saves it as a PNG. There's no gpui here, so both can
//! use it.

use std::{
    io::Cursor,
    time::{Duration, Instant},
};

use pendown_logo::{Pen, Point};
use vello_cpu::{
    RasterizerSettings, RenderContext, RenderSettings, Resources, TargetInit,
    kurbo::{BezPath, Cap, Stroke},
};

/// The turtle area's default size in turtle steps: from -500 to 500 on both
/// axes (ADR 0003).
pub const TURTLE_AREA: f64 = 1000.0;

/// The size of the tiles the GUI uploads (spike 02).
pub const TILE: u32 = 256;

/// UCBLogo's default background.
pub const BACKGROUND: [u8; 4] = [0, 0, 0, 255];

/// A finished move.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub from: Point,
    pub to: Point,
    pub pen: Pen,
}

/// A part of the buffer that changed, in device pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tile {
    /// The tile's index, row by row.
    pub index: usize,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub struct Drawing {
    width: u32,
    height: u32,
    /// Premultiplied pixels, drawn with red and blue swapped so the bytes
    /// are already the BGRA that gpui wants (spike 02). The background is
    /// always opaque, so premultiplied and straight are the same.
    data: Vec<u8>,
    /// Everything drawn since the last `CLEAN`, to re-render at a new size.
    display: Vec<Segment>,
    pending: Vec<Segment>,
    dirty: Vec<bool>,
    columns: u32,
    area: f64,
    context: RenderContext,
    resources: Resources,
    threads: u16,
    /// Recent drawing cost per segment, which the GUI uses to size Instant
    /// mode's frames.
    pub per_segment: Duration,
    pub last_draw: Duration,
}

fn settings(threads: u16) -> RenderSettings {
    RenderSettings { num_threads: threads, ..RenderSettings::default() }
}

impl Default for Drawing {
    fn default() -> Drawing {
        Drawing::new()
    }
}

impl Drawing {
    /// An empty drawing. vello_cpu uses one worker thread fewer than the CPUs
    /// (ADR 0010).
    pub fn new() -> Drawing {
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get().saturating_sub(1).max(1)) as u16;
        Drawing {
            width: 0,
            height: 0,
            data: Vec::new(),
            display: Vec::new(),
            pending: Vec::new(),
            dirty: Vec::new(),
            columns: 0,
            area: TURTLE_AREA,
            context: RenderContext::new_with(1, 1, settings(threads)),
            resources: Resources::new(),
            threads,
            per_segment: Duration::from_micros(2),
            last_draw: Duration::ZERO,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// How many segments have been drawn since the last `CLEAN`.
    pub fn segments(&self) -> usize {
        self.display.len()
    }

    /// Clears the drawing, as `CLEAN` and Run do.
    pub fn clean(&mut self) {
        self.display.clear();
        self.pending.clear();
        self.fill_background();
    }

    /// Adds finished moves. They're drawn by the next `draw_pending`.
    pub fn push(&mut self, segments: &[Segment]) {
        self.display.extend_from_slice(segments);
        self.pending.extend_from_slice(segments);
    }

    /// Sets the turtle area's size in turtle steps, to match the
    /// interpreter's (`Interpreter::set_area`). It's fitted to the buffer
    /// without stretching, so only the larger of the two matters here.
    pub fn set_area(&mut self, width: f64, height: f64) {
        self.area = width.max(height);
        let (w, h) = (self.width, self.height);
        self.width = 0;
        self.resize(w, h);
    }

    /// Device pixels per turtle step.
    pub fn scale(&self) -> f64 {
        self.width.min(self.height) as f64 / self.area
    }

    /// Where a turtle position is, in device pixels: the turtle area is
    /// fitted to the buffer without stretching, and y points down.
    pub fn to_device(&self, p: Point) -> (f64, f64) {
        let scale = self.scale();
        (self.width as f64 / 2.0 + p.x * scale, self.height as f64 / 2.0 - p.y * scale)
    }

    /// Makes the buffer `width` × `height` device pixels, re-rendering
    /// everything since the last `CLEAN` (ADR 0003). Does nothing if it's
    /// already that size. Returns how long the re-render took, if it did one.
    pub fn resize(&mut self, width: u32, height: u32) -> Option<Duration> {
        if (self.width, self.height) == (width, height) {
            return None;
        }
        assert!(width <= u16::MAX as u32 && height <= u16::MAX as u32, "vello_cpu buffers are at most 65,535 pixels wide");
        self.width = width;
        self.height = height;
        self.data = vec![0; width as usize * height as usize * 4];
        self.context = RenderContext::new_with(width as u16, height as u16, settings(self.threads));
        self.columns = width.div_ceil(TILE);
        self.dirty = vec![true; (self.columns * height.div_ceil(TILE)) as usize];
        self.fill_background();
        let started = Instant::now();
        let display = std::mem::take(&mut self.display);
        self.draw(&display);
        self.display = display;
        self.pending.clear();
        Some(started.elapsed())
    }

    fn fill_background(&mut self) {
        for pixel in self.data.as_chunks_mut::<4>().0 {
            *pixel = BACKGROUND;
        }
        self.dirty.iter_mut().for_each(|d| *d = true);
    }

    /// Draws everything pushed since the last call.
    pub fn draw_pending(&mut self) {
        let pending = std::mem::take(&mut self.pending);
        self.draw(&pending);
    }

    fn draw(&mut self, segments: &[Segment]) {
        if segments.is_empty() || self.width == 0 || self.height == 0 {
            return;
        }
        let started = Instant::now();
        let scale = self.scale();
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
            let width = pen_width(segment.pen, scale);
            self.context.set_stroke(Stroke::new(width).with_caps(Cap::Round));
            self.context.stroke_path(&path);
            self.mark_dirty(fx.min(tx), fy.min(ty), fx.max(tx), fy.max(ty), width / 2.0 + 2.0);
        }
        self.context.flush();
        let target = vello_cpu::PixmapMut::new(self.width as u16, self.height as u16, &mut self.data).expect("size matches");
        let settings = RasterizerSettings { target_init: TargetInit::SrcOver, ..RasterizerSettings::default() };
        self.context.render_with(target, &mut self.resources, settings);
        self.last_draw = started.elapsed();
        let each = self.last_draw / segments.len() as u32;
        self.per_segment = (self.per_segment * 3 + each) / 4;
    }

    fn mark_dirty(&mut self, left: f64, top: f64, right: f64, bottom: f64, margin: f64) {
        if self.columns == 0 {
            return;
        }
        let rows = self.dirty.len() as u32 / self.columns;
        // Off-buffer coordinates clamp to the edge tiles.
        let column = |x: f64| ((x.max(0.0) as u32).min(self.width - 1) / TILE).min(self.columns - 1);
        let row = |y: f64| ((y.max(0.0) as u32).min(self.height - 1) / TILE).min(rows - 1);
        if right + margin < 0.0 || bottom + margin < 0.0 || left - margin >= self.width as f64 || top - margin >= self.height as f64 {
            return;
        }
        for ty in row(top - margin)..=row(bottom + margin) {
            for tx in column(left - margin)..=column(right + margin) {
                self.dirty[(ty * self.columns + tx) as usize] = true;
            }
        }
    }

    /// The tiles that changed since the last call, which the GUI uploads.
    pub fn take_dirty_tiles(&mut self) -> Vec<Tile> {
        let mut tiles = Vec::new();
        for index in 0..self.dirty.len() {
            if std::mem::take(&mut self.dirty[index]) {
                tiles.push(self.tile(index));
            }
        }
        tiles
    }

    /// Every tile, for a fresh upload.
    pub fn tiles(&self) -> impl Iterator<Item = Tile> + '_ {
        (0..self.dirty.len()).map(|index| self.tile(index))
    }

    fn tile(&self, index: usize) -> Tile {
        let (x, y) = (index as u32 % self.columns * TILE, index as u32 / self.columns * TILE);
        Tile { index, x, y, width: TILE.min(self.width - x), height: TILE.min(self.height - y) }
    }

    /// A tile's pixels, in BGRA order, as gpui's `RenderImage` wants them.
    pub fn tile_bgra(&self, tile: Tile) -> Vec<u8> {
        let stride = self.width as usize * 4;
        let mut bytes = Vec::with_capacity(tile.width as usize * tile.height as usize * 4);
        for y in tile.y..tile.y + tile.height {
            let start = y as usize * stride + tile.x as usize * 4;
            bytes.extend_from_slice(&self.data[start..start + tile.width as usize * 4]);
        }
        bytes
    }

    /// One pixel, as RGBA.
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let i = (y as usize * self.width as usize + x as usize) * 4;
        let [b, g, r, a] = self.data[i..i + 4].try_into().expect("4 bytes");
        [r, g, b, a]
    }

    /// The whole buffer as RGBA, for saving.
    pub fn rgba(&self) -> Vec<u8> {
        let mut out = self.data.clone();
        for pixel in out.as_chunks_mut::<4>().0 {
            pixel.swap(0, 2);
        }
        out
    }

    /// The drawing as a PNG file.
    pub fn png(&self) -> Vec<u8> {
        let image = image::RgbaImage::from_raw(self.width, self.height, self.rgba()).expect("buffer matches size");
        let mut out = Cursor::new(Vec::new());
        image.write_to(&mut out, image::ImageFormat::Png).expect("PNG encoding to memory can't fail");
        out.into_inner()
    }
}

/// A pen's width in device pixels: never thinner than one pixel.
pub fn pen_width(pen: Pen, scale: f64) -> f64 {
    (pen.size * scale).max(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WHITE: Pen = Pen { down: true, color: [255, 255, 255, 255], size: 1.0 };
    const RED: Pen = Pen { down: true, color: [255, 0, 0, 255], size: 10.0 };

    fn p(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    #[test]
    fn draws_a_line_in_the_pen_colour() {
        let mut d = Drawing::new();
        d.resize(1000, 1000);
        d.push(&[Segment { from: p(-200.0, 0.0), to: p(200.0, 0.0), pen: RED }]);
        d.draw_pending();
        assert_eq!(d.pixel(500, 500), [255, 0, 0, 255], "on the line");
        assert_eq!(d.pixel(500, 520), BACKGROUND, "off the line");
        assert_eq!(d.pixel(250, 500), BACKGROUND, "past its end");
    }

    #[test]
    fn y_points_up_and_the_area_is_fitted_without_stretching() {
        let mut d = Drawing::new();
        d.resize(400, 200);
        assert_eq!(d.scale(), 0.2);
        assert_eq!(d.to_device(p(0.0, 0.0)), (200.0, 100.0));
        assert_eq!(d.to_device(p(500.0, 500.0)), (300.0, 0.0));
    }

    #[test]
    fn resizing_re_renders_the_display_list() {
        let mut d = Drawing::new();
        d.resize(500, 500);
        d.push(&[Segment { from: p(0.0, -300.0), to: p(0.0, 300.0), pen: RED }]);
        d.draw_pending();
        assert_eq!(d.pixel(250, 250), [255, 0, 0, 255]);
        let took = d.resize(1000, 1000);
        assert!(took.is_some());
        assert_eq!(d.pixel(500, 500), [255, 0, 0, 255], "drawn again at the new size");
        // The line ends at y 300, which is device row 200 at this size.
        assert_eq!(d.pixel(500, 210), [255, 0, 0, 255]);
        assert_eq!(d.pixel(500, 180), BACKGROUND, "and no further");
        assert_eq!(d.resize(1000, 1000), None, "same size: nothing to do");
    }

    #[test]
    fn clean_clears_the_drawing() {
        let mut d = Drawing::new();
        d.resize(100, 100);
        d.push(&[Segment { from: p(-400.0, 0.0), to: p(400.0, 0.0), pen: RED }]);
        d.draw_pending();
        d.clean();
        assert_eq!(d.segments(), 0);
        assert_eq!(d.pixel(50, 50), BACKGROUND);
    }

    #[test]
    fn only_touched_tiles_are_dirty() {
        let mut d = Drawing::new();
        d.resize(1000, 600);
        let all = d.take_dirty_tiles();
        assert_eq!(all.len(), 4 * 3, "everything is dirty at first");
        assert_eq!(all.last().unwrap(), &Tile { index: 11, x: 768, y: 512, width: 232, height: 88 });
        assert!(d.take_dirty_tiles().is_empty());
        // A short line near the centre, inside one tile.
        d.push(&[Segment { from: p(-10.0, 50.0), to: p(10.0, 50.0), pen: WHITE }]);
        d.draw_pending();
        let dirty: Vec<_> = d.take_dirty_tiles().iter().map(|t| t.index).collect();
        assert_eq!(dirty, [5]);
    }

    #[test]
    fn tiles_and_pngs_have_the_right_bytes() {
        let mut d = Drawing::new();
        d.resize(300, 300);
        d.push(&[Segment { from: p(-500.0, 0.0), to: p(500.0, 0.0), pen: RED }]);
        d.draw_pending();
        let tile = d.tiles().next().unwrap();
        let bgra = d.tile_bgra(tile);
        assert_eq!(bgra.len(), 256 * 256 * 4);
        // Row 150, column 10: the red line, stored as BGRA.
        let i = (150 * 256 + 10) * 4;
        assert_eq!(&bgra[i..i + 4], &[0, 0, 255, 255]);
        let png = d.png();
        let image = image::load_from_memory(&png).unwrap().to_rgba8();
        assert_eq!(image.dimensions(), (300, 300));
        assert_eq!(image.get_pixel(10, 150).0, [255, 0, 0, 255]);
    }
}
