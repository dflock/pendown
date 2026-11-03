//! The Pendown Logo interpreter. It has no GUI code: it runs a program and
//! sends a stream of turtle events to whoever is listening (ADR 0009).
//!
//! The pipeline follows UCBLogo:
//!
//! 1. [`reader`] splits source text into words and bracketed lists, keeping a
//!    span for every word (ADR 0001).
//! 2. [`parse`] splits words into tokens (`runparse` in UCBLogo) and groups
//!    them into instructions, using the number of inputs each procedure takes.
//!    Lists are only parsed into instructions when they're first run.
//! 3. [`interp`] walks the instructions, eliminating tail calls (ADR 0007).

mod colour;
mod error;
mod event;
mod geometry;
mod interp;
pub mod metrics;
mod names;
mod number;
mod parse;
mod prims;
mod random;
mod reader;
pub mod reference;
mod span;
pub mod syntax;
mod value;
pub mod watch;

pub use colour::{NAMES as COLOUR_NAMES, PALETTE, Rgba};
pub use error::{CallSite, Error, RunError};
pub use event::{BatchSender, Event, EventSink, Halted, Pen, Point, SyncPoint};
pub use geometry::{Area, TurtleMode, arc_center, arc_point};
pub use interp::{Control, Interpreter, Limits, STACK_MARGIN, Stats};
pub use number::{format_number, parse_number};
pub use span::{SourceId, Span};
pub use value::Value;
pub use watch::Watch;

/// A seed for `RANDOM` that's different every time, for the GUI to pick for
/// each Run and keep, so it can run the program again with the same seed.
pub fn fresh_seed() -> u64 {
    random::Random::fresh_seed()
}

/// Every built-in command's names, the first being the one the manual uses
/// first, for checking against `content/reference.kdl`.
pub fn command_names() -> impl Iterator<Item = &'static [&'static str]> {
    prims::PRIMS.iter().map(|p| p.names)
}

/// The special variables, which change how Pendown behaves.
pub fn special_variables() -> &'static [&'static str] {
    interp::SPECIAL_VARIABLES
}
