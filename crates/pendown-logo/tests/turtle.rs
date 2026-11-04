//! Turtle motion, queries and visibility (phase 1 issue 05).

mod common;

use common::*;
use pendown_logo::Event;

#[test]
fn forward_back_and_turns() {
    let r = run("FD 100 RT 90 FD 50 LT 45 BK 10");
    assert_eq!(r.turns(), [(0.0, 90.0), (90.0, -45.0)]);
    let moves = r.moves();
    assert_eq!(moves[0], ((0.0, 0.0), (0.0, 100.0), true));
    assert_eq!(
        moves[1],
        ((0.0, 100.0), (50.0, 100.0), true),
        "exact at 90 degrees"
    );
    let ((x, y), _, _) = moves[2];
    assert_eq!((x, y), (50.0, 100.0));
}

#[test]
fn positions_are_exact_at_right_angles() {
    assert_eq!(output("REPEAT 4 [FD 100 RT 90] SHOW POS"), "[0 0]\n");
    assert_eq!(
        output("RT 90 FD 100 SHOW POS PRINT XCOR PRINT YCOR"),
        "[100 0]\n100\n0\n"
    );
    // 120 degrees isn't exact, but POS hides the rounding.
    assert_eq!(
        output("REPEAT 3 [FD 100 RT 120] SHOW POS PRINT HEADING"),
        "[0 0]\n0\n"
    );
}

#[test]
fn setpos_setxy_setx_sety() {
    let r = run("SETPOS [30 40] SETXY -10 20 SETX 5 SETY -5");
    assert_eq!(
        r.moves(),
        [
            ((0.0, 0.0), (30.0, 40.0), true),
            ((30.0, 40.0), (-10.0, 20.0), true),
            ((-10.0, 20.0), (5.0, 20.0), true),
            ((5.0, 20.0), (5.0, -5.0), true),
        ]
    );
    assert_eq!(output("SETXY 3 4 SHOW POS"), "[3 4]\n");
}

#[test]
fn setheading_turns_the_shorter_way() {
    let r = run("SETH 270 SETH 90 SETH 450 SETH 90 SETHEADING -90");
    // 0 to 270 is -90; 270 to 90 is 180 either way; 450 is 90 again, so no
    // turn; -90 is 270.
    assert_eq!(r.turns(), [(0.0, -90.0), (270.0, 180.0), (90.0, 180.0)]);
    assert_eq!(output("SETH -90 PRINT HEADING"), "270\n");
}

#[test]
fn home_is_a_move_that_draws() {
    let r = run("FD 50 RT 30 HOME");
    assert_eq!(r.moves()[1], ((0.0, 50.0), (0.0, 0.0), true));
    assert_eq!(r.turns(), [(0.0, 30.0), (30.0, -30.0)]);
    assert_eq!(
        output("FD 50 RT 30 HOME SHOW POS PRINT HEADING"),
        "[0 0]\n0\n"
    );
}

#[test]
fn towards() {
    assert_eq!(output("PRINT TOWARDS [0 100]"), "0\n");
    assert_eq!(output("PRINT TOWARDS [100 0]"), "90\n");
    assert_eq!(output("PRINT TOWARDS [-100 -100]"), "225\n");
    assert_eq!(output("SETXY 100 100 PRINT TOWARDS [0 0]"), "225\n");
    assert_eq!(output("PRINT TOWARDS [0 0]"), "0\n", "already there");
}

#[test]
fn showing_and_hiding() {
    let r = run("PRINT SHOWNP HT PRINT SHOWNP ST PRINT SHOWN?");
    assert_eq!(r.printed(), "true\nfalse\ntrue\n");
    assert!(r.events.contains(&Event::HideTurtle));
    assert!(r.events.contains(&Event::ShowTurtle));
}

#[test]
fn clean_keeps_the_turtle_where_it_is() {
    let r = run("FD 50 RT 90 CLEAN SHOW POS PRINT HEADING");
    assert!(r.events.contains(&Event::Clean));
    assert_eq!(r.printed(), "[0 50]\n90\n");
}

#[test]
fn clearscreen_goes_home_without_drawing() {
    let r = run("FD 50 RT 90 CS FD 10");
    let clean = r.events.iter().position(|e| *e == Event::Clean).unwrap();
    let after: Vec<_> = r.events[clean..]
        .iter()
        .filter(|e| matches!(e, Event::Move { .. }))
        .collect();
    let Event::Move { from, to, pen } = after[0] else {
        unreachable!()
    };
    assert_eq!(
        ((from.x, from.y), (to.x, to.y), pen.down),
        ((0.0, 50.0), (0.0, 0.0), false),
        "home with the pen up"
    );
    let Event::Move { pen, .. } = after[1] else {
        unreachable!()
    };
    assert!(pen.down, "and the pen is down again after");
    assert_eq!(
        output("FD 50 RT 90 CLEARSCREEN SHOW POS PRINT HEADING"),
        "[0 0]\n0\n"
    );
}

#[test]
fn errors() {
    assert_eq!(
        error("SETPOS \"abc"),
        "SETPOS needs a list of two numbers, but got the word abc"
    );
    assert_eq!(
        error("SETPOS [1]"),
        "SETPOS needs a list of two numbers, but got the list [1]"
    );
    assert_eq!(
        error("SETPOS [1 a]"),
        "SETPOS needs a list of two numbers, but got the list [1 a]"
    );
    assert_eq!(
        error("SETXY 1 \"a"),
        "SETXY needs a number, but got the word a"
    );
    assert_eq!(
        error("PRINT TOWARDS 5"),
        "TOWARDS needs a list of two numbers, but got 5"
    );
    assert_eq!(error("FD [10]"), "FD needs a number, but got the list [10]");
}
