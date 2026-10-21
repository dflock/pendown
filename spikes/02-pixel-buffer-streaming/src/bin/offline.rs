//! The measurements that don't need a window: drawing and conversion costs,
//! flood fill at anti-aliased edges (question 5), and blending with an RGBA
//! pen colour (question 6). Writes `fill-edges.png`.
//!
//! Run with `cargo run --release --bin offline --no-default-features`.

use std::time::{Duration, Instant};

use spike_02_pixel_buffer_streaming::tiny_skia::{
    Color, LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform,
};
use spike_02_pixel_buffer_streaming::*;

fn time<T>(runs: usize, mut f: impl FnMut() -> T) -> Samples {
    let mut samples = Samples::default();
    for _ in 0..runs {
        let started = Instant::now();
        std::hint::black_box(f());
        samples.push(started.elapsed());
    }
    samples
}

fn per_item(took: Duration, count: usize) -> String {
    format!("{:.1} ms, {:.2} µs each", took.as_secs_f64() * 1000.0, took.as_secs_f64() * 1e6 / count as f64)
}

fn segments(count: usize) -> Vec<Segment> {
    let mut turtle = Turtle::new(42);
    (0..count).map(|_| turtle.step()).collect()
}

fn drawing() {
    println!("## Drawing 20,000 segments (random walk, pen width 1.5 turtle steps)");
    let list = segments(20_000);
    for (width, height) in [(1280, 800), (2560, 1600)] {
        let mut pixmap = Pixmap::new(width, height).expect("size");
        let view = TurtleView::fit(width, height);
        println!("{width} × {height}, scale {:.2}:", view.scale);
        println!("  one path per segment, round caps: {}", per_item(render_all(&mut pixmap, &list), list.len()));

        clear(&mut pixmap);
        let started = Instant::now();
        for segment in &list {
            let from = view.map(segment.from);
            let to = view.map(segment.to);
            let mut path = PathBuilder::new();
            path.move_to(from.0, from.1);
            path.line_to(to.0, to.1);
            let stroke = Stroke { width: (segment.width * view.scale).max(1.0), line_cap: LineCap::Butt, ..Stroke::default() };
            if let Some(path) = path.finish() {
                pixmap.stroke_path(&path, &paint_for(segment.color), &stroke, Transform::identity(), None);
            }
        }
        println!("  one path per segment, butt caps: {}", per_item(started.elapsed(), list.len()));

        clear(&mut pixmap);
        let started = Instant::now();
        draw_runs(&mut pixmap, &view, &list);
        println!("  one path per run of the same colour: {}", per_item(started.elapsed(), list.len()));

        let thin: Vec<Segment> = list.iter().map(|s| Segment { width: 0.1, ..*s }).collect();
        println!("  one path per segment, 1 px wide: {}", per_item(render_all(&mut pixmap, &thin), list.len()));

        let opaque: Vec<Segment> = list.iter().map(|s| Segment { color: [s.color[0], s.color[1], s.color[2], 255], ..*s }).collect();
        println!("  one path per segment, all opaque: {}", per_item(render_all(&mut pixmap, &opaque), list.len()));
    }
    println!();
}

/// Re-renders the display list in horizontal bands, one thread per band.
/// Each thread draws only the segments that touch its band.
fn render_banded(pixmap: &mut Pixmap, segments: &[Segment], bands: usize) -> Duration {
    let started = Instant::now();
    let (width, height) = (pixmap.width(), pixmap.height());
    let view = TurtleView::fit(width, height);
    let band_height = height.div_ceil(bands as u32);
    let stride = width as usize * 4;
    let rows: Vec<&mut [u8]> = pixmap.data_mut().chunks_mut(stride * band_height as usize).collect();
    std::thread::scope(|scope| {
        for (index, band) in rows.into_iter().enumerate() {
            scope.spawn(move || {
                let top = index as u32 * band_height;
                let rows_here = (band.len() / stride) as u32;
                let mut part = Pixmap::new(width, rows_here).expect("size");
                clear(&mut part);
                let band_view = TurtleView { scale: view.scale, centre: (view.centre.0, view.centre.1 - top as f32) };
                let margin = 4.0;
                for segment in segments {
                    let (a, b) = (band_view.map(segment.from), band_view.map(segment.to));
                    if a.1.max(b.1) < -margin || a.1.min(b.1) > rows_here as f32 + margin {
                        continue;
                    }
                    draw_segment(&mut part, &band_view, segment);
                }
                band.copy_from_slice(part.data());
            });
        }
    });
    started.elapsed()
}

fn rerendering() {
    println!("## Re-rendering a 60,000-segment display list at 2560 × 1600, as on a resize");
    let list = segments(60_000);
    let mut pixmap = Pixmap::new(2560, 1600).expect("size");
    println!("  tiny-skia, one thread: {}", per_item(render_all(&mut pixmap, &list), list.len()));
    pixmap.save_png("tiny-skia.png").expect("write tiny-skia.png");
    let reference = pixmap.data().to_vec();
    for bands in [4, 8] {
        let took = render_banded(&mut pixmap, &list, bands);
        let differing = pixmap.data().chunks_exact(4).zip(reference.chunks_exact(4)).filter(|(a, b)| a != b).count();
        println!("  tiny-skia, {bands} bands on {bands} threads: {} ({differing} pixels differ from one thread)", per_item(took, list.len()));
    }
    #[cfg(feature = "vello")]
    vello::rerender(&list);
    println!();
}

#[cfg(feature = "vello")]
mod vello {
    use super::*;
    use spike_02_pixel_buffer_streaming::vello::VelloDrawer;

    pub fn rerender(list: &[Segment]) {
        let mut pixmap = Pixmap::new(2560, 1600).expect("size");
        for threads in [0u16, 7] {
            let mut drawer = VelloDrawer::new(threads);
            println!("  vello_cpu, {threads} worker threads: {}", per_item(drawer.render_all(&mut pixmap, list, false), list.len()));
        }
        pixmap.save_png("vello.png").expect("write vello.png");

        // Frame by frame: 500 new segments onto the existing drawing.
        let mut turtle = Turtle::new(7);
        for threads in [0u16, 7] {
            let mut drawer = VelloDrawer::new(threads);
            let mut samples = Samples::default();
            for _ in 0..60 {
                let batch: Vec<Segment> = (0..500).map(|_| turtle.step()).collect();
                let started = Instant::now();
                drawer.draw(&mut pixmap, &batch, false);
                samples.push(started.elapsed());
            }
            println!("  vello_cpu, 500 segments a frame onto the drawing, {threads} worker threads: {}", samples.summary());
        }
        let view = TurtleView::fit(2560, 1600);
        let mut samples = Samples::default();
        for _ in 0..60 {
            let batch: Vec<Segment> = (0..500).map(|_| turtle.step()).collect();
            let started = Instant::now();
            for segment in &batch {
                draw_segment(&mut pixmap, &view, segment);
            }
            samples.push(started.elapsed());
        }
        println!("  tiny-skia, 500 segments a frame, for comparison: {}", samples.summary());
    }
}

fn conversion() {
    println!("## Converting 2560 × 1600 to BGRA (30 runs)");
    let mut pixmap = Pixmap::new(2560, 1600).expect("size");
    render_all(&mut pixmap, &segments(2000));
    let data = pixmap.data();
    let mut out = vec![];
    println!("  pixel by pixel with extend_from_slice: {}", time(30, || to_bgra_opaque(data, &mut out)).summary());
    println!("  a u32 at a time into a zeroed Vec: {}", time(30, || to_bgra_opaque_fast(data)).summary());
    println!("  demultiply too: {}", time(30, || to_bgra_demultiplied(data, &mut out)).summary());
    println!("  plain copy (to_vec), for drawing with swapped colours: {}", time(30, || data.to_vec()).summary());
    println!();
}

/// A closed shape for the fill tests, drawn as one stroke per side, the way
/// Logo moves draw it.
struct Shape {
    name: &'static str,
    points: Vec<(f32, f32)>,
    width: f32,
    line: [u8; 3],
    background: [u8; 3],
    fill: [u8; 3],
}

fn polygon(sides: usize, radius: impl Fn(usize) -> f32, turn: f32) -> Vec<(f32, f32)> {
    (0..sides)
        .map(|i| {
            let angle = (i as f32 / sides as f32 * 360.0 + turn).to_radians();
            (200.0 + radius(i) * angle.cos(), 200.0 + radius(i) * angle.sin())
        })
        .collect()
}

fn shapes() -> Vec<Shape> {
    let circle = polygon(72, |_| 150.0, 0.0);
    let black = [0, 0, 0];
    let white = [255, 255, 255];
    let yellow = [255, 220, 0];
    vec![
        Shape { name: "circle, 1 px", points: circle.clone(), width: 1.0, line: black, background: white, fill: yellow },
        Shape { name: "circle, 3 px", points: circle.clone(), width: 3.0, line: black, background: white, fill: yellow },
        Shape { name: "star, 1 px", points: polygon(10, |i| if i % 2 == 0 { 170.0 } else { 70.0 }, -90.0), width: 1.0, line: black, background: white, fill: yellow },
        Shape { name: "square at 30°, 0.5 px", points: polygon(4, |_| 150.0, 30.0), width: 0.5, line: black, background: white, fill: yellow },
        Shape { name: "pale grey circle, 1 px", points: circle.clone(), width: 1.0, line: [200, 200, 200], background: white, fill: yellow },
        Shape { name: "red on blue, 1 px", points: circle, width: 1.0, line: [220, 30, 30], background: [30, 60, 200], fill: yellow },
    ]
}

fn draw_shape(shape: &Shape) -> Pixmap {
    let mut pixmap = Pixmap::new(400, 400).expect("size");
    let [r, g, b] = shape.background;
    pixmap.fill(Color::from_rgba8(r, g, b, 255));
    let [r, g, b] = shape.line;
    let paint: Paint = paint_for([r, g, b, 255]);
    let stroke = Stroke { width: shape.width, line_cap: LineCap::Round, ..Stroke::default() };
    for i in 0..shape.points.len() {
        let (a, b) = (shape.points[i], shape.points[(i + 1) % shape.points.len()]);
        let mut path = PathBuilder::new();
        path.move_to(a.0, a.1);
        path.line_to(b.0, b.1);
        if let Some(path) = path.finish() {
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }
    pixmap
}

fn distance_to_edges(points: &[(f32, f32)], p: (f32, f32)) -> f32 {
    let mut best = f32::MAX;
    for i in 0..points.len() {
        let (a, b) = (points[i], points[(i + 1) % points.len()]);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let t = (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
        let (x, y) = (a.0 + t * dx - p.0, a.1 + t * dy - p.1);
        best = best.min((x * x + y * y).sqrt());
    }
    best
}

fn inside(points: &[(f32, f32)], p: (f32, f32)) -> bool {
    let mut inside = false;
    for i in 0..points.len() {
        let (a, b) = (points[i], points[(i + 1) % points.len()]);
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < a.0 + (p.1 - a.1) / (b.1 - a.1) * (b.0 - a.0) {
            inside = !inside;
        }
    }
    inside
}

fn colour_distance(a: [u8; 3], b: [u8; 3]) -> u8 {
    (0..3).map(|c| a[c].abs_diff(b[c])).max().unwrap_or(0)
}

struct FillResult {
    leaks: usize,
    gaps: usize,
    halo: usize,
}

/// Counts what went wrong, using the shape's geometry:
/// - leaks: fill colour well outside the shape;
/// - gaps: no fill colour well inside the shape;
/// - halo: pixels just inside the line that still look like the background.
fn judge(shape: &Shape, pixmap: &Pixmap) -> FillResult {
    let mut result = FillResult { leaks: 0, gaps: 0, halo: 0 };
    let clear_of_line = shape.width / 2.0 + 1.5;
    for y in 0..400 {
        for x in 0..400 {
            let p = (x as f32 + 0.5, y as f32 + 0.5);
            let pixel = pixmap.pixel(x, y).expect("in bounds");
            let colour = [pixel.red(), pixel.green(), pixel.blue()];
            let d = distance_to_edges(&shape.points, p);
            let is_inside = inside(&shape.points, p);
            let to_fill = colour_distance(colour, shape.fill);
            let to_background = colour_distance(colour, shape.background);
            let to_line = colour_distance(colour, shape.line);
            if !is_inside && d > clear_of_line && to_fill < to_background {
                result.leaks += 1;
            }
            if is_inside && d > clear_of_line && to_fill > 32 {
                result.gaps += 1;
            }
            if is_inside && d > shape.width / 2.0 - 0.5 && d <= clear_of_line && to_background < to_fill && to_background < to_line {
                result.halo += 1;
            }
        }
    }
    result
}

/// Copies a square of `source` into `sheet`, magnified.
fn paste_magnified(sheet: &mut Pixmap, source: &Pixmap, from: (u32, u32), size: u32, to: (u32, u32), zoom: u32) {
    let sheet_width = sheet.width();
    let data = sheet.data_mut();
    for y in 0..size * zoom {
        for x in 0..size * zoom {
            let pixel = source.pixel(from.0 + x / zoom, from.1 + y / zoom).expect("crop inside source");
            let index = (((to.1 + y) * sheet_width + to.0 + x) * 4) as usize;
            data[index..index + 4].copy_from_slice(&[pixel.red(), pixel.green(), pixel.blue(), 255]);
        }
    }
}

fn fill_edges() {
    println!("## Flood fill at anti-aliased edges");
    println!("Each shape is drawn one side at a time, then filled from its centre (4-connected).");
    println!("Leaks: fill colour more than w/2 + 1.5 px outside the line. Gaps: no fill more than w/2 + 1.5 px inside.");
    println!("Halo: pixels just inside the line that look more like the background than the fill or the line.");
    println!();
    let tolerances = [0u8, 32, 64, 128, 192, 250];
    let sheet_tolerances = [0u8, 64, 128, 192];
    let (crop, zoom, gap) = (40u32, 6u32, 6u32);
    let cell = crop * zoom + gap;
    let shapes = shapes();
    let mut sheet = Pixmap::new(cell * sheet_tolerances.len() as u32 * 2, cell * shapes.len() as u32).expect("size");
    sheet.fill(Color::from_rgba8(128, 128, 128, 255));

    for (row, shape) in shapes.iter().enumerate() {
        println!("{}:", shape.name);
        println!("  | tolerance | edges | leaks | gaps | halo | filled |");
        println!("  |---|---|---|---|---|---|");
        for blend_edges in [false, true] {
            for tolerance in tolerances {
                let mut pixmap = draw_shape(shape);
                let filled = flood_fill(&mut pixmap, (200, 200), shape.fill, FillOptions { tolerance, blend_edges });
                let result = judge(shape, &pixmap);
                println!(
                    "  | {tolerance} | {} | {} | {} | {} | {filled} |",
                    if blend_edges { "blended" } else { "plain" },
                    result.leaks,
                    result.gaps,
                    result.halo
                );
                if let Some(column) = sheet_tolerances.iter().position(|t| *t == tolerance) {
                    let column = column as u32 + if blend_edges { sheet_tolerances.len() as u32 } else { 0 };
                    // A piece of the boundary: the top of the shape, where the
                    // side runs nearly horizontal and anti-aliasing is strongest.
                    let top = shape.points.iter().map(|p| p.1).fold(f32::MAX, f32::min);
                    let from = (180, (top as u32).saturating_sub(crop / 2));
                    paste_magnified(&mut sheet, &pixmap, from, crop, (column * cell, row as u32 * cell), zoom);
                }
            }
        }
        println!();
    }
    sheet.save_png("fill-edges.png").expect("write fill-edges.png");
    println!("Wrote fill-edges.png: one row per shape; columns are tolerance {sheet_tolerances:?} plain, then the same blended.");
    println!();
}

fn blending() {
    println!("## Blending an RGBA pen colour: red [255 0 0 128] over blue [0 0 255]");
    let mut pixmap = Pixmap::new(440, 160).expect("size");
    pixmap.fill(Color::from_rgba8(0, 0, 255, 255));
    let red = paint_for([255, 0, 0, 128]);
    let stroke = Stroke { width: 20.0, ..Stroke::default() };
    let line = |pixmap: &mut Pixmap, points: &[(f32, f32)]| {
        let mut path = PathBuilder::new();
        path.move_to(points[0].0, points[0].1);
        for p in &points[1..] {
            path.line_to(p.0, p.1);
        }
        if let Some(path) = path.finish() {
            pixmap.stroke_path(&path, &red, &stroke, Transform::identity(), None);
        }
    };
    line(&mut pixmap, &[(40., 40.), (200., 40.)]);
    line(&mut pixmap, &[(240., 40.), (380., 40.)]);
    line(&mut pixmap, &[(380., 40.), (380., 140.)]);
    line(&mut pixmap, &[(40., 110.), (200., 110.), (200., 150.)]);
    let show = |name: &str, x: u32, y: u32| {
        let p = pixmap.pixel(x, y).expect("in bounds");
        println!("  {name}: ({}, {}, {}, {})", p.red(), p.green(), p.blue(), p.alpha());
    };
    println!("  Expected for one layer: (128, 0, 127, 255); for two layers: (192, 0, 63, 255).");
    show("one stroke", 120, 40);
    show("corner where two separate strokes overlap", 375, 40);
    show("corner of one two-segment path", 197, 110);
    println!();
}

fn main() {
    drawing();
    rerendering();
    conversion();
    blending();
    fill_edges();
}
