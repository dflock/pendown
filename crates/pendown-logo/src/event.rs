use std::{
    fmt,
    sync::{Arc, mpsc::SyncSender},
    time::{Duration, Instant},
};

use crate::{interp::Control, span::Span, watch::Watch};

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
    Instruction {
        span: Span,
        starts_line: bool,
    },
    Move {
        from: Point,
        to: Point,
        pen: Pen,
    },
    /// `by` is the signed angle turned, clockwise, in degrees.
    Turn {
        from: f64,
        by: f64,
    },
    /// The turtle walks along an arc (ADR 0016), starting at `start_heading`
    /// and turning through `angle` degrees, clockwise if positive, around
    /// `center`. See `arc_point` for where it is part-way along.
    Arc {
        center: Point,
        radius: f64,
        start_heading: f64,
        angle: f64,
        pen: Pen,
    },
    /// `CIRCLE` and `ELLIPSE` (ADR 0017): `radii` are across and along the
    /// heading. The turtle doesn't move.
    Ellipse {
        center: Point,
        radii: (f64, f64),
        heading: f64,
        pen: Pen,
        fill: Option<[u8; 4]>,
    },
    /// `SQUARE` and `RECTANGLE`: `size` is the full width across and length
    /// along the heading, and `corner` the radius of the rounded corners.
    Rect {
        center: Point,
        size: (f64, f64),
        corner: f64,
        heading: f64,
        pen: Pen,
        fill: Option<[u8; 4]>,
    },
    ShowTurtle,
    HideTurtle,
    /// Clear the drawing. The turtle doesn't move.
    Clean,
    /// A new background colour, always opaque. It goes under everything
    /// already drawn.
    SetBackground {
        color: [u8; 4],
    },
    /// Console output, including the newline if there is one.
    Print(String),
    /// `WAIT`: the animator waits this long at every speed, including
    /// Instant (ADR 0011).
    Wait(std::time::Duration),
    /// `SETSPEED`: the animator changes to this level, from 1 to 11, when it
    /// gets here.
    SetSpeed(u8),
    /// The interpreter waits here until the front end replies (ADR 0011).
    /// In lockstep, one follows each `Instruction` that starts a line, so
    /// the program can't run a line before the animator gets to it.
    Sync(SyncPoint),
    /// What the counters and variables panels show, sent when it changes
    /// (ADR 0022): just before the `Instruction` that sees it, and at the
    /// end of a run.
    Watch(Arc<Watch>),
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
    /// Where to record time spent waiting on a full channel.
    control: Option<Arc<Control>>,
}

impl BatchSender {
    pub fn new(tx: SyncSender<Vec<Event>>, size: usize, max_wait: Duration) -> BatchSender {
        BatchSender {
            tx,
            batch: Vec::with_capacity(size),
            size,
            opened: Instant::now(),
            max_wait,
            control: None,
        }
    }

    /// Records the time spent waiting on a full channel in `control`'s
    /// metrics.
    pub fn measured(self, control: Arc<Control>) -> BatchSender {
        BatchSender {
            control: Some(control),
            ..self
        }
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
        if self.batch.len() >= self.size
            || (self.batch.len().is_multiple_of(16) && self.opened.elapsed() >= self.max_wait)
        {
            self.flush()?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Halted> {
        if self.batch.is_empty() {
            return Ok(());
        }
        let batch = std::mem::replace(&mut self.batch, Vec::with_capacity(self.size));
        let waiting = self
            .control
            .as_ref()
            .and_then(|c| c.metrics.channel.start());
        let sent = self.tx.send(batch).map_err(|_| Halted);
        if let Some(control) = &self.control {
            control.metrics.channel.stop(waiting);
        }
        sent
    }
}
