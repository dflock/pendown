//! What can be drawn: lines, arcs (ADR 0016) and shapes (ADR 0017). Each is
//! flattened into a polyline in turtle coordinates, which the renderer
//! strokes, the animator measures, and the GUI draws part of while a shape
//! sweeps round.

use pendown_logo::{Pen, Point, arc_point};

use crate::Segment;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mark {
    Line(Segment),
    Arc {
        center: Point,
        radius: f64,
        start_heading: f64,
        angle: f64,
        pen: Pen,
    },
    /// Shapes are filled with `fill`, if there is one, then outlined if the
    /// pen is down (ADR 0017).
    Ellipse {
        center: Point,
        radii: (f64, f64),
        heading: f64,
        pen: Pen,
        fill: Option<[u8; 4]>,
    },
    Rect {
        center: Point,
        size: (f64, f64),
        corner: f64,
        heading: f64,
        pen: Pen,
        fill: Option<[u8; 4]>,
    },
}

impl From<Segment> for Mark {
    fn from(segment: Segment) -> Mark {
        Mark::Line(segment)
    }
}

/// How many straight pieces a curve of this radius, turning through
/// `radians`, needs to stay within `tolerance` of the true curve.
fn pieces(radius: f64, radians: f64, tolerance: f64) -> usize {
    if radius <= tolerance {
        return radians.abs().ceil().max(1.0) as usize;
    }
    let per_piece = 2.0 * (1.0 - tolerance / radius).acos();
    (radians.abs() / per_piece).ceil().clamp(1.0, 100_000.0) as usize
}

/// From the shape's own frame, where y is along the heading and x across
/// it to the right, to turtle coordinates.
fn placed(center: Point, heading: f64, x: f64, y: f64) -> Point {
    let (sin, cos) = heading.to_radians().sin_cos();
    Point {
        x: center.x + x * cos + y * sin,
        y: center.y - x * sin + y * cos,
    }
}

impl Mark {
    pub fn pen(&self) -> Pen {
        match self {
            Mark::Line(s) => s.pen,
            Mark::Arc { pen, .. } | Mark::Ellipse { pen, .. } | Mark::Rect { pen, .. } => *pen,
        }
    }

    pub fn fill(&self) -> Option<[u8; 4]> {
        match self {
            Mark::Ellipse { fill, .. } | Mark::Rect { fill, .. } => *fill,
            Mark::Line(_) | Mark::Arc { .. } => None,
        }
    }

    /// Whether drawing it changes anything: an outline with the pen down,
    /// or a fill.
    pub fn visible(&self) -> bool {
        self.pen().down || self.fill().is_some()
    }

    /// Whether the outline goes all the way round.
    pub fn closed(&self) -> bool {
        matches!(self, Mark::Ellipse { .. } | Mark::Rect { .. })
    }

    /// The outline, as points no further than `tolerance` turtle steps from
    /// the true curve. Shapes start straight ahead of the turtle and go
    /// clockwise, which is how they sweep round when animated.
    pub fn outline(&self, tolerance: f64) -> Vec<Point> {
        match *self {
            Mark::Line(s) => vec![s.from, s.to],
            Mark::Arc {
                center,
                radius,
                start_heading,
                angle,
                ..
            } => {
                let n = pieces(radius, angle.to_radians(), tolerance);
                (0..=n)
                    .map(|i| arc_point(center, radius, start_heading, angle, i as f64 / n as f64).0)
                    .collect()
            }
            Mark::Ellipse {
                center,
                radii: (across, along),
                heading,
                ..
            } => {
                let n = pieces(across.max(along), std::f64::consts::TAU, tolerance).max(8);
                (0..=n)
                    .map(|i| {
                        let theta = std::f64::consts::TAU * i as f64 / n as f64;
                        placed(center, heading, across * theta.sin(), along * theta.cos())
                    })
                    .collect()
            }
            Mark::Rect {
                center,
                size: (across, along),
                corner,
                heading,
                ..
            } => {
                let (w, l, r) = (across / 2.0, along / 2.0, corner);
                let mut local = vec![(0.0, l)];
                // Each corner, clockwise from the front right: its centre,
                // and the angle (clockwise from straight ahead) it starts at.
                let corners = [
                    ((w - r, l - r), 0.0),
                    ((w - r, -l + r), 90.0),
                    ((-w + r, -l + r), 180.0),
                    ((-w + r, l - r), 270.0),
                ];
                for ((cx, cy), start) in corners {
                    if r > 0.0 {
                        let n = pieces(r, std::f64::consts::FRAC_PI_2, tolerance);
                        for i in 0..=n {
                            let a = (start + 90.0 * i as f64 / n as f64).to_radians();
                            local.push((cx + r * a.sin(), cy + r * a.cos()));
                        }
                    } else {
                        local.push((cx, cy));
                    }
                }
                local.push((0.0, l));
                local
                    .into_iter()
                    .map(|(x, y)| placed(center, heading, x, y))
                    .collect()
            }
        }
    }

    /// The length of the outline in turtle steps: how far the turtle or the
    /// pen travels drawing it.
    pub fn length(&self) -> f64 {
        length(&self.outline(0.01))
    }

    /// The first `fraction` of the outline, for drawing a shape sweeping
    /// round.
    pub fn partial(&self, fraction: f64, tolerance: f64) -> Vec<Point> {
        let points = self.outline(tolerance);
        let want = length(&points) * fraction.clamp(0.0, 1.0);
        let mut out = vec![points[0]];
        let mut gone = 0.0;
        for pair in points.windows(2) {
            let step = distance(pair[0], pair[1]);
            if gone + step >= want {
                let t = if step > 0.0 {
                    (want - gone) / step
                } else {
                    0.0
                };
                out.push(Point {
                    x: pair[0].x + (pair[1].x - pair[0].x) * t,
                    y: pair[0].y + (pair[1].y - pair[0].y) * t,
                });
                return out;
            }
            gone += step;
            out.push(pair[1]);
        }
        out
    }
}

fn distance(a: Point, b: Point) -> f64 {
    ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt()
}

fn length(points: &[Point]) -> f64 {
    points.windows(2).map(|w| distance(w[0], w[1])).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PEN: Pen = Pen {
        down: true,
        color: [255, 255, 255, 255],
        size: 1.0,
    };

    fn p(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    fn close(a: Point, b: Point) -> bool {
        distance(a, b) < 1e-6
    }

    #[test]
    fn a_circles_length_is_its_circumference() {
        let circle = Mark::Ellipse {
            center: p(0.0, 0.0),
            radii: (100.0, 100.0),
            heading: 0.0,
            pen: PEN,
            fill: None,
        };
        assert!((circle.length() - std::f64::consts::TAU * 100.0).abs() < 0.1);
        let points = circle.outline(0.1);
        assert!(close(points[0], p(0.0, 100.0)), "starts straight ahead");
        assert!(points[1].x > 0.0, "and goes clockwise, to the right first");
    }

    #[test]
    fn ellipses_turn_with_the_heading() {
        // At heading 90, "along" points east.
        let e = Mark::Ellipse {
            center: p(0.0, 0.0),
            radii: (20.0, 50.0),
            heading: 90.0,
            pen: PEN,
            fill: None,
        };
        assert!(close(e.outline(0.1)[0], p(50.0, 0.0)));
    }

    #[test]
    fn rectangles() {
        let r = Mark::Rect {
            center: p(0.0, 0.0),
            size: (200.0, 100.0),
            corner: 0.0,
            heading: 0.0,
            pen: PEN,
            fill: None,
        };
        let points = r.outline(0.1);
        assert_eq!(points.len(), 6);
        assert!(
            close(points[0], p(0.0, 50.0))
                && close(points[1], p(100.0, 50.0))
                && close(points[2], p(100.0, -50.0))
        );
        assert!((r.length() - 600.0).abs() < 1e-6);
        // Rounded corners make it shorter: each corner trades 2r for a
        // quarter circle.
        let rounded = Mark::Rect {
            center: p(0.0, 0.0),
            size: (200.0, 100.0),
            corner: 10.0,
            heading: 0.0,
            pen: PEN,
            fill: None,
        };
        let expected = 600.0 - 4.0 * 20.0 + std::f64::consts::TAU * 10.0;
        assert!(
            (rounded.length() - expected).abs() < 0.1,
            "{}",
            rounded.length()
        );
    }

    #[test]
    fn arcs_follow_the_turtle() {
        let arc = Mark::Arc {
            center: p(100.0, 0.0),
            radius: 100.0,
            start_heading: 0.0,
            angle: 90.0,
            pen: PEN,
        };
        let points = arc.outline(0.1);
        assert!(close(points[0], p(0.0, 0.0)));
        assert!(close(*points.last().unwrap(), p(100.0, 100.0)));
        assert!((arc.length() - std::f64::consts::FRAC_PI_2 * 100.0).abs() < 0.01);
    }

    #[test]
    fn partial_outlines() {
        let line = Mark::Line(Segment {
            from: p(0.0, 0.0),
            to: p(0.0, 100.0),
            pen: PEN,
        });
        assert_eq!(line.partial(0.25, 0.1), [p(0.0, 0.0), p(0.0, 25.0)]);
        let square = Mark::Rect {
            center: p(0.0, 0.0),
            size: (100.0, 100.0),
            corner: 0.0,
            heading: 0.0,
            pen: PEN,
            fill: None,
        };
        // Half way round from the top middle: the bottom middle.
        assert!(close(
            *square.partial(0.5, 0.1).last().unwrap(),
            p(0.0, -50.0)
        ));
    }
}
