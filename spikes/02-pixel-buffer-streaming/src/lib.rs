//! Drawing code shared by the window (`main.rs`) and the offline measurements
//! (`bin/offline.rs`): a fast random-walk turtle, the display list, drawing
//! segments with tiny-skia, conversion to gpui's BGRA, and flood fill.

use std::time::{Duration, Instant};

pub use tiny_skia;
use tiny_skia::{Color, IntRect, LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};

/// One finished move, in turtle coordinates: -500..500 on both axes, y up.
#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub from: (f32, f32),
    pub to: (f32, f32),
    /// Straight (not premultiplied) RGBA, as events carry it.
    pub color: [u8; 4],
    /// Pen width in turtle steps.
    pub width: f32,
}

/// Pen colours for the random walk. Two have opacity, for question 6.
pub const PALETTE: [[u8; 4]; 6] = [
    [0, 0, 0, 255],
    [220, 40, 40, 255],
    [30, 120, 220, 255],
    [20, 160, 70, 255],
    [255, 140, 0, 128],
    [140, 60, 200, 96],
];

pub const BACKGROUND: [u8; 4] = [255, 255, 255, 255];

/// A turtle that walks randomly, for a stream of plausible segments.
pub struct Turtle {
    pos: (f32, f32),
    heading: f32,
    rng: u64,
    colour: usize,
    steps: usize,
}

impl Turtle {
    pub fn new(seed: u64) -> Self {
        Self { pos: (0.0, 0.0), heading: 0.0, rng: seed | 1, colour: 0, steps: 0 }
    }

    fn random(&mut self) -> f32 {
        // xorshift64
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn step(&mut self) -> Segment {
        self.steps += 1;
        if self.steps % 200 == 0 {
            self.colour = (self.colour + 1) % PALETTE.len();
        }
        self.heading += (self.random() - 0.5) * 120.0;
        let length = 5.0 + self.random() * 35.0;
        let (sin, cos) = self.heading.to_radians().sin_cos();
        let mut to = (self.pos.0 + length * sin, self.pos.1 + length * cos);
        // Turn round at the edge of the turtle area.
        if to.0.abs() > 480.0 || to.1.abs() > 480.0 {
            self.heading += 180.0;
            to = (to.0.clamp(-480.0, 480.0), to.1.clamp(-480.0, 480.0));
        }
        let segment = Segment { from: self.pos, to, color: PALETTE[self.colour], width: 1.5 };
        self.pos = to;
        segment
    }
}

/// Maps turtle coordinates to pixmap pixels: the 1000 × 1000 turtle area,
/// scaled to fit without stretching, and centred (ADR 0003).
#[derive(Clone, Copy, Debug)]
pub struct TurtleView {
    pub scale: f32,
    pub centre: (f32, f32),
}

impl TurtleView {
    pub fn fit(width: u32, height: u32) -> Self {
        Self {
            scale: width.min(height) as f32 / 1000.0,
            centre: (width as f32 / 2.0, height as f32 / 2.0),
        }
    }

    pub fn map(&self, (x, y): (f32, f32)) -> (f32, f32) {
        (self.centre.0 + x * self.scale, self.centre.1 - y * self.scale)
    }
}

pub fn paint_for(color: [u8; 4]) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(color[0], color[1], color[2], color[3]);
    paint.anti_alias = true;
    paint
}

pub fn clear(pixmap: &mut Pixmap) {
    let [r, g, b, a] = BACKGROUND;
    pixmap.fill(Color::from_rgba8(r, g, b, a));
}

/// Draws one segment and returns the pixel rectangle it touched.
pub fn draw_segment(pixmap: &mut Pixmap, view: &TurtleView, segment: &Segment) -> Option<IntRect> {
    let from = view.map(segment.from);
    let to = view.map(segment.to);
    let width = (segment.width * view.scale).max(1.0);
    let mut path = PathBuilder::new();
    path.move_to(from.0, from.1);
    path.line_to(to.0, to.1);
    let path = path.finish()?;
    let stroke = Stroke { width, line_cap: LineCap::Round, ..Stroke::default() };
    pixmap.stroke_path(&path, &paint_for(segment.color), &stroke, Transform::identity(), None);
    let margin = width / 2.0 + 2.0;
    let left = (from.0.min(to.0) - margin).floor().max(0.0) as i32;
    let top = (from.1.min(to.1) - margin).floor().max(0.0) as i32;
    let right = ((from.0.max(to.0) + margin).ceil() as i32).min(pixmap.width() as i32);
    let bottom = ((from.1.max(to.1) + margin).ceil() as i32).min(pixmap.height() as i32);
    IntRect::from_ltrb(left, top, right, bottom)
}

/// Re-renders the whole drawing from the display list, as on a resize.
pub fn render_all(pixmap: &mut Pixmap, segments: &[Segment]) -> Duration {
    let started = Instant::now();
    let view = TurtleView::fit(pixmap.width(), pixmap.height());
    clear(pixmap);
    for segment in segments {
        draw_segment(pixmap, &view, segment);
    }
    started.elapsed()
}

/// tiny-skia's premultiplied RGBA to gpui's straight BGRA, for a pixmap with
/// an opaque background. Every pixel is then opaque, so premultiplied and
/// straight are the same and only red and blue need swapping.
pub fn to_bgra_opaque(rgba: &[u8], bgra: &mut Vec<u8>) {
    bgra.clear();
    bgra.reserve(rgba.len());
    for pixel in rgba.chunks_exact(4) {
        bgra.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
    }
}

/// The same conversion, a whole pixel at a time, so the compiler can
/// vectorise it.
pub fn to_bgra_opaque_fast(rgba: &[u8]) -> Vec<u8> {
    let mut bgra = vec![0; rgba.len()];
    for (out, pixel) in bgra.chunks_exact_mut(4).zip(rgba.chunks_exact(4)) {
        let value = u32::from_le_bytes([pixel[0], pixel[1], pixel[2], pixel[3]]);
        let swapped = (value & 0xff00ff00) | ((value >> 16) & 0xff) | ((value & 0xff) << 16);
        out.copy_from_slice(&swapped.to_le_bytes());
    }
    bgra
}

/// Swaps red and blue in a colour. Drawing with swapped colours leaves the
/// pixmap's bytes in BGRA order, so no conversion is needed afterwards:
/// blending treats every channel the same way.
pub fn swap_red_blue([r, g, b, a]: [u8; 4]) -> [u8; 4] {
    [b, g, r, a]
}

/// Draws runs of connected segments of the same colour and width as one path
/// each, instead of one path per segment.
pub fn draw_runs(pixmap: &mut Pixmap, view: &TurtleView, segments: &[Segment]) {
    let mut start = 0;
    while start < segments.len() {
        let first = segments[start];
        let mut path = PathBuilder::new();
        let from = view.map(first.from);
        path.move_to(from.0, from.1);
        let mut end = start;
        while end < segments.len()
            && segments[end].color == first.color
            && segments[end].width == first.width
            && (end == start || segments[end].from == segments[end - 1].to)
        {
            let to = view.map(segments[end].to);
            path.line_to(to.0, to.1);
            end += 1;
        }
        if let Some(path) = path.finish() {
            let width = (first.width * view.scale).max(1.0);
            let stroke = Stroke { width, line_cap: LineCap::Round, line_join: tiny_skia::LineJoin::Round, ..Stroke::default() };
            pixmap.stroke_path(&path, &paint_for(first.color), &stroke, Transform::identity(), None);
        }
        start = end;
    }
}

/// The general conversion, for pixmaps that may have transparent pixels:
/// demultiply, then swap red and blue.
pub fn to_bgra_demultiplied(rgba: &[u8], bgra: &mut Vec<u8>) {
    bgra.clear();
    bgra.reserve(rgba.len());
    for pixel in rgba.chunks_exact(4) {
        let alpha = pixel[3];
        let demultiply = |channel: u8| {
            if alpha == 0 { 0 } else { ((channel as u32 * 255 + alpha as u32 / 2) / alpha as u32) as u8 }
        };
        bgra.extend_from_slice(&[demultiply(pixel[2]), demultiply(pixel[1]), demultiply(pixel[0]), alpha]);
    }
}

/// Copies one rectangle of the pixmap out as BGRA, for a tile.
pub fn tile_to_bgra(pixmap: &Pixmap, rect: IntRect, bgra: &mut Vec<u8>) {
    bgra.clear();
    bgra.reserve(rect.width() as usize * rect.height() as usize * 4);
    let stride = pixmap.width() as usize * 4;
    let data = pixmap.data();
    for y in rect.top()..rect.bottom() {
        let start = y as usize * stride + rect.left() as usize * 4;
        let row = &data[start..start + rect.width() as usize * 4];
        for pixel in row.chunks_exact(4) {
            bgra.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
        }
    }
}

/// How flood fill treats the anti-aliased pixels at the edge of a line.
#[derive(Clone, Copy, Debug)]
pub struct FillOptions {
    /// A pixel is filled when no channel differs from the seed pixel by more
    /// than this.
    pub tolerance: u8,
    /// Also recolour the unfilled pixels next to the filled area, swapping
    /// their share of the seed colour for the fill colour.
    pub blend_edges: bool,
}

fn rgb(pixmap: &Pixmap, index: usize) -> [f32; 3] {
    let data = pixmap.data();
    [data[index * 4] as f32, data[index * 4 + 1] as f32, data[index * 4 + 2] as f32]
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).abs().max((a[1] - b[1]).abs()).max((a[2] - b[2]).abs())
}

/// A 4-connected scanline flood fill with an opaque fill colour, on an opaque
/// pixmap. Returns how many pixels it filled.
pub fn flood_fill(pixmap: &mut Pixmap, seed: (u32, u32), fill: [u8; 3], options: FillOptions) -> usize {
    let (width, height) = (pixmap.width() as usize, pixmap.height() as usize);
    let seed_index = seed.1 as usize * width + seed.0 as usize;
    let seed_colour = rgb(pixmap, seed_index);
    let tolerance = options.tolerance as f32;
    let fillable = |pixmap: &Pixmap, index: usize| distance(rgb(pixmap, index), seed_colour) <= tolerance;

    let mut filled = vec![false; width * height];
    let mut stack = vec![(seed.0 as usize, seed.1 as usize)];
    let mut count = 0;
    while let Some((x, y)) = stack.pop() {
        let row = y * width;
        if filled[row + x] || !fillable(pixmap, row + x) {
            continue;
        }
        let mut left = x;
        while left > 0 && !filled[row + left - 1] && fillable(pixmap, row + left - 1) {
            left -= 1;
        }
        let mut right = x;
        while right + 1 < width && !filled[row + right + 1] && fillable(pixmap, row + right + 1) {
            right += 1;
        }
        for xx in left..=right {
            filled[row + xx] = true;
            count += 1;
            if y > 0 {
                stack.push((xx, y - 1));
            }
            if y + 1 < height {
                stack.push((xx, y + 1));
            }
        }
    }

    let fill_colour = [fill[0] as f32, fill[1] as f32, fill[2] as f32];
    let mut edges = vec![];
    if options.blend_edges {
        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;
                if filled[index] {
                    continue;
                }
                let next_to_fill = (x > 0 && filled[index - 1])
                    || (x + 1 < width && filled[index + 1])
                    || (y > 0 && filled[index - width])
                    || (y + 1 < height && filled[index + width]);
                if !next_to_fill {
                    continue;
                }
                // The line's colour is unknown, so take the unfilled
                // neighbour that is least like the seed colour as the line,
                // and estimate how much of the seed colour this pixel holds.
                let pixel = rgb(pixmap, index);
                let mut line = pixel;
                for (dx, dy) in [(-1i32, -1i32), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
                    let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                    if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                        continue;
                    }
                    let neighbour = ny as usize * width + nx as usize;
                    if !filled[neighbour] {
                        let colour = rgb(pixmap, neighbour);
                        if distance(colour, seed_colour) > distance(line, seed_colour) {
                            line = colour;
                        }
                    }
                }
                let axis = [seed_colour[0] - line[0], seed_colour[1] - line[1], seed_colour[2] - line[2]];
                let length = axis.iter().map(|c| c * c).sum::<f32>();
                if length < 1.0 {
                    continue;
                }
                let share = ((0..3).map(|c| (pixel[c] - line[c]) * axis[c]).sum::<f32>() / length).clamp(0.0, 1.0);
                let blended: [f32; 3] = std::array::from_fn(|c| pixel[c] + share * (fill_colour[c] - seed_colour[c]));
                edges.push((index, blended));
            }
        }
    }

    let data = pixmap.data_mut();
    for (index, is_filled) in filled.iter().enumerate() {
        if *is_filled {
            data[index * 4..index * 4 + 3].copy_from_slice(&fill);
        }
    }
    for (index, colour) in edges {
        for c in 0..3 {
            data[index * 4 + c] = colour[c].round().clamp(0.0, 255.0) as u8;
        }
    }
    count
}

/// The same drawing with vello_cpu, onto the pixels already in a tiny-skia
/// pixmap. Both use premultiplied RGBA, so the pixmap stays the one buffer
/// that flood fill and PNG export work on.
#[cfg(feature = "vello")]
pub mod vello {
    use super::*;
    use vello_cpu::kurbo::{BezPath, Cap, Stroke as KurboStroke};
    use vello_cpu::{RasterizerSettings, RenderContext, RenderSettings, Resources, TargetInit};

    pub struct VelloDrawer {
        context: RenderContext,
        resources: Resources,
        threads: u16,
    }

    impl VelloDrawer {
        /// `threads` is the number of worker threads; 0 draws on the calling thread.
        pub fn new(threads: u16) -> Self {
            Self { context: RenderContext::new_with(1, 1, RenderSettings { num_threads: threads, ..RenderSettings::default() }), resources: Resources::new(), threads }
        }

        pub fn draw(&mut self, pixmap: &mut Pixmap, segments: &[Segment], swap: bool) {
            let (width, height) = (pixmap.width() as u16, pixmap.height() as u16);
            if (self.context.width(), self.context.height()) != (width, height) {
                self.context = RenderContext::new_with(width, height, RenderSettings { num_threads: self.threads, ..RenderSettings::default() });
            } else {
                self.context.reset();
            }
            let view = TurtleView::fit(pixmap.width(), pixmap.height());
            for segment in segments {
                let from = view.map(segment.from);
                let to = view.map(segment.to);
                let mut path = BezPath::new();
                path.move_to((from.0 as f64, from.1 as f64));
                path.line_to((to.0 as f64, to.1 as f64));
                let [r, g, b, a] = if swap { swap_red_blue(segment.color) } else { segment.color };
                self.context.set_paint(vello_cpu::color::AlphaColor::from_rgba8(r, g, b, a));
                self.context.set_stroke(KurboStroke::new(((segment.width * view.scale).max(1.0)) as f64).with_caps(Cap::Round));
                self.context.stroke_path(&path);
            }
            let target = vello_cpu::PixmapMut::new(width, height, pixmap.data_mut()).expect("size matches");
            let settings = RasterizerSettings { target_init: TargetInit::SrcOver, ..RasterizerSettings::default() };
            self.context.flush();
            self.context.render_with(target, &mut self.resources, settings);
        }

        pub fn render_all(&mut self, pixmap: &mut Pixmap, segments: &[Segment], swap: bool) -> Duration {
            let started = Instant::now();
            clear(pixmap);
            self.draw(pixmap, segments, swap);
            started.elapsed()
        }
    }
}

#[derive(Default)]
pub struct Samples(pub Vec<Duration>);

impl Samples {
    pub fn push(&mut self, duration: Duration) {
        self.0.push(duration);
    }

    pub fn summary(&self) -> String {
        if self.0.is_empty() {
            return "no samples".into();
        }
        let mut sorted = self.0.clone();
        sorted.sort();
        let ms = |d: Duration| d.as_secs_f64() * 1000.0;
        let mean = sorted.iter().map(|d| ms(*d)).sum::<f64>() / sorted.len() as f64;
        format!(
            "n={} mean={:.2}ms p50={:.2}ms p95={:.2}ms max={:.2}ms",
            sorted.len(),
            mean,
            ms(sorted[sorted.len() / 2]),
            ms(sorted[sorted.len() * 95 / 100]),
            ms(sorted[sorted.len() - 1])
        )
    }
}

/// Process memory: resident set size, and GPU buffer memory from the DRM
/// driver's fdinfo, in MiB. On an integrated GPU the textures live in system
/// memory, so both matter.
pub fn memory_mib() -> (f64, f64) {
    let page = 4096.0;
    let rss = std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|s| s.split_whitespace().nth(1)?.parse::<f64>().ok())
        .map_or(0.0, |pages| pages * page / (1024.0 * 1024.0));
    let mut gpu_kib = 0.0;
    // Several file descriptors can share one DRM client, so count each once.
    let mut clients = std::collections::HashSet::new();
    if let Ok(entries) = std::fs::read_dir("/proc/self/fdinfo") {
        for entry in entries.flatten() {
            let Ok(info) = std::fs::read_to_string(entry.path()) else { continue };
            let client = info.lines().find_map(|line| line.strip_prefix("drm-client-id:"));
            let Some(client) = client else { continue };
            if !clients.insert(client.trim().to_string()) {
                continue;
            }
            for line in info.lines() {
                // e.g. "drm-total-system0:	123456 KiB"
                if let Some(rest) = line.strip_prefix("drm-total-") {
                    let mut parts = rest.split_whitespace().skip(1);
                    let value = parts.next().and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
                    gpu_kib += match parts.next() {
                        Some("MiB") => value * 1024.0,
                        Some("KiB") => value,
                        _ => value / 1024.0,
                    };
                }
            }
        }
    }
    (rss, gpu_kib / 1024.0)
}
