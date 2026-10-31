//! The interpreter's thread (ADR 0011). One interpreter lives for the whole
//! session, so the console can use whatever the last run left (ADR 0006):
//! it takes jobs one at a time, each with its own channel and `Control`, so
//! each can be stopped on its own.

use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};

use pendown_logo::{BatchSender, Control, Event, EventSink, Halted, Interpreter, RunError};

use crate::animator::{Animator, Replay, Speed};

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

/// Batches of 256 events, with room for 4 batches (spikes 03 and 04).
pub const DEFAULT_CHANNEL: Channel = Channel {
    batch: 256,
    capacity: 4,
    max_wait: Duration::from_millis(4),
};

/// What the interpreter runs.
#[derive(Clone, Debug)]
pub enum Job {
    /// The Run button: a clean slate, then the whole program, with `seed`
    /// for `RANDOM`.
    Program { text: String, seed: u64 },
    /// A console entry, against what the last run left.
    Console(String),
    /// Undo at the console (ADR 0025): Kame back as the last entry found it.
    Undo,
    /// Defines Teach Kame's procedure at the console, without saving Kame
    /// for Undo, since it isn't one of the user's entries.
    Define(String),
    /// Reset (ADR 0028): the clean slate a program starts from, with
    /// nothing run.
    Reset,
}

/// One job's progress, as the GUI sees it.
pub struct Run {
    pub control: Arc<Control>,
    pub started: Instant,
    /// The seed `RANDOM` started from, for a program: running it again with
    /// this seed does the same.
    pub seed: Option<u64>,
    /// How the job ended, once it has.
    pub result: Arc<Mutex<Option<Result<(), RunError>>>>,
    /// When it ended, for measuring how long Stop takes.
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

struct Command {
    job: Job,
    events: mpsc::SyncSender<Vec<Event>>,
    channel: Channel,
    control: Arc<Control>,
    result: Arc<Mutex<Option<Result<(), RunError>>>>,
    exited: Arc<Mutex<Option<Instant>>>,
}

/// Throws events away, between jobs.
struct Discard;

impl EventSink for Discard {
    fn send(&mut self, _: Event) -> Result<(), Halted> {
        Err(Halted)
    }
}

/// The interpreter's thread, waiting for jobs.
pub struct Session {
    commands: mpsc::Sender<Command>,
}

impl Default for Session {
    fn default() -> Session {
        Session::new()
    }
}

impl Session {
    pub fn new() -> Session {
        let (commands, rx) = mpsc::channel::<Command>();
        thread::Builder::new()
            .name("interpreter".into())
            .stack_size(STACK)
            .spawn(move || {
                let mut it = Interpreter::new(Box::new(Discard), Arc::default());
                it.limits.stack_bytes = Some(STACK);
                // Jobs run one at a time, until the session is dropped.
                for command in rx {
                    let sink = BatchSender::new(
                        command.events,
                        command.channel.batch,
                        command.channel.max_wait,
                    )
                    .measured(command.control.clone());
                    it.set_output(Box::new(sink), command.control);
                    let result = match &command.job {
                        Job::Program { text, seed } => {
                            it.set_seed(Some(*seed));
                            it.run_program(text)
                        }
                        Job::Console(text) => it.run_console(text),
                        Job::Undo => it.undo_console(),
                        Job::Define(text) => it.define_console(text),
                        Job::Reset => {
                            it.reset();
                            Ok(())
                        }
                    };
                    // The result first: dropping the sender tells the
                    // animator the job is over, and the window reads the
                    // result as soon as it sees that.
                    *command.result.lock().unwrap() = Some(result);
                    it.set_output(Box::new(Discard), Arc::default());
                    *command.exited.lock().unwrap() = Some(Instant::now());
                }
            })
            .expect("can start the interpreter thread");
        Session { commands }
    }

    /// Starts a job, and returns the animator that plays its events. A job
    /// started while another runs waits for it, so stop that one first.
    pub fn start(&self, job: Job, speed: Speed, channel: Channel) -> (Animator, Run) {
        self.start_with(job, speed, channel, None, &[])
    }

    /// Starts a job that replays out of sight up to a point first, if
    /// `replay` says so (ADR 0028), and stops at the editor's lines in
    /// `breakpoints`, counting from 0 (ADR 0029). Both are set before the
    /// job starts.
    pub fn start_with(
        &self,
        job: Job,
        speed: Speed,
        channel: Channel,
        replay: Option<Replay>,
        breakpoints: &[u32],
    ) -> (Animator, Run) {
        let control = Arc::new(Control::default());
        control.set_breakpoints(breakpoints.iter().copied());
        let (events, rx) = mpsc::sync_channel::<Vec<Event>>(channel.capacity);
        let run = Run {
            control: control.clone(),
            started: Instant::now(),
            seed: match &job {
                Job::Program { seed, .. } => Some(*seed),
                _ => None,
            },
            result: Arc::default(),
            exited: Arc::default(),
        };
        // The animator sets the lockstep flag and the speed before the job starts.
        let animator = Animator::replaying(rx, control.clone(), speed, replay);
        let command = Command {
            job,
            events,
            channel,
            control,
            result: run.result.clone(),
            exited: run.exited.clone(),
        };
        self.commands
            .send(command)
            .expect("the interpreter thread is running");
        (animator, run)
    }

    /// Puts the interpreter back to how a run starts, without running
    /// anything (ADR 0028). Nothing plays it, so it sends no events.
    pub fn reset(&self) {
        let (events, _) = mpsc::sync_channel::<Vec<Event>>(1);
        let command = Command {
            job: Job::Reset,
            events,
            channel: DEFAULT_CHANNEL,
            control: Arc::default(),
            result: Arc::default(),
            exited: Arc::default(),
        };
        self.commands
            .send(command)
            .expect("the interpreter thread is running");
    }
}

/// Runs one program on a session of its own, with a fresh seed.
pub fn start(source: String, speed: Speed, channel: Channel) -> (Animator, Run) {
    let job = Job::Program {
        text: source,
        seed: pendown_logo::fresh_seed(),
    };
    Session::new().start(job, speed, channel)
}
