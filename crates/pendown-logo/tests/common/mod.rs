//! Helpers shared by the command tests: run a program and look at what it
//! printed, drew or got wrong.

#![allow(dead_code)]

use std::{cell::RefCell, rc::Rc, sync::Arc};

use pendown_logo::{Error, Event, EventSink, Halted, Interpreter, Limits, Point, RunError};

#[derive(Clone, Default)]
pub struct Shared(pub Rc<RefCell<Vec<Event>>>);

impl EventSink for Shared {
    fn send(&mut self, event: Event) -> Result<(), Halted> {
        self.0.borrow_mut().push(event);
        Ok(())
    }
}

/// A move: where from, where to, and whether the pen was down.
pub type Move = ((f64, f64), (f64, f64), bool);

pub struct Run {
    pub result: Result<(), RunError>,
    pub events: Vec<Event>,
}

impl Run {
    pub fn printed(&self) -> String {
        self.events
            .iter()
            .filter_map(|e| {
                if let Event::Print(t) = e {
                    Some(t.as_str())
                } else {
                    None
                }
            })
            .collect()
    }

    /// The moves, as ((from), (to), pen down).
    pub fn moves(&self) -> Vec<Move> {
        self.events
            .iter()
            .filter_map(|e| match e {
                Event::Move { from, to, pen } => Some(((from.x, from.y), (to.x, to.y), pen.down)),
                _ => None,
            })
            .collect()
    }

    pub fn turns(&self) -> Vec<(f64, f64)> {
        self.events
            .iter()
            .filter_map(|e| {
                if let Event::Turn { from, by } = e {
                    Some((*from, *by))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn error(&self) -> &Error {
        match &self.result {
            Err(RunError::Runtime(e)) => e,
            other => panic!("expected a runtime error, got {other:?}"),
        }
    }
}

pub fn run(src: &str) -> Run {
    let events = Shared::default();
    let mut it = Interpreter::new(Box::new(events.clone()), Arc::default());
    it.limits = Limits {
        instruction_events: false,
        ..Limits::default()
    };
    let result = it.run_program(src);
    let events = events.0.borrow().clone();
    Run { result, events }
}

/// What `src` prints, which must run without errors.
#[track_caller]
pub fn output(src: &str) -> String {
    let r = run(src);
    assert_eq!(r.result, Ok(()), "{src}");
    r.printed()
}

/// The runtime error `src` stops with.
#[track_caller]
pub fn error(src: &str) -> String {
    run(src).error().message.clone()
}

pub fn p(x: f64, y: f64) -> Point {
    Point { x, y }
}
