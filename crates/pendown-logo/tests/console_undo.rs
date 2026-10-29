//! Undo at the console (ADR 0025): Kame comes back as the entry found it,
//! and variables and procedures stay.

mod common;

use std::sync::Arc;

use common::Shared;
use pendown_logo::{Event, Interpreter, Limits};

/// Everything about Kame and the background that Undo puts back.
const KAME: &str =
    "SHOW (LIST POS HEADING PENDOWNP PENCOLOR PENSIZE FILLCOLOR SHOWNP BACKGROUND TURTLEMODE)";

struct Console {
    it: Interpreter,
    events: Shared,
}

impl Console {
    fn new(program: &str) -> Console {
        let events = Shared::default();
        let mut it = Interpreter::new(Box::new(events.clone()), Arc::default());
        it.limits = Limits {
            instruction_events: false,
            ..Limits::default()
        };
        it.run_program(program).unwrap();
        Console { it, events }
    }

    /// Runs an entry, and returns what it printed.
    #[track_caller]
    fn enter(&mut self, text: &str) -> String {
        self.events.0.borrow_mut().clear();
        assert_eq!(self.it.run_console(text), Ok(()), "{text}");
        self.events
            .0
            .borrow()
            .iter()
            .filter_map(|e| match e {
                Event::Print(t) => Some(t.clone()),
                _ => None,
            })
            .collect()
    }

    fn undo(&mut self) {
        assert_eq!(self.it.undo_console(), Ok(()));
    }
}

#[test]
fn undo_puts_kame_back() {
    let mut c = Console::new("");
    let home = c.enter(KAME);
    c.enter("FD 100 RT 45 PU SETPC \"teal SETPENSIZE 3 SETFC \"orange HT SETBG 1 FENCE");
    let moved = c.enter(KAME);
    assert_ne!(moved, home);
    c.undo(); // the KAME query
    c.undo(); // the entry that changed everything
    assert_eq!(c.enter(KAME), home);
}

#[test]
fn undo_goes_back_one_entry_at_a_time() {
    let mut c = Console::new("");
    c.enter("FD 100");
    c.enter("RT 90");
    let before_the_oops = c.enter(KAME);
    c.undo(); // the KAME query
    c.enter("RT 45");
    c.undo();
    assert_eq!(c.enter(KAME), before_the_oops);
    c.undo(); // the KAME query
    c.undo();
    c.undo();
    assert_eq!(c.enter("SHOW (LIST POS HEADING)"), "[[0 0] 0]\n");
}

#[test]
fn an_entry_that_failed_can_be_undone() {
    let mut c = Console::new("");
    let home = c.enter(KAME);
    c.undo();
    // It moves, then stops with an error.
    assert!(c.it.run_console("FD 50 RT 30 FD").is_err());
    c.undo();
    assert_eq!(c.enter(KAME), home);
    // One that doesn't parse has a saved Kame too, so Undo stays in step.
    c.undo();
    c.enter("FD 10");
    assert!(c.it.run_console("FD (").is_err());
    c.undo();
    c.undo();
    assert_eq!(c.enter(KAME), home);
}

#[test]
fn undo_leaves_variables_and_procedures() {
    let mut c = Console::new("TO BOX :n\n REPEAT 4 [FD :n RT 90]\nEND");
    c.enter("MAKE \"size 30");
    c.enter("TO TWICE :n OUTPUT :n * 2 END");
    c.undo();
    c.undo();
    assert_eq!(c.enter("SHOW TWICE :size"), "60\n");
    c.enter("BOX 10");
    c.undo();
    assert_eq!(c.enter("SHOW (LIST POS HEADING)"), "[[0 0] 0]\n");
}

#[test]
fn undo_with_nothing_to_undo_changes_nothing() {
    let mut c = Console::new("FD 20 RT 10");
    c.undo();
    assert_eq!(c.enter("SHOW HEADING"), "10\n");
    // Run starts again with nothing to undo.
    c.enter("RT 50");
    c.it.run_program("FD 30").unwrap();
    c.undo();
    assert_eq!(c.enter("SHOW (LIST POS HEADING)"), "[[0 30] 0]\n");
}

#[test]
fn undo_sends_a_watch_with_the_colours() {
    let events = Shared::default();
    let mut it = Interpreter::new(Box::new(events.clone()), Arc::default());
    it.run_program("").unwrap();
    it.run_console("SETPC \"teal").unwrap();
    events.0.borrow_mut().clear();
    it.undo_console().unwrap();
    let events = events.0.borrow();
    let [Event::Watch(watch)] = &events[..] else {
        panic!("expected one Watch, got {} events", events.len())
    };
    let pen = watch.colours.iter().find(|c| c.name == "PENCOLOR").unwrap();
    assert_eq!(pen.value, "7");
}
