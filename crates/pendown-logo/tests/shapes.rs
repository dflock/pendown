//! `ARC` and the shapes (phase 1 issue 08).

mod common;

use common::*;
use pendown_logo::Event;

/// An arc: centre, radius, start heading, angle, and whether the pen was down.
type ArcEvent = ((f64, f64), f64, f64, f64, bool);

fn arcs(r: &Run) -> Vec<ArcEvent> {
    r.events
        .iter()
        .filter_map(|e| match e {
            Event::Arc { center, radius, start_heading, angle, pen } => {
                Some(((center.x, center.y), *radius, *start_heading, *angle, pen.down))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn arc_walks_the_turtle_along_it() {
    let r = run("ARC 90 100 SHOW POS PRINT HEADING");
    assert_eq!(arcs(&r), [((100.0, 0.0), 100.0, 0.0, 90.0, true)], "centre on the right for a right turn");
    assert_eq!(r.printed(), "[100 100]\n90\n");
    let r = run("ARC -90 100 SHOW POS PRINT HEADING");
    assert_eq!(arcs(&r), [((-100.0, 0.0), 100.0, 0.0, -90.0, true)], "and on the left for a left turn");
    assert_eq!(r.printed(), "[-100 100]\n270\n");
}

#[test]
fn full_circles_come_back_to_the_start() {
    assert_eq!(output("ARC 360 50 SHOW POS PRINT HEADING"), "[0 0]\n0\n");
    assert_eq!(output("REPEAT 4 [ARC 90 50] SHOW POS PRINT HEADING"), "[0 0]\n0\n");
    assert_eq!(output("RT 30 ARC 720 80 SHOW POS PRINT HEADING"), "[0 0]\n30\n");
}

#[test]
fn arc_edge_cases() {
    assert_eq!(run("ARC 90 0").turns(), [(0.0, 90.0)], "no radius: just the turn");
    assert!(run("ARC 0 100").events.is_empty(), "no angle: nothing");
    let r = run("PU ARC 90 100 SHOW POS");
    assert!(!arcs(&r)[0].4, "the pen is up, but the turtle still walks");
    assert_eq!(r.printed(), "[100 100]\n");
    assert_eq!(error("ARC 90 -5"), "ARC doesn't like -5 as input");
}

#[test]
fn arcs_wrap_like_moves() {
    // From (450, 0) heading north, a half circle right of radius 100 goes
    // out past the right edge, and ends at (650, 0), which wraps to
    // (-350, 0).
    let r = run("PU SETXY 450 0 PD ARC 180 100 SHOW POS PRINT HEADING");
    assert!(arcs(&r).is_empty(), "drawn as short moves instead");
    assert!(r.moves().len() > 100);
    assert_eq!(r.printed(), "[-350 0]\n180\n");
    // Inside the area, it's one smooth arc.
    assert_eq!(arcs(&run("PU SETXY 300 0 PD ARC 180 100")).len(), 1);
    // WINDOW lets it go off the area as one arc.
    assert_eq!(arcs(&run("WINDOW PU SETXY 450 0 PD ARC 180 100")).len(), 1);
}

#[test]
fn arcs_stop_at_the_fence() {
    let r = run("FENCE PU SETXY 450 0 PD ARC 180 100");
    assert_eq!(r.error().message, "The turtle can't go past the edge in FENCE mode. Use WRAP or WINDOW to let it.");
    let ((_, _), (x, _), _) = *r.moves().last().unwrap();
    assert!((x - 500.0).abs() < 1e-9, "stopped at the edge, at x = {x}");
}

fn shapes(r: &Run) -> Vec<String> {
    r.events
        .iter()
        .filter_map(|e| match e {
            Event::Ellipse { center, radii, heading, pen, fill } => {
                Some(format!("ellipse at ({}, {}) radii {:?} heading {heading} pen {} fill {fill:?}", center.x, center.y, radii, pen.down))
            }
            Event::Rect { center, size, corner, heading, pen, fill } => Some(format!(
                "rect at ({}, {}) size {:?} corner {corner} heading {heading} pen {} fill {fill:?}",
                center.x, center.y, size, pen.down
            )),
            _ => None,
        })
        .collect()
}

#[test]
fn shapes_are_centred_on_the_turtle_which_stays_put() {
    let r = run("SETXY 10 20 RT 30 CIRCLE 50 ELLIPSE 20 40 SQUARE 100 RECTANGLE 60 30 SHOW POS PRINT HEADING");
    assert_eq!(
        shapes(&r),
        [
            "ellipse at (10, 20) radii (50.0, 50.0) heading 30 pen true fill None",
            "ellipse at (10, 20) radii (20.0, 40.0) heading 30 pen true fill None",
            "rect at (10, 20) size (100.0, 100.0) corner 0 heading 30 pen true fill None",
            "rect at (10, 20) size (60.0, 30.0) corner 0 heading 30 pen true fill None",
        ]
    );
    assert_eq!(r.printed(), "[10 20]\n30\n");
}

#[test]
fn rounded_corners() {
    let r = run("(SQUARE 100 10) (RECTANGLE 100 50 80)");
    assert_eq!(
        shapes(&r),
        [
            "rect at (0, 0) size (100.0, 100.0) corner 10 heading 0 pen true fill None",
            // A corner can't be rounder than half the shorter side.
            "rect at (0, 0) size (100.0, 50.0) corner 25 heading 0 pen true fill None",
        ]
    );
}

#[test]
fn shape_errors() {
    assert_eq!(error("CIRCLE -1"), "CIRCLE doesn't like -1 as input");
    assert_eq!(error("ELLIPSE 10 \"wide"), "ELLIPSE doesn't like wide as input");
    assert_eq!(error("(SQUARE 10 -1)"), "SQUARE doesn't like -1 as input");
    assert!(matches!(run("(SQUARE 1 2 3)").result, Err(pendown_logo::RunError::Parse(_))), "too many inputs");
}
