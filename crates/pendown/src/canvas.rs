//! The canvas (ADR 0010): `pendown-render`'s pixel buffer shown as tiles,
//! with the move in progress painted as a path each frame, and the turtle as
//! two SVGs, which gpui renders once and keeps on the GPU.

use std::{rc::Rc, sync::Arc, time::Duration};

use gpui_kit::*;
use pendown_logo::{Pen, Point};
use pendown::metrics::WindowMetrics;
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
    ("pendown/turtle-body.svg", include_bytes!("../images/turtle-body.svg"), 0x2e7d32),
    ("pendown/turtle-shell.svg", include_bytes!("../images/turtle-shell.svg"), 0x8bc34a),
];

/// What changes each frame, painted on top of the buffer.
#[derive(Clone)]
pub struct Overlay {
    /// What's drawn so far of the move, arc or shape in progress.
    pub partial: Option<(Vec<Point>, Pen)>,
    /// The turtle's position and heading, if it's shown.
    pub turtle: Option<(Point, f64)>,
}

impl Canvas {
    pub fn new(metrics: Rc<WindowMetrics>) -> Canvas {
        Canvas { drawing: Drawing::new(), tiles: Vec::new(), rerender: None, metrics }
    }

    pub fn paint(&mut self, bounds: Bounds<Pixels>, window: &mut Window, overlay: Overlay, cx: &App) {
        let metrics = self.metrics.clone();
        metrics.canvas.time(|| self.paint_timed(bounds, window, overlay, cx));
    }

    fn paint_timed(&mut self, bounds: Bounds<Pixels>, window: &mut Window, overlay: Overlay, cx: &App) {
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
            let buffer = image::RgbaImage::from_raw(tile.width, tile.height, bytes).expect("buffer matches size");
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
                point(bounds.origin.x + logical(tile.x as f64), bounds.origin.y + logical(tile.y as f64)),
                size(logical(tile.width as f64), logical(tile.height as f64)),
            );
            window.paint_image(tile_bounds, tile_bounds, Corners::default(), image.clone(), 0, false).ok();
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
            let color = Rgba { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: a as f32 / 255.0 };
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        }
        if let Some((pos, heading)) = overlay.turtle {
            let centre = to_window(pos);
            let half = px(TURTLE_SIZE / 2.);
            let turtle = Bounds::new(point(centre.x - half, centre.y - half), size(px(TURTLE_SIZE), px(TURTLE_SIZE)));
            // Turned clockwise by its heading, about its centre, as gpui's
            // `svg()` does, in device pixels.
            let (x, y) = (centre.x.as_f32() * per_device, centre.y.as_f32() * per_device);
            let turn = TransformationMatrix::unit()
                .translate(point(ScaledPixels(x), ScaledPixels(y)))
                .rotate(Radians(heading.to_radians() as f32))
                .translate(point(ScaledPixels(-x), ScaledPixels(-y)));
            for (path, svg, colour) in TURTLE {
                window.paint_svg(turtle, path.into(), Some(svg), turn, rgb(colour).into(), cx).ok();
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
            window.paint_quad(fill(Bounds::new(point(o.x + px(w - gap_x), o.y), size(px(gap_x), px(h))), shade));
        }
        if gap_y > 0.5 {
            window.paint_quad(fill(Bounds::new(o, size(px(w), px(gap_y))), shade));
            window.paint_quad(fill(Bounds::new(point(o.x, o.y + px(h - gap_y)), size(px(w), px(gap_y))), shade));
        }
    }
}
