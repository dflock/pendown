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
mod names;
mod number;
mod parse;
mod prims;
mod random;
mod reader;
mod span;
mod value;

pub use colour::{NAMES as COLOUR_NAMES, PALETTE, Rgba};
pub use error::{CallSite, Error, RunError};
pub use geometry::{Area, TurtleMode, arc_center, arc_point};
pub use event::{BatchSender, Event, EventSink, Halted, Pen, Point, SyncPoint};
pub use interp::{Control, Interpreter, Limits, STACK_MARGIN, Stats};
pub use number::{format_number, parse_number};
pub use span::{SourceId, Span};
pub use value::Value;
