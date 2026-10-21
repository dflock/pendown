//! `WRAP`, `WINDOW` and `FENCE` (phase 1 issue 06).

mod common;

use common::*;

#[test]
fn wrap_is_the_default() {
    assert_eq!(output("PRINT TURTLEMODE"), "WRAP\n");
    let r = run("FD 700 SHOW POS");
    assert_eq!(r.moves(), [((0.0, 0.0), (0.0, 500.0), true), ((0.0, -500.0), (0.0, -300.0), true)]);
    assert_eq!(r.printed(), "[0 -300]\n");
}

#[test]
fn wrapping_more_than_once_and_diagonally() {
    assert_eq!(output("FD 2300 SHOW POS"), "[0 300]\n");
    let r = run("RT 45 FD 1000 SHOW POS");
    assert!(r.moves().len() >= 2, "{:?}", r.moves());
    // 1000 steps at 45 degrees is about 707 in each direction, so it wraps
    // to about -293 on both axes.
    assert_eq!(r.printed(), "[-292.893218813452 -292.893218813452]\n");
}

#[test]
fn setpos_wraps_on_the_way() {
    let r = run("SETPOS [700 0] SHOW POS");
    assert_eq!(r.moves(), [((0.0, 0.0), (500.0, 0.0), true), ((-500.0, 0.0), (-300.0, 0.0), true)]);
    assert_eq!(r.printed(), "[-300 0]\n");
}

#[test]
fn fence_stops_at_the_edge_with_an_error() {
    let r = run("FENCE FD 700");
    assert_eq!(r.moves(), [((0.0, 0.0), (0.0, 500.0), true)]);
    assert_eq!(r.error().message, "The turtle can't go past the edge in FENCE mode. Use WRAP or WINDOW to let it.");
    assert_eq!(output("FENCE FD 500 SHOW POS RT 180 FD 100 SHOW POS"), "[0 500]\n[0 400]\n", "up to the edge is fine");
    // SETPOS draws up to the fence too.
    let r = run("FENCE SETPOS [0 900]");
    assert_eq!(r.moves(), [((0.0, 0.0), (0.0, 500.0), true)]);
}

#[test]
fn window_goes_anywhere_and_wrap_brings_it_back() {
    assert_eq!(output("WINDOW FD 700 SHOW POS PRINT TURTLEMODE"), "[0 700]\nWINDOW\n");
    let r = run("WINDOW FD 1700 WRAP SHOW POS");
    assert_eq!(r.printed(), "[0 -300]\n");
    // The jump back in isn't drawn.
    assert_eq!(r.moves().last().unwrap(), &((0.0, 1700.0), (0.0, -300.0), false));
    assert_eq!(output("WINDOW SETXY 600 -600 FENCE SHOW POS PRINT TURTLEMODE"), "[-400 400]\nFENCE\n");
}

#[test]
fn run_starts_in_wrap_mode() {
    use std::sync::Arc;
    let events = Shared::default();
    let mut it = pendown_logo::Interpreter::new(Box::new(events), Arc::default());
    it.run_program("FENCE").unwrap();
    it.run_program("FD 700").unwrap();
}

#[test]
fn other_area_sizes() {
    use std::sync::Arc;
    let events = Shared::default();
    let mut it = pendown_logo::Interpreter::new(Box::new(events.clone()), Arc::default());
    it.set_area(400.0, 200.0);
    it.run_program("FD 150 SHOW POS").unwrap();
    let printed: String = events.0.borrow().iter().filter_map(|e| match e {
        pendown_logo::Event::Print(t) => Some(t.clone()),
        _ => None,
    }).collect();
    assert_eq!(printed, "[0 -50]\n");
}
