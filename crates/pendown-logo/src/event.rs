use std::{
    fmt,
    sync::mpsc::SyncSender,
    time::{Duration, Instant},
};

use crate::span::Span;

/// A point in turtle coordinates: y points up, and the turtle starts at 0, 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pen {
    pub down: bool,
    /// Resolved RGBA, from 0 to 255.
    pub color: [u8; 4],
    pub size: f64,
}

/// What the interpreter tells its front end (ADR 0009). More events arrive
/// with the commands that need them.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// Sent before each instruction runs. `starts_line` is set on the first
    /// instruction run from a line, and is where Step pauses.
    Instruction { span: Span, starts_line: bool },
    Move { from: Point, to: Point, pen: Pen },
    /// `by` is the signed angle turned, clockwise, in degrees.
    Turn { from: f64, by: f64 },
    /// Console output, including the newline if there is one.
    Print(String),
    /// The interpreter waits here until the front end replies (ADR 0011).
    /// In lockstep, one follows each `Instruction` that starts a line, so
    /// the program can't run a line before the animator gets to it.
    Sync(SyncPoint),
}

/// The reply half of a `Sync`. Replying lets the interpreter go on; dropping
/// it without replying stops the program.
#[derive(Clone)]
pub struct SyncPoint(pub(crate) SyncSender<()>);

impl SyncPoint {
    pub fn reply(self) {
        let _ = self.0.send(());
    }
}

impl PartialEq for SyncPoint {
    fn eq(&self, _: &SyncPoint) -> bool {
        true
    }
}

impl fmt::Debug for SyncPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("SyncPoint")
    }
}

/// The receiver has gone away, so the program should stop.
#[derive(Debug)]
pub struct Halted;

/// Where events go: a channel to the GUI, or a list for the CLI and tests.
pub trait EventSink {
    fn send(&mut self, event: Event) -> Result<(), Halted>;

    /// Sends anything held back. Called when a run ends.
    fn flush(&mut self) -> Result<(), Halted> {
        Ok(())
    }
}

/// A bounded channel blocks the interpreter when it's full, which keeps it
/// in step with the animator. Dropping the receiver wakes it up to stop.
impl EventSink for SyncSender<Event> {
    fn send(&mut self, event: Event) -> Result<(), Halted> {
        SyncSender::send(self, event).map_err(|_| Halted)
    }
}

impl EventSink for Vec<Event> {
    fn send(&mut self, event: Event) -> Result<(), Halted> {
        self.push(event);
        Ok(())
    }
}

/// Sends events in batches, which costs far less per event than one channel
/// send each. A batch goes when it's full, when it's been open for
/// `max_wait`, and when the run ends, so a slow program's events still
/// arrive promptly.
pub struct BatchSender {
    tx: SyncSender<Vec<Event>>,
    batch: Vec<Event>,
    size: usize,
    opened: Instant,
    max_wait: Duration,
}

impl BatchSender {
    pub fn new(tx: SyncSender<Vec<Event>>, size: usize, max_wait: Duration) -> BatchSender {
        BatchSender { tx, batch: Vec::with_capacity(size), size, opened: Instant::now(), max_wait }
    }
}

impl EventSink for BatchSender {
    fn send(&mut self, event: Event) -> Result<(), Halted> {
        if self.batch.is_empty() {
            self.opened = Instant::now();
        }
        self.batch.push(event);
        // Checking the clock costs about as much as an event, so only every
        // 16 events.
        if self.batch.len() >= self.size || (self.batch.len().is_multiple_of(16) && self.opened.elapsed() >= self.max_wait) {
            self.flush()?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Halted> {
        if self.batch.is_empty() {
            return Ok(());
        }
        let batch = std::mem::replace(&mut self.batch, Vec::with_capacity(self.size));
        self.tx.send(batch).map_err(|_| Halted)
    }
}
