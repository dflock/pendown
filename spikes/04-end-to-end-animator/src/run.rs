//! Starting a program on its own thread (ADR 0011).

use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};

use pendown_logo::{BatchSender, Control, Event, Interpreter, RunError};

use crate::animator::{Animator, Speed};

/// The interpreter's stack (ADR 0007).
pub const STACK: usize = 256 * 1024 * 1024;

/// How events travel to the animator: in batches of up to `batch` events, at
/// most `max_wait` old, over a channel holding `capacity` batches.
#[derive(Clone, Copy, Debug)]
pub struct Channel {
    pub batch: usize,
    pub capacity: usize,
    pub max_wait: Duration,
}

/// What spike 03 recommended.
pub const DEFAULT_CHANNEL: Channel = Channel { batch: 256, capacity: 4, max_wait: Duration::from_millis(4) };

/// The interpreter's side of a run, as the GUI sees it.
pub struct Run {
    pub control: Arc<Control>,
    pub started: Instant,
    /// How the program ended, once the thread finishes.
    pub result: Arc<Mutex<Option<Result<(), RunError>>>>,
    /// When the thread finished, for measuring how long Stop takes.
    pub exited: Arc<Mutex<Option<Instant>>>,
}

impl Run {
    pub fn result(&self) -> Option<Result<(), RunError>> {
        self.result.lock().unwrap().clone()
    }

    pub fn exited(&self) -> Option<Instant> {
        *self.exited.lock().unwrap()
    }
}

/// Starts `source` on a new interpreter thread, and returns the animator
/// that plays its events.
pub fn start(source: String, speed: Speed, channel: Channel) -> (Animator, Run) {
    let control = Arc::new(Control::default());
    let (tx, rx) = mpsc::sync_channel::<Vec<Event>>(channel.capacity);
    let run = Run {
        control: control.clone(),
        started: Instant::now(),
        result: Arc::default(),
        exited: Arc::default(),
    };
    // The animator sets the lockstep flag before the program starts.
    let animator = Animator::new(rx, control.clone(), speed);
    let (result, exited) = (run.result.clone(), run.exited.clone());
    thread::Builder::new()
        .name("interpreter".into())
        .stack_size(STACK)
        .spawn(move || {
            let sink = BatchSender::new(tx, channel.batch, channel.max_wait);
            let mut it = Interpreter::new(Box::new(sink), control);
            it.limits.stack_bytes = Some(STACK);
            let r = it.run_program(&source);
            *result.lock().unwrap() = Some(r);
            *exited.lock().unwrap() = Some(Instant::now());
        })
        .expect("can start the interpreter thread");
    (animator, run)
}
