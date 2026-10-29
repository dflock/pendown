//! The turtle area's edges (ADR 0009): the interpreter does all the
//! geometry, so the GUI only scales.

use crate::event::Point;

/// The turtle area: from `-half_width` to `half_width` and `-half_height` to
/// `half_height`, 1000 × 1000 by default (ADR 0003).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Area {
    pub half_width: f64,
    pub half_height: f64,
}

impl Default for Area {
    fn default() -> Area {
        Area {
            half_width: 500.0,
            half_height: 500.0,
        }
    }
}

/// `WRAP`, `WINDOW` and `FENCE`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TurtleMode {
    /// Crossing an edge brings the turtle back at the opposite one. UCBLogo's
    /// default.
    #[default]
    Wrap,
    /// The turtle can go anywhere, including off the area.
    Window,
    /// The turtle stops at the edge, with an error.
    Fence,
}

/// A move that wraps more than this many times only draws this many
/// pieces, then jumps to where it would end. A drawing that wraps that often
/// is already solid, and an enormous move could otherwise take forever.
pub const MAX_WRAPS: usize = 10_000;

/// How a move went.
#[derive(Debug, PartialEq)]
pub struct Path {
    /// The straight pieces to draw, in order.
    pub pieces: Vec<(Point, Point)>,
    /// Where the turtle ends up.
    pub end: Point,
    /// In `FENCE` mode, the move was stopped at the edge.
    pub fenced: bool,
}

impl Area {
    pub fn contains(&self, p: Point) -> bool {
        p.x.abs() <= self.half_width && p.y.abs() <= self.half_height
    }

    /// Wraps a position into the area, as `WRAP` and `FENCE` do to a turtle
    /// that's outside it.
    pub fn wrap_point(&self, p: Point) -> Point {
        let wrap = |v: f64, half: f64| {
            if v.abs() <= half {
                return v;
            }
            let w = (v + half).rem_euclid(2.0 * half) - half;
            // Exactly on the far edge reads better as that edge.
            if w == -half && v > 0.0 { half } else { w }
        };
        Point {
            x: wrap(p.x, self.half_width),
            y: wrap(p.y, self.half_height),
        }
    }

    /// The straight move from `from` to `to` in the given mode. `from` is
    /// inside the area, except in `WINDOW` mode.
    pub fn line(&self, from: Point, to: Point, mode: TurtleMode) -> Path {
        match mode {
            TurtleMode::Window => Path {
                pieces: vec![(from, to)],
                end: to,
                fenced: false,
            },
            TurtleMode::Fence => match self.exit(from, to) {
                None => Path {
                    pieces: vec![(from, to)],
                    end: to,
                    fenced: false,
                },
                Some((t, _)) => {
                    let edge = lerp(from, to, t);
                    let pieces = if t > 0.0 { vec![(from, edge)] } else { vec![] };
                    Path {
                        pieces,
                        end: edge,
                        fenced: true,
                    }
                }
            },
            TurtleMode::Wrap => self.wrap_line(from, to),
        }
    }

    fn wrap_line(&self, from: Point, to: Point) -> Path {
        let mut pieces = Vec::new();
        let (mut p, mut dx, mut dy) = (from, to.x - from.x, to.y - from.y);
        loop {
            let target = Point {
                x: p.x + dx,
                y: p.y + dy,
            };
            let Some((t, (crosses_x, crosses_y))) = self.exit(p, target) else {
                pieces.push((p, target));
                return Path {
                    pieces,
                    end: target,
                    fenced: false,
                };
            };
            let edge = lerp(p, target, t);
            if t > 0.0 {
                pieces.push((p, edge));
            }
            if pieces.len() >= MAX_WRAPS {
                let end = self.wrap_point(target);
                return Path {
                    pieces,
                    end,
                    fenced: false,
                };
            }
            // Reappear at the opposite edge, or edges at a corner.
            p = edge;
            if crosses_x {
                p.x = -p.x.signum() * self.half_width;
            }
            if crosses_y {
                p.y = -p.y.signum() * self.half_height;
            }
            dx *= 1.0 - t;
            dy *= 1.0 - t;
        }
    }

    /// Where a move from `from` (inside the area) towards `to` first leaves
    /// it, as a fraction of the way, with which edges it crosses there: the
    /// left or right, and the top or bottom. `None` if it stays inside.
    fn exit(&self, from: Point, to: Point) -> Option<(f64, (bool, bool))> {
        if self.contains(to) {
            return None;
        }
        let (dx, dy) = (to.x - from.x, to.y - from.y);
        // How far until each pair of edges, or infinity if it never gets there.
        let along = |start: f64, delta: f64, half: f64, end: f64| {
            if end.abs() <= half || delta == 0.0 {
                f64::INFINITY
            } else {
                let edge = half * delta.signum();
                ((edge - start) / delta).max(0.0)
            }
        };
        let tx = along(from.x, dx, self.half_width, to.x);
        let ty = along(from.y, dy, self.half_height, to.y);
        let t = tx.min(ty);
        // Within a billionth counts as both: the move goes through a corner.
        let close = |a: f64| (a - t).abs() < 1e-9;
        Some((t, (close(tx), close(ty))))
    }
}

/// Where the turtle is, and its heading, a fraction `t` of the way along an
/// arc (ADR 0016). A clockwise arc (positive `angle`) has its centre on the
/// turtle's right, so the turtle is on the centre's left: at heading `h`,
/// that's the direction `h - 90`.
pub fn arc_point(
    center: Point,
    radius: f64,
    start_heading: f64,
    angle: f64,
    t: f64,
) -> (Point, f64) {
    let heading = start_heading + angle * t;
    let side = if angle >= 0.0 { -90.0 } else { 90.0 };
    let (sin, cos) = crate::number::sin_cos_degrees(heading + side);
    (
        Point {
            x: center.x + radius * sin,
            y: center.y + radius * cos,
        },
        heading.rem_euclid(360.0),
    )
}

/// The centre of the arc the turtle walks from `from` at `heading`: on its
/// right for a clockwise arc, on its left otherwise.
pub fn arc_center(from: Point, heading: f64, radius: f64, angle: f64) -> Point {
    let side = if angle >= 0.0 { 90.0 } else { -90.0 };
    let (sin, cos) = crate::number::sin_cos_degrees(heading + side);
    Point {
        x: from.x + radius * sin,
        y: from.y + radius * cos,
    }
}

/// Points along an arc, one per degree at most, including both ends. Used
/// to split an arc into straight pieces where it crosses an edge.
pub fn arc_chords(center: Point, radius: f64, start_heading: f64, angle: f64) -> Vec<Point> {
    let n = angle.abs().ceil().max(1.0) as usize;
    (0..=n)
        .map(|i| arc_point(center, radius, start_heading, angle, i as f64 / n as f64).0)
        .collect()
}

fn lerp(a: Point, b: Point, t: f64) -> Point {
    Point {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    const AREA: Area = Area {
        half_width: 500.0,
        half_height: 500.0,
    };

    fn pieces(path: &Path) -> Vec<((f64, f64), (f64, f64))> {
        path.pieces
            .iter()
            .map(|(a, b)| ((a.x.round(), a.y.round()), (b.x.round(), b.y.round())))
            .collect()
    }

    #[test]
    fn a_move_inside_is_one_piece() {
        let path = AREA.line(p(0.0, 0.0), p(100.0, 200.0), TurtleMode::Wrap);
        assert_eq!(
            path,
            Path {
                pieces: vec![(p(0.0, 0.0), p(100.0, 200.0))],
                end: p(100.0, 200.0),
                fenced: false
            }
        );
    }

    #[test]
    fn wrapping_once() {
        let path = AREA.line(p(400.0, 0.0), p(700.0, 0.0), TurtleMode::Wrap);
        assert_eq!(
            pieces(&path),
            [((400.0, 0.0), (500.0, 0.0)), ((-500.0, 0.0), (-300.0, 0.0))]
        );
        assert_eq!(path.end, p(-300.0, 0.0));
    }

    #[test]
    fn wrapping_more_than_once() {
        // 2,500 steps up from the centre: 500 to the top, then twice the
        // whole height.
        let path = AREA.line(p(0.0, 0.0), p(0.0, 2500.0), TurtleMode::Wrap);
        assert_eq!(
            pieces(&path),
            [
                ((0.0, 0.0), (0.0, 500.0)),
                ((0.0, -500.0), (0.0, 500.0)),
                ((0.0, -500.0), (0.0, 500.0))
            ]
        );
        assert_eq!(path.end, p(0.0, 500.0));
    }

    #[test]
    fn diagonal_wrapping() {
        let path = AREA.line(p(400.0, 0.0), p(600.0, 100.0), TurtleMode::Wrap);
        assert_eq!(
            pieces(&path),
            [
                ((400.0, 0.0), (500.0, 50.0)),
                ((-500.0, 50.0), (-400.0, 100.0))
            ]
        );
    }

    #[test]
    fn through_a_corner() {
        let path = AREA.line(p(400.0, 400.0), p(600.0, 600.0), TurtleMode::Wrap);
        assert_eq!(
            pieces(&path),
            [
                ((400.0, 400.0), (500.0, 500.0)),
                ((-500.0, -500.0), (-400.0, -400.0))
            ]
        );
    }

    #[test]
    fn along_an_edge() {
        // Moving along the right edge never leaves the area.
        let path = AREA.line(p(500.0, 0.0), p(500.0, 300.0), TurtleMode::Wrap);
        assert_eq!(pieces(&path), [((500.0, 0.0), (500.0, 300.0))]);
        // Leaving from exactly on the edge wraps straight away.
        let path = AREA.line(p(500.0, 0.0), p(600.0, 0.0), TurtleMode::Wrap);
        assert_eq!(pieces(&path), [((-500.0, 0.0), (-400.0, 0.0))]);
    }

    #[test]
    fn a_huge_move_stops_drawing_after_max_wraps() {
        let path = AREA.line(p(0.0, 0.0), p(0.0, 1e12), TurtleMode::Wrap);
        assert_eq!(path.pieces.len(), MAX_WRAPS);
        assert!(AREA.contains(path.end));
    }

    #[test]
    fn fence_stops_at_the_edge() {
        let path = AREA.line(p(0.0, 0.0), p(0.0, 800.0), TurtleMode::Fence);
        assert_eq!(
            path,
            Path {
                pieces: vec![(p(0.0, 0.0), p(0.0, 500.0))],
                end: p(0.0, 500.0),
                fenced: true
            }
        );
        let path = AREA.line(p(0.0, 500.0), p(0.0, 600.0), TurtleMode::Fence);
        assert_eq!(
            path,
            Path {
                pieces: vec![],
                end: p(0.0, 500.0),
                fenced: true
            },
            "already at the fence"
        );
        let path = AREA.line(p(0.0, 500.0), p(0.0, 400.0), TurtleMode::Fence);
        assert!(!path.fenced, "moving back in is fine");
    }

    #[test]
    fn window_goes_anywhere() {
        let path = AREA.line(p(0.0, 0.0), p(2000.0, 0.0), TurtleMode::Window);
        assert_eq!(path.end, p(2000.0, 0.0));
        assert_eq!(path.pieces.len(), 1);
    }

    #[test]
    fn wrapping_a_point_back_in() {
        assert_eq!(AREA.wrap_point(p(700.0, -1200.0)), p(-300.0, -200.0));
        assert_eq!(AREA.wrap_point(p(1500.0, 0.0)), p(500.0, 0.0));
        assert_eq!(AREA.wrap_point(p(10.0, 20.0)), p(10.0, 20.0));
    }

    #[test]
    fn arcs() {
        // A quarter turn right, radius 100, from the origin heading north:
        // the centre is at (100, 0), and the turtle ends at (100, 100)
        // heading east.
        let center = arc_center(p(0.0, 0.0), 0.0, 100.0, 90.0);
        assert_eq!(center, p(100.0, 0.0));
        let (start, h0) = arc_point(center, 100.0, 0.0, 90.0, 0.0);
        let (end, h1) = arc_point(center, 100.0, 0.0, 90.0, 1.0);
        assert_eq!((start, h0), (p(0.0, 0.0), 0.0));
        assert_eq!((end, h1), (p(100.0, 100.0), 90.0));
        // A quarter turn left ends at (-100, 100) heading west.
        let center = arc_center(p(0.0, 0.0), 0.0, 100.0, -90.0);
        assert_eq!(center, p(-100.0, 0.0));
        assert_eq!(
            arc_point(center, 100.0, 0.0, -90.0, 1.0),
            (p(-100.0, 100.0), 270.0)
        );
        assert_eq!(arc_chords(center, 100.0, 0.0, -90.0).len(), 91);
    }

    #[test]
    fn other_sizes() {
        let area = Area {
            half_width: 200.0,
            half_height: 100.0,
        };
        let path = area.line(p(0.0, 0.0), p(0.0, 150.0), TurtleMode::Wrap);
        assert_eq!(
            pieces(&path),
            [((0.0, 0.0), (0.0, 100.0)), ((0.0, -100.0), (0.0, -50.0))]
        );
    }
}
