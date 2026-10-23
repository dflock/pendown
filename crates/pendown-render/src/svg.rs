//! The drawing as SVG, from its marks rather than its pixels: each line, arc
//! and shape becomes an element of its own, so it scales without blurring.
//! It shows the whole turtle area, in turtle steps, with y flipped to point
//! down as SVG's does.

use std::fmt::Write as _;

use pendown_logo::{Point, arc_point};

use crate::{Drawing, Mark};

/// A number with at most three decimals, and no trailing zeros.
fn num(value: f64) -> String {
    let text = format!("{value:.3}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" { "0".into() } else { text.into() }
}

/// A turtle position in SVG coordinates.
fn at(p: Point) -> String {
    format!("{} {}", num(p.x), num(-p.y))
}

/// A colour's `#rrggbb`, and its opacity if it isn't opaque.
fn colour([r, g, b, a]: [u8; 4]) -> (String, Option<String>) {
    (format!("#{r:02x}{g:02x}{b:02x}"), (a < 255).then(|| num(a as f64 / 255.0)))
}

/// The fill and stroke attributes for a mark, with no line thinner than
/// `thinnest` steps.
fn paint(mark: &Mark, thinnest: f64) -> String {
    let mut out = String::new();
    if let Some(fill) = mark.fill() {
        let (hex, opacity) = colour(fill);
        let _ = write!(out, r#" fill="{hex}""#);
        if let Some(o) = opacity {
            let _ = write!(out, r#" fill-opacity="{o}""#);
        }
    }
    let pen = mark.pen();
    if pen.down {
        let (hex, opacity) = colour(pen.color);
        let _ = write!(out, r#" stroke="{hex}" stroke-width="{}""#, num(pen.size.max(thinnest)));
        if let Some(o) = opacity {
            let _ = write!(out, r#" stroke-opacity="{o}""#);
        }
    }
    out
}

/// A rotation by a turtle heading about `center`: headings and SVG angles
/// both go clockwise on the screen.
fn turned(heading: f64, center: Point) -> String {
    if heading.rem_euclid(360.0) == 0.0 { String::new() } else { format!(r#" transform="rotate({} {})""#, num(heading), at(center)) }
}

/// One mark as an SVG element.
fn element(mark: &Mark, thinnest: f64) -> String {
    let paint = paint(mark, thinnest);
    match *mark {
        Mark::Line(s) => {
            format!(r#"<line x1="{}" y1="{}" x2="{}" y2="{}"{paint}/>"#, num(s.from.x), num(-s.from.y), num(s.to.x), num(-s.to.y))
        }
        Mark::Arc { center, radius, start_heading, angle, .. } => {
            // A piece of an SVG arc can't go all the way round, so it's
            // drawn in pieces of at most half a turn. A clockwise arc
            // (positive `angle`) sweeps clockwise on the screen.
            let pieces = (angle.abs() / 180.0).ceil().max(1.0) as usize;
            let sweep = if angle >= 0.0 { 1 } else { 0 };
            let mut d = format!("M{}", at(arc_point(center, radius, start_heading, angle, 0.0).0));
            for i in 1..=pieces {
                let end = arc_point(center, radius, start_heading, angle, i as f64 / pieces as f64).0;
                let _ = write!(d, " A{r} {r} 0 0 {sweep} {}", at(end), r = num(radius));
            }
            format!(r#"<path d="{d}"{paint}/>"#)
        }
        Mark::Ellipse { center, radii: (across, along), heading, .. } => format!(
            r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}"{}{paint}/>"#,
            num(center.x),
            num(-center.y),
            num(across),
            num(along),
            turned(heading, center),
        ),
        Mark::Rect { center, size: (across, along), corner, heading, .. } => {
            let rounded = if corner > 0.0 { format!(r#" rx="{0}" ry="{0}""#, num(corner)) } else { String::new() };
            format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}"{rounded}{}{paint}/>"#,
                num(center.x - across / 2.0),
                num(-center.y - along / 2.0),
                num(across),
                num(along),
                turned(heading, center),
            )
        }
    }
}

impl Drawing {
    /// The drawing as an SVG file: the background, then everything drawn
    /// since the last `CLEAN`, over the whole turtle area. One turtle step
    /// is one pixel. No line is thinner than `thinnest`, a fraction of the
    /// width: an SVG shown small, as the help panel's pictures are, would
    /// otherwise lose its thin lines. Zero keeps every pen's own width.
    pub fn svg(&self, thinnest: f64) -> String {
        let thinnest = thinnest * self.area;
        let area = num(self.area);
        let corner = num(-self.area / 2.0);
        let (background, opacity) = colour(self.background);
        let mut out = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{area}\" height=\"{area}\" viewBox=\"{corner} {corner} {area} {area}\">\n\
             <rect x=\"{corner}\" y=\"{corner}\" width=\"{area}\" height=\"{area}\" fill=\"{background}\"{}/>\n\
             <g fill=\"none\" stroke-linecap=\"round\" stroke-linejoin=\"round\">\n",
            opacity.map(|o| format!(r#" fill-opacity="{o}""#)).unwrap_or_default(),
        );
        for mark in &self.display {
            out.push_str(&element(mark, thinnest));
            out.push('\n');
        }
        out.push_str("</g>\n</svg>\n");
        out
    }
}

#[cfg(test)]
mod tests {
    use pendown_logo::{Pen, Point};

    use crate::{Drawing, Mark, Segment};

    const RED: Pen = Pen { down: true, color: [255, 0, 0, 128], size: 2.5 };

    fn p(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    #[test]
    fn marks_become_elements_with_y_flipped() {
        let mut d = Drawing::new();
        d.push(&[
            Segment { from: p(0.0, 0.0), to: p(0.0, 100.0), pen: RED }.into(),
            Mark::Ellipse { center: p(10.0, 20.0), radii: (30.0, 40.0), heading: 45.0, pen: Pen { down: false, ..RED }, fill: Some([0, 0, 255, 255]) },
        ]);
        let svg = d.svg(0.0);
        assert!(svg.contains(r#"viewBox="-500 -500 1000 1000""#), "{svg}");
        assert!(svg.contains(r##"<rect x="-500" y="-500" width="1000" height="1000" fill="#000000"/>"##), "{svg}");
        assert!(
            svg.contains(r##"<line x1="0" y1="0" x2="0" y2="-100" stroke="#ff0000" stroke-width="2.5" stroke-opacity="0.502"/>"##),
            "{svg}"
        );
        assert!(svg.contains(r##"<ellipse cx="10" cy="-20" rx="30" ry="40" transform="rotate(45 10 -20)" fill="#0000ff"/>"##), "{svg}");
    }

    #[test]
    fn no_line_is_thinner_than_asked() {
        let mut d = Drawing::new();
        d.push(&[Segment { from: p(0.0, 0.0), to: p(0.0, 100.0), pen: RED }.into()]);
        // A hundredth of the 1000-step area is 10 steps.
        assert!(d.svg(0.01).contains(r#"stroke-width="10""#));
        assert!(d.svg(0.001).contains(r#"stroke-width="2.5""#));
    }

    #[test]
    fn arcs_go_round_in_half_turns_at_most() {
        let mut d = Drawing::new();
        d.push(&[Mark::Arc { center: p(0.0, 0.0), radius: 50.0, start_heading: 0.0, angle: 360.0, pen: RED }]);
        let svg = d.svg(0.0);
        // The turtle starts on the centre's left, facing up, and goes
        // clockwise: over the top to the right, and back underneath.
        assert!(svg.contains(r#"d="M-50 0 A50 50 0 0 1 50 0 A50 50 0 0 1 -50 0""#), "{svg}");
    }
}
