//! The canvas (ADR 0010): `pendown-render`'s pixel buffer shown as tiles,
//! with the move in progress painted as a path each frame, and the turtle as
//! two SVGs, which gpui renders once and keeps on the GPU. With Show
//! thinking on, a turn's arc and a move's ruler go under the turtle.

use std::{rc::Rc, sync::Arc, time::Duration};

use gpui_kit::{component::ActiveTheme as _, *};
use pendown::{
    animator::{Thinking, Thought},
    metrics::WindowMetrics,
};
use pendown_logo::{Pen, Point};
use pendown_render::{Drawing, pen_width};

pub struct Canvas {
    pub drawing: Drawing,
    tiles: Vec<Option<Arc<RenderImage>>>,
    /// How long the last re-render after a resize took, and of how much.
    pub rerender: Option<(usize, Duration)>,
    metrics: Rc<WindowMetrics>,
}

/// The turtle's size in logical pixels, whatever the zoom: the square its
/// pictures fill, centred on its position.
pub const TURTLE_SIZE: f32 = 32.0;

/// The turtle, facing up: its body, then the top of its shell, painted over
/// it in a lighter colour. gpui paints an SVG in one colour.
const TURTLE: [(&str, &[u8], u32); 2] = [
    (
        "pendown/turtle-body.svg",
        include_bytes!("../images/turtle-body.svg"),
        0x2e7d32,
    ),
    (
        "pendown/turtle-shell.svg",
        include_bytes!("../images/turtle-shell.svg"),
        0x8bc34a,
    ),
];

/// What changes each frame, painted on top of the buffer.
#[derive(Clone)]
pub struct Overlay {
    /// What's drawn so far of the move, arc or shape in progress.
    pub partial: Option<(Vec<Point>, Pen)>,
    /// The turtle's position and heading, if it's shown.
    pub turtle: Option<(Point, f64)>,
    /// What Kame is thinking about, with Show thinking on.
    pub thinking: Thinking,
}

/// The arc's radius, and how far the ruler is from the line, in logical
/// pixels, so they read the same at any zoom.
const ARC_RADIUS: f32 = 40.0;
const RULER_GAP: f32 = 10.0;

/// The thinking colours: pale yellow on a dark background, dark amber on a
/// light one.
const THINKING_ON_DARK: u32 = 0xfde68a;
const THINKING_ON_LIGHT: u32 = 0xb45309;

/// A point `length` logical pixels from `at`, towards `heading`, in degrees
/// clockwise from up.
fn towards(at: gpui_kit::Point<Pixels>, heading: f64, length: f32) -> gpui_kit::Point<Pixels> {
    let radians = heading.to_radians();
    point(
        at.x + px(radians.sin() as f32 * length),
        at.y - px(radians.cos() as f32 * length),
    )
}

impl Canvas {
    pub fn new(metrics: Rc<WindowMetrics>) -> Canvas {
        Canvas {
            drawing: Drawing::new(),
            tiles: Vec::new(),
            rerender: None,
            metrics,
        }
    }

    pub fn paint(
        &mut self,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        overlay: Overlay,
        cx: &mut App,
    ) {
        let metrics = self.metrics.clone();
        metrics
            .canvas
            .time(|| self.paint_timed(bounds, window, overlay, cx));
    }

    fn paint_timed(
        &mut self,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        overlay: Overlay,
        cx: &mut App,
    ) {
        let per_device = window.scale_factor();
        let width = (bounds.size.width.as_f32() * per_device).round() as u32;
        let height = (bounds.size.height.as_f32() * per_device).round() as u32;
        if width == 0 || height == 0 {
            return;
        }
        if let Some(took) = self.drawing.resize(width, height) {
            for tile in self.tiles.drain(..).flatten() {
                window.drop_image(tile).ok();
            }
            self.tiles = vec![None; self.drawing.tiles().count()];
            if self.drawing.segments() > 0 {
                self.rerender = Some((self.drawing.segments(), took));
            }
        }
        self.drawing.draw_pending();
        let uploading = self.metrics.upload.start();
        for tile in self.drawing.take_dirty_tiles() {
            let bytes = self.drawing.tile_bgra(tile);
            let buffer = image::RgbaImage::from_raw(tile.width, tile.height, bytes)
                .expect("buffer matches size");
            let image = Arc::new(RenderImage::new([image::Frame::new(buffer)]));
            // The old tile isn't painted this frame, so it can go now. An
            // image dropped in the frame that paints it never shows (spike 02).
            if let Some(old) = self.tiles[tile.index].replace(image) {
                window.drop_image(old).ok();
            }
        }
        self.metrics.upload.stop(uploading);

        let logical = |device: f64| px(device as f32 / per_device);
        for (tile, image) in self.drawing.tiles().zip(&self.tiles) {
            let Some(image) = image else { continue };
            let tile_bounds = Bounds::new(
                point(
                    bounds.origin.x + logical(tile.x as f64),
                    bounds.origin.y + logical(tile.y as f64),
                ),
                size(logical(tile.width as f64), logical(tile.height as f64)),
            );
            window
                .paint_image(
                    tile_bounds,
                    tile_bounds,
                    Corners::default(),
                    image.clone(),
                    0,
                    false,
                )
                .ok();
        }
        self.shade_outside(bounds, window, per_device);

        let to_window = |p: Point| {
            let (x, y) = self.drawing.to_device(p);
            point(bounds.origin.x + logical(x), bounds.origin.y + logical(y))
        };
        if let Some((points, pen)) = overlay.partial
            && points.len() >= 2
        {
            let width = pen_width(pen, self.drawing.scale()) as f32 / per_device;
            let mut path = PathBuilder::stroke(px(width));
            path.move_to(to_window(points[0]));
            for p in &points[1..] {
                path.line_to(to_window(*p));
            }
            let [r, g, b, a] = pen.color;
            let color = Rgba {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: a as f32 / 255.0,
            };
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        }
        let light = {
            let [r, g, b, _] = self.drawing.background();
            0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32 > 140.0
        };
        let colour: Hsla = rgb(if light {
            THINKING_ON_LIGHT
        } else {
            THINKING_ON_DARK
        })
        .into();
        let px_per_step = self.drawing.scale() as f32 / per_device;
        if let Some((thought, left)) = overlay.thinking.fading {
            paint_thought(
                thought,
                left as f32,
                colour,
                px_per_step,
                &to_window,
                window,
                cx,
            );
        }
        if let Some(thought) = overlay.thinking.current {
            paint_thought(thought, 1.0, colour, px_per_step, &to_window, window, cx);
        }

        if let Some((pos, heading)) = overlay.turtle {
            let centre = to_window(pos);
            let (turtle, turn) = turtle_placement(
                centre.x.as_f32() * per_device,
                centre.y.as_f32() * per_device,
                heading,
                per_device,
            );
            for (path, svg, colour) in TURTLE {
                window
                    .paint_svg(turtle, path.into(), Some(svg), turn, rgb(colour).into(), cx)
                    .ok();
            }
        }
    }

    /// The canvas outside the square turtle area is shown slightly lighter,
    /// since the turtle can't draw there in `WRAP` or `FENCE` mode (ADR 0003).
    fn shade_outside(&self, bounds: Bounds<Pixels>, window: &mut Window, per_device: f32) {
        let side = self.drawing.width().min(self.drawing.height()) as f32 / per_device;
        let shade = hsla(0.0, 0.0, 1.0, 0.06);
        let (w, h) = (bounds.size.width.as_f32(), bounds.size.height.as_f32());
        let (gap_x, gap_y) = ((w - side) / 2.0, (h - side) / 2.0);
        let o = bounds.origin;
        if gap_x > 0.5 {
            window.paint_quad(fill(Bounds::new(o, size(px(gap_x), px(h))), shade));
            window.paint_quad(fill(
                Bounds::new(point(o.x + px(w - gap_x), o.y), size(px(gap_x), px(h))),
                shade,
            ));
        }
        if gap_y > 0.5 {
            window.paint_quad(fill(Bounds::new(o, size(px(w), px(gap_y))), shade));
            window.paint_quad(fill(
                Bounds::new(point(o.x, o.y + px(h - gap_y)), size(px(w), px(gap_y))),
                shade,
            ));
        }
    }
}

/// Where to paint the turtle's pictures, centred on `(x, y)` in device
/// pixels, and how to turn them by `heading`. gpui puts an SVG on whole
/// device pixels, so a turtle placed directly would move a pixel at a time
/// while its line moves smoothly, and jitter against it. So the pictures go
/// on whole pixels, and the turn, which gpui doesn't round, moves them the
/// rest of the way.
fn turtle_placement(
    x: f32,
    y: f32,
    heading: f64,
    per_device: f32,
) -> (Bounds<Pixels>, TransformationMatrix) {
    let (whole_x, whole_y) = (x.round(), y.round());
    let half = (TURTLE_SIZE * per_device / 2.).round();
    let logical = |device: f32| px(device / per_device);
    let bounds = Bounds::new(
        point(logical(whole_x - half), logical(whole_y - half)),
        size(logical(2. * half), logical(2. * half)),
    );
    // Clockwise about the centre, as gpui's `svg()` turns, then on to `(x, y)`.
    let turn = TransformationMatrix::unit()
        .translate(point(ScaledPixels(x), ScaledPixels(y)))
        .rotate(Radians(heading.to_radians() as f32))
        .translate(point(ScaledPixels(-whole_x), ScaledPixels(-whole_y)));
    (bounds, turn)
}

/// One of Kame's thoughts, `strength` from 1 for solid down to 0 as it
/// fades: a turn's wedge and number, or a move's ruler and length so far.
fn paint_thought(
    thought: Thought,
    strength: f32,
    colour: Hsla,
    px_per_step: f32,
    to_window: &dyn Fn(Point) -> gpui_kit::Point<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    let faded = |a: f32| colour.opacity(a * strength);
    let line = |from, to, width: f32, a: f32, window: &mut Window| {
        let mut path = PathBuilder::stroke(px(width));
        path.move_to(from);
        path.line_to(to);
        if let Ok(path) = path.build() {
            window.paint_path(path, faded(a));
        }
    };
    match thought {
        Thought::Turn { at, from, by, done } => {
            let centre = to_window(at);
            // What's left over whole turns, in the turn's direction; a whole
            // number of turns shows a whole circle.
            let mut shown = by % 360.0;
            if shown == 0.0 && by != 0.0 {
                shown = 360.0 * by.signum();
            }
            let to = from + shown;
            line(
                centre,
                towards(centre, from, ARC_RADIUS * 1.5),
                1.0,
                0.45,
                window,
            );
            line(
                centre,
                towards(centre, to, ARC_RADIUS * 1.5),
                1.0,
                0.45,
                window,
            );
            let steps = (shown.abs() / 3.0).ceil().max(1.0) as usize;
            let rim: Vec<_> = (0..=steps)
                .map(|i| towards(centre, from + shown * i as f64 / steps as f64, ARC_RADIUS))
                .collect();
            let mut wedge = PathBuilder::fill();
            wedge.move_to(centre);
            for p in &rim {
                wedge.line_to(*p);
            }
            wedge.close();
            if let Ok(path) = wedge.build() {
                window.paint_path(path, faded(0.22));
            }
            let mut edge = PathBuilder::stroke(px(1.0));
            edge.move_to(rim[0]);
            for p in &rim[1..] {
                edge.line_to(*p);
            }
            if let Ok(path) = edge.build() {
                window.paint_path(path, faded(0.7));
            }
            let label = format!("{}°", round(by.abs(), done));
            paint_label(
                &label,
                towards(centre, from + shown / 2.0, ARC_RADIUS + 18.0),
                faded(0.95),
                window,
                cx,
            );
        }
        Thought::Move {
            from,
            to,
            length,
            heading,
            done,
        } => {
            let (start, end) = (to_window(from), to_window(to));
            // The ruler goes on Kame's left, whichever way it moves.
            let side = towards(point(px(0.), px(0.)), heading - 90.0, RULER_GAP);
            let offset = |p: gpui_kit::Point<Pixels>| point(p.x + side.x, p.y + side.y);
            line(offset(start), offset(end), 1.0, 0.8, window);
            // A tick every 10 steps, longer every 50, or fewer when they'd be
            // under 4 pixels apart.
            let every: f32 = [10.0, 50.0, 100.0]
                .into_iter()
                .find(|e| e * px_per_step >= 4.0)
                .unwrap_or(100.0);
            let along = |steps: f64| {
                let t = if length > 0.0 {
                    (steps / length) as f32
                } else {
                    0.0
                };
                point(
                    start.x + (end.x - start.x) * t,
                    start.y + (end.y - start.y) * t,
                )
            };
            let mut n = 0.0;
            while n <= length + 1e-9 {
                let long = (n / every as f64).round() as i64 % 5 == 0;
                let tick = offset(along(n));
                let reach = if long { 8.0 } else { 4.0 };
                line(tick, towards(tick, heading - 90.0, reach), 1.0, 0.8, window);
                n += every as f64;
            }
            let label = round(length, done);
            // Out to the side of the ruler's end, clear of Kame.
            let beyond = towards(offset(end), heading - 90.0, 22.0);
            paint_label(&label, beyond, faded(0.95), window, cx);
        }
    }
}

/// A number as Kame thinks of it: whole while it's still turning or
/// moving, and to one decimal place once it's `done`, so `FD 12.5` ends on
/// 12.5.
fn round(n: f64, done: bool) -> String {
    if !done {
        return format!("{:.0}", n.floor());
    }
    let tenths = (n * 10.0).round() / 10.0;
    if tenths.fract() == 0.0 {
        format!("{tenths:.0}")
    } else {
        format!("{tenths:.1}")
    }
}

/// `text` centred on `at`, in the editor's font.
fn paint_label(
    text: &str,
    at: gpui_kit::Point<Pixels>,
    colour: Hsla,
    window: &mut Window,
    cx: &mut App,
) {
    let size = px(13.0);
    let mut font = font(cx.theme().mono_font_family.clone());
    font.weight = FontWeight::BOLD;
    let run = TextRun {
        len: text.len(),
        font,
        color: colour,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let shaped = window
        .text_system()
        .shape_line(text.to_string().into(), size, &[run], None);
    let line_height = px(16.0);
    let origin = point(at.x - shaped.width() / 2.0, at.y - line_height / 2.0);
    shaped
        .paint(origin, line_height, TextAlign::Left, None, window, cx)
        .ok();
}

#[cfg(test)]
mod tests {
    use super::turtle_placement;

    #[test]
    fn the_turtle_is_on_whole_pixels_and_turned_onto_its_position() {
        for per_device in [1.0, 1.5, 2.0] {
            for (x, y) in [(100.0, 50.0), (100.3, 50.5), (100.7, 49.49)] {
                let (bounds, turn) = turtle_placement(x, y, 30.0, per_device);
                // gpui's rounding leaves the pictures where they are.
                for edge in [bounds.left(), bounds.top(), bounds.right(), bounds.bottom()] {
                    let device = edge.as_f32() * per_device;
                    assert!(
                        (device - device.round()).abs() < 1e-3,
                        "{device} at {per_device}x"
                    );
                }
                // The turn takes their centre to the turtle's position.
                let c = bounds.center();
                let [[a, b], [cc, d]] = turn.rotation_scale;
                let [tx, ty] = turn.translation;
                let (cx, cy) = (c.x.as_f32() * per_device, c.y.as_f32() * per_device);
                let (mx, my) = (a * cx + b * cy + tx, cc * cx + d * cy + ty);
                assert!(
                    (mx - x).abs() < 1e-3 && (my - y).abs() < 1e-3,
                    "({mx}, {my}) for ({x}, {y}) at {per_device}x"
                );
            }
        }
    }
}
