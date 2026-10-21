//! The pen, colours and background (phase 1 issue 07).

mod common;

use std::sync::Arc;

use common::*;
use pendown_logo::{Event, Interpreter};

fn pens(r: &Run) -> Vec<([u8; 4], f64, bool)> {
    r.events
        .iter()
        .filter_map(|e| if let Event::Move { pen, .. } = e { Some((pen.color, pen.size, pen.down)) } else { None })
        .collect()
}

#[test]
fn defaults_are_ucblogos() {
    assert_eq!(output("PRINT PC PRINT BG SHOW PENSIZE PRINT PENDOWNP"), "7\n0\n[1 1]\ntrue\n");
    assert_eq!(pens(&run("FD 10")), [([255, 255, 255, 255], 1.0, true)]);
}

#[test]
fn every_colour_form_resolves_in_the_events() {
    let r = run("SETPC 4 FD 1 SETPC [1 2 3] FD 1 SETPC [1 2 3 4] FD 1 SETPC \"Gold FD 1 SETPC \"#f80 FD 1 SETPC \"#ff880080 FD 1");
    let colours: Vec<_> = pens(&r).into_iter().map(|(c, ..)| c).collect();
    assert_eq!(
        colours,
        [[255, 0, 0, 255], [1, 2, 3, 255], [1, 2, 3, 4], [255, 215, 0, 255], [255, 136, 0, 255], [255, 136, 0, 128]]
    );
}

#[test]
fn reporters_give_the_form_it_was_set_in() {
    let cases = [("4", "4"), ("[1 2 3]", "[1 2 3]"), ("[1 2 3 4]", "[1 2 3 4]"), ("\"Gold", "Gold"), ("\"#f80", "#f80")];
    for (input, shown) in cases {
        assert_eq!(output(&format!("SETPC {input} SHOW PC")), format!("{shown}\n"), "{input}");
        assert_eq!(output(&format!("SETBG {input} SHOW BG")), format!("{shown}\n"), "{input}");
    }
    // So SETPC PC always works.
    assert_eq!(pens(&run("SETPC \"#ff880080 SETPC PC FD 1"))[0].0, [255, 136, 0, 128]);
}

#[test]
fn pen_up_and_down() {
    let r = run("PU FD 10 PRINT PENDOWNP PD FD 10 PRINT PENDOWN?");
    assert_eq!(r.printed(), "false\ntrue\n");
    assert_eq!(pens(&r).iter().map(|p| p.2).collect::<Vec<_>>(), [false, true]);
}

#[test]
fn pen_size() {
    assert_eq!(pens(&run("SETPENSIZE 5 FD 1 SETPENSIZE [3 7] FD 1 SETPENSIZE 0.5 FD 1")).iter().map(|p| p.1).collect::<Vec<_>>(), [5.0, 3.0, 0.5]);
    assert_eq!(output("SETPENSIZE [3 7] SHOW PENSIZE"), "[3 3]\n", "the pen is square");
}

#[test]
fn the_background_is_solid() {
    let r = run("SETBG [0 0 255 100] SHOW BG");
    assert!(r.events.contains(&Event::SetBackground { color: [0, 0, 255, 255] }));
    assert_eq!(r.printed(), "[0 0 255 100]\n");
}

#[test]
fn run_resets_the_pen_and_background() {
    let events = Shared::default();
    let mut it = Interpreter::new(Box::new(events.clone()), Arc::default());
    it.run_program("SETPC 4 SETBG 1 SETPENSIZE 9 PU").unwrap();
    it.run_program("PRINT PC PRINT BG SHOW PENSIZE PRINT PENDOWNP").unwrap();
    let printed: String = events.0.borrow().iter().filter_map(|e| if let Event::Print(t) = e { Some(t.clone()) } else { None }).collect();
    assert_eq!(printed, "7\n0\n[1 1]\ntrue\n");
}

#[test]
fn colour_errors() {
    assert_eq!(error("SETPC \"bleu"), "I don't know the colour \"bleu\". Did you mean \"blue\"?");
    assert_eq!(error("SETPC 16"), "SETPC doesn't like 16 as input");
    assert_eq!(error("SETPC 1.5"), "SETPC doesn't like 1.5 as input");
    assert_eq!(error("SETPENCOLOR [1 2]"), "SETPENCOLOR doesn't like [1 2] as input");
    assert_eq!(error("SETPC [1 2 256]"), "SETPC doesn't like [1 2 256] as input");
    assert_eq!(error("SETBG \"#12"), "\"#12\" isn't a hex colour: use 3, 4, 6 or 8 hex digits after the #, as in \"#ff8800");
    assert_eq!(error("SETPENSIZE 0"), "SETPENSIZE doesn't like 0 as input");
    assert_eq!(error("SETPENSIZE \"thick"), "SETPENSIZE doesn't like thick as input");
}

#[test]
fn the_palette() {
    assert_eq!(output("SHOW PALETTE 4"), "[255 0 0]\n");
    assert_eq!(output("SETPALETTE 8 [10 20 30] SHOW PALETTE 8"), "[10 20 30]\n");
    assert_eq!(output("SETPALETTE 9 \"#ff000080 SHOW PALETTE 9"), "[255 0 0 128]\n");
    let r = run("SETPALETTE 8 \"red SETPC 8 FD 10");
    assert_eq!(pens(&r), [([255, 0, 0, 255], 1.0, true)]);
    // Run puts the palette back.
    assert_eq!(output("SHOW PALETTE 8"), "[155 96 59]\n");
    assert_eq!(error("SETPALETTE 7 \"red"), "SETPALETTE doesn't like 7 as input");
    assert_eq!(error("SETPALETTE 8 4"), "SETPALETTE doesn't like 4 as input");
    assert_eq!(error("SHOW PALETTE 16"), "PALETTE doesn't like 16 as input");
}

#[test]
fn fill_colours() {
    use pendown_logo::Event;
    let fills = |r: &Run| -> Vec<Option<[u8; 4]>> {
        r.events
            .iter()
            .filter_map(|e| match e {
                Event::Ellipse { fill, .. } | Event::Rect { fill, .. } => Some(*fill),
                _ => None,
            })
            .collect()
    };
    let r = run("CIRCLE 10 SETFC \"orange SQUARE 10 SETFILLCOLOR [1 2 3 4] CIRCLE 5 SETFC \"none CIRCLE 5");
    assert_eq!(fills(&r), [None, Some([255, 165, 0, 255]), Some([1, 2, 3, 4]), None]);
    assert_eq!(output("SHOW FILLCOLOR SETFC \"orange SHOW FC SETFC 4 SHOW FC"), "none\norange\n4\n");
    assert_eq!(error("SETFC \"nosuch"), "I don't know the colour \"nosuch\"");
}
