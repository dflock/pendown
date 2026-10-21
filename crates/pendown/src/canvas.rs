//! The canvas (ADR 0010): `pendown-render`'s pixel buffer shown as tiles,
//! with the move in progress and the turtle painted as paths each frame.

use std::{sync::Arc, time::Duration};

use gpui_kit::*;
use pendown_logo::Point;
use pendown_render::{Drawing, Segment, pen_width};

pub struct Canvas {
    pub drawing: Drawing,
    tiles: Vec<Option<Arc<RenderImage>>>,
    /// How long the last re-render after a resize took, and of how much.
    pub rerender: Option<(usize, Duration)>,
}

/// What changes each frame, painted on top of the buffer.
#[derive(Clone, Copy)]
pub struct Overlay {
    pub partial: Option<Segment>,
    /// The turtle's position and heading, if it's shown.
    pub turtle: Option<(Point, f64)>,
}

impl Canvas {
    pub fn new() -> Canvas {
        Canvas { drawing: Drawing::new(), tiles: Vec::new(), rerender: None }
    }

    pub fn paint(&mut self, bounds: Bounds<Pixels>, window: &mut Window, overlay: Overlay) {
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
        if let Some(segment) = overlay.partial {
            let width = pen_width(segment.pen, self.drawing.scale()) as f32 / per_device;
            let mut path = PathBuilder::stroke(px(width));
            path.move_to(to_window(segment.from));
            path.line_to(to_window(segment.to));
            let [r, g, b, a] = segment.pen.color;
            let color = Rgba { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: a as f32 / 255.0 };
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        }
        if let Some((pos, heading)) = overlay.turtle {
            // An isosceles triangle with the turtle's position at the middle
            // of its short side, 18 logical pixels long whatever the zoom.
            let length = 18.0 * per_device as f64 / self.drawing.scale().max(0.01);
            let (s, c) = heading.to_radians().sin_cos();
            let at = |forward: f64, right: f64| Point { x: pos.x + forward * s + right * c, y: pos.y + forward * c - right * s };
            let mut triangle = PathBuilder::fill();
            triangle.move_to(to_window(at(length, 0.0)));
            triangle.line_to(to_window(at(0.0, length * 0.4)));
            triangle.line_to(to_window(at(0.0, -length * 0.4)));
            triangle.close();
            if let Ok(triangle) = triangle.build() {
                window.paint_path(triangle, rgb(0x4caf50));
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
