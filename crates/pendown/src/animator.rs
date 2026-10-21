//! The animator (ADR 0011): plays the interpreter's events at the chosen
//! speed, and implements Pause, Step and Stop (ADR 0012). It has no gpui
//! code, so `tests/animator.rs` tests it with a real interpreter thread and
//! no window.

use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::Ordering,
        mpsc::{Receiver, RecvTimeoutError, TryRecvError},
    },
    time::{Duration, Instant},
};

use pendown_logo::{Control, Event, Pen, Point, Span, SyncPoint};
use pendown_render::Segment;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Speed {
    Instant,
    /// Turtle steps per second. Turning runs at `DEGREES_PER_STEP` degrees
    /// for each step, so one setting controls both.
    Animated(f64),
}

/// How fast the turtle turns compared with how fast it moves: at 100 steps
/// a second, `RT 90` takes 0.3 s and `FD 50` takes 0.5 s.
pub const DEGREES_PER_STEP: f64 = 3.0;

/// How long the highlight stays on an instruction that doesn't move the
/// turtle, such as `MAKE` or a procedure call, measured in steps: at 100
/// steps a second it's 0.1 s, and it's never more than `MAX_DWELL`.
pub const DWELL_STEPS: f64 = 10.0;
pub const MAX_DWELL: f64 = 0.25;

/// Instant mode takes events for at most this long each frame.
pub const INSTANT_SLICE: Duration = Duration::from_millis(8);

/// After replying to a `Sync`, the animator waits up to this long in a frame
/// for the interpreter to send the next line's events, so lockstep doesn't
/// limit the program to one line a frame.
pub const SYNC_WAIT: Duration = Duration::from_millis(4);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Running,
    Paused,
    /// Run to the start of the next line, then pause.
    StepLine,
    /// Run to the next instruction, then pause.
    StepInstruction,
}


enum Current {
    Move { from: Point, to: Point, pen: Pen, done: f64 },
    Turn { from: f64, by: f64, done: f64 },
    /// Keeps the highlight on an instruction that drew nothing.
    Dwell { left: f64 },
}

/// What a frame produced.
#[derive(Default)]
pub struct Frame {
    /// Moves finished this frame, to draw into the pixel buffer.
    pub segments: Vec<Segment>,
    /// A `CLEAN` happened this frame: clear the drawing before adding
    /// `segments`, which only holds the moves since the last one.
    pub cleaned: bool,
    /// The background changed this frame, to this. It goes under
    /// everything, so it doesn't matter where among `segments` it came.
    pub background: Option<[u8; 4]>,
    pub printed: Vec<String>,
    pub highlight_changed: bool,
    /// Events taken this frame.
    pub events: usize,
}

pub struct Animator {
    rx: Option<Receiver<Vec<Event>>>,
    queue: VecDeque<Event>,
    current: Option<Current>,
    /// A `Sync` the animator is holding: the interpreter waits until it's
    /// released.
    held: Option<SyncPoint>,
    mode: Mode,
    speed: Speed,
    control: Arc<Control>,
    /// How long Instant mode takes events for each frame.
    pub instant_slice: Duration,
    pub pos: Point,
    pub heading: f64,
    /// Whether the turtle sprite is shown.
    pub shown: bool,
    pub highlight: Option<Span>,
    /// The last instruction hasn't moved or turned the turtle yet.
    idle_instruction: bool,
    /// The last event taken was an `Instruction` that starts a line, so a
    /// `Sync` straight after it belongs to that line.
    after_line_start: bool,
    /// Instructions shown since the current step began.
    shown_this_step: u64,
    /// Line starts shown, to compare with `Control::lines_started`.
    pub lines_shown: u64,
    pub instructions_shown: u64,
    /// The interpreter has finished, and every event has been taken.
    disconnected: bool,
    pub stopped: bool,
}

impl Animator {
    pub fn new(rx: Receiver<Vec<Event>>, control: Arc<Control>, speed: Speed) -> Animator {
        let animator = Animator {
            rx: Some(rx),
            queue: VecDeque::new(),
            current: None,
            held: None,
            mode: Mode::Running,
            speed,
            control,
            instant_slice: INSTANT_SLICE,
            pos: Point { x: 0.0, y: 0.0 },
            heading: 0.0,
            shown: true,
            highlight: None,
            idle_instruction: false,
            after_line_start: false,
            shown_this_step: 0,
            lines_shown: 0,
            instructions_shown: 0,
            disconnected: false,
            stopped: false,
        };
        animator.update_lockstep();
        animator
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn speed(&self) -> Speed {
        self.speed
    }

    /// The interpreter has finished and everything it sent has been played.
    pub fn finished(&self) -> bool {
        self.stopped || (self.disconnected && self.queue.is_empty() && self.current.is_none())
    }

    /// Whether the window needs another frame.
    pub fn wants_frames(&self) -> bool {
        !self.finished() && matches!(self.mode, Mode::Running | Mode::StepLine | Mode::StepInstruction)
    }

    /// Whether a move, turn or dwell is part-way through.
    pub fn in_progress(&self) -> bool {
        self.current.is_some()
    }

    /// The move in progress, from its start to where the turtle is now.
    pub fn partial(&self) -> Option<Segment> {
        match &self.current {
            Some(Current::Move { from, pen, .. }) if pen.down => Some(Segment { from: *from, to: self.pos, pen: *pen }),
            _ => None,
        }
    }

    /// How many lines the interpreter has started.
    pub fn lines_started(&self) -> u64 {
        self.control.lines_started.load(Ordering::Relaxed)
    }

    /// How many lines the interpreter has started that the highlight hasn't
    /// reached yet.
    pub fn lines_ahead(&self) -> i64 {
        self.lines_started() as i64 - self.lines_shown as i64
    }

    /// The interpreter waits at each line while paused or stepping, and at
    /// animated speeds, so it never runs a line before the highlight gets
    /// there. Only Instant, running freely, lets it run ahead.
    fn update_lockstep(&self) {
        let free = self.mode == Mode::Running && self.speed == Speed::Instant;
        self.control.lockstep.store(!free, Ordering::Relaxed);
    }

    pub fn set_speed(&mut self, speed: Speed) {
        self.speed = speed;
        self.update_lockstep();
    }

    pub fn pause(&mut self) {
        self.mode = Mode::Paused;
        self.update_lockstep();
    }

    pub fn resume(&mut self) {
        self.mode = Mode::Running;
        self.update_lockstep();
        if let Some(sync) = self.held.take() {
            sync.reply();
        }
    }

    pub fn step_line(&mut self) {
        self.begin_step(Mode::StepLine);
    }

    pub fn step_instruction(&mut self) {
        self.begin_step(Mode::StepInstruction);
    }

    fn begin_step(&mut self, mode: Mode) {
        self.mode = mode;
        self.shown_this_step = 0;
        self.update_lockstep();
        if let Some(sync) = self.held.take() {
            sync.reply();
        }
    }

    /// Stop (ADR 0011): set the cancel flag, and drop the channel and any
    /// held `Sync`, which wakes the interpreter wherever it's blocked.
    pub fn stop(&mut self) {
        self.control.cancel.store(true, Ordering::Relaxed);
        self.rx = None;
        self.queue.clear();
        self.held = None;
        self.current = None;
        self.stopped = true;
    }

    /// Plays one frame's worth of events. `dt` is the time since the last
    /// frame.
    pub fn frame(&mut self, dt: Duration, max_segments: usize) -> Frame {
        let started = Instant::now();
        let mut out = Frame::default();
        let instant = self.speed == Speed::Instant;
        let slice_end = started + self.instant_slice;
        let sync_wait_end = started + SYNC_WAIT;
        // Animation time left this frame, in seconds. A long frame (the
        // window was hidden, say) doesn't make the turtle jump.
        let mut budget = dt.as_secs_f64().min(0.1);
        let mut replied = false;

        loop {
            if !self.is_playing() {
                break;
            }
            if let Some(current) = self.current.take()
                && !self.advance(current, &mut budget, &mut out)
            {
                break;
            }
            if instant && (Instant::now() >= slice_end || out.segments.len() >= max_segments) {
                break;
            }
            // Instant mode waits for events until its slice is used up: the
            // channel only holds a few batches, so taking what's there and
            // stopping would limit it to a channel's worth a frame. After a
            // reply to a Sync, the next line's events are about to arrive.
            let wait_until = if instant {
                Some(slice_end)
            } else {
                replied.then_some(sync_wait_end)
            };
            let Some(event) = self.next_event(wait_until) else { break };
            out.events += 1;
            match event {
                Event::Instruction { span, starts_line } => {
                    if self.idle_instruction && !instant && self.mode != Mode::StepInstruction {
                        // Show the instruction that did nothing for a moment,
                        // then come back to this one.
                        self.idle_instruction = false;
                        self.queue.push_front(Event::Instruction { span, starts_line });
                        out.events -= 1;
                        self.current = Some(Current::Dwell { left: self.dwell() });
                        continue;
                    }
                    self.highlight = Some(span);
                    out.highlight_changed = true;
                    self.idle_instruction = true;
                    self.after_line_start = starts_line;
                    self.instructions_shown += 1;
                    self.shown_this_step += 1;
                    if starts_line {
                        self.lines_shown += 1;
                    }
                    if self.mode == Mode::StepInstruction {
                        self.mode = Mode::Paused;
                        self.update_lockstep();
                    }
                    continue;
                }
                Event::Move { from, to, pen } => {
                    self.idle_instruction = false;
                    self.current = Some(Current::Move { from, to, pen, done: 0.0 });
                }
                Event::Turn { from, by } => {
                    self.idle_instruction = false;
                    self.current = Some(Current::Turn { from, by, done: 0.0 });
                }
                Event::Print(text) => out.printed.push(text),
                Event::ShowTurtle => self.shown = true,
                Event::HideTurtle => self.shown = false,
                Event::Clean => {
                    out.segments.clear();
                    out.cleaned = true;
                }
                Event::SetBackground { color } => out.background = Some(color),
                Event::Sync(sync) => {
                    // A Sync straight after the line start the step began
                    // on belongs to that line, so Step runs it. Any other
                    // Sync is the start of the next line.
                    let own_line = self.after_line_start && self.shown_this_step == 0;
                    if self.mode == Mode::StepLine && !own_line {
                        self.held = Some(sync);
                        self.mode = Mode::Paused;
                        self.update_lockstep();
                    } else {
                        sync.reply();
                        replied = true;
                    }
                }
            }
            self.after_line_start = false;
        }
        out
    }

    fn is_playing(&self) -> bool {
        !self.stopped && self.mode != Mode::Paused
    }

    fn dwell(&self) -> f64 {
        match self.speed {
            Speed::Instant => 0.0,
            Speed::Animated(steps) => (DWELL_STEPS / steps).min(MAX_DWELL),
        }
    }

    /// Plays the current move, turn or dwell for as much of `budget` as it
    /// needs. Returns false if the frame's time ran out first.
    fn advance(&mut self, current: Current, budget: &mut f64, out: &mut Frame) -> bool {
        let rate = match self.speed {
            Speed::Instant => None,
            Speed::Animated(steps) => Some(steps),
        };
        // How far to go now, in steps or degrees, out of `remaining`, at
        // `rate` per second: all of it in Instant mode.
        let mut progress = |remaining: f64, rate: Option<f64>| -> f64 {
            let Some(rate) = rate else { return remaining };
            if *budget * rate >= remaining {
                *budget -= remaining / rate;
                remaining
            } else {
                let step = *budget * rate;
                *budget = 0.0;
                step
            }
        };
        match current {
            Current::Move { from, to, pen, done } => {
                let length = ((to.x - from.x).powi(2) + (to.y - from.y).powi(2)).sqrt();
                let done = done + progress(length - done, rate);
                if done >= length {
                    self.pos = to;
                    if pen.down {
                        out.segments.push(Segment { from, to, pen });
                    }
                    return true;
                }
                let t = done / length;
                self.pos = Point { x: from.x + (to.x - from.x) * t, y: from.y + (to.y - from.y) * t };
                self.current = Some(Current::Move { from, to, pen, done });
                false
            }
            Current::Turn { from, by, done } => {
                let done = done + progress(by.abs() - done, rate.map(|r| r * DEGREES_PER_STEP));
                self.heading = (from + by.signum() * done).rem_euclid(360.0);
                if done >= by.abs() {
                    return true;
                }
                self.current = Some(Current::Turn { from, by, done });
                false
            }
            Current::Dwell { left } => {
                let done = progress(left, Some(1.0));
                if done >= left {
                    return true;
                }
                self.current = Some(Current::Dwell { left: left - done });
                false
            }
        }
    }

    /// The next event: from the queue, then the channel. Waits until
    /// `wait_until` if given, which is how lockstep stays fast: just after a
    /// reply, the next line's events are a few microseconds away.
    fn next_event(&mut self, wait_until: Option<Instant>) -> Option<Event> {
        if let Some(event) = self.queue.pop_front() {
            return Some(event);
        }
        let rx = self.rx.as_ref()?;
        let batch = match rx.try_recv() {
            Ok(batch) => batch,
            Err(TryRecvError::Disconnected) => {
                self.rx = None;
                self.disconnected = true;
                return None;
            }
            Err(TryRecvError::Empty) => {
                let wait = wait_until?.checked_duration_since(Instant::now())?;
                match rx.recv_timeout(wait) {
                    Ok(batch) => batch,
                    Err(RecvTimeoutError::Timeout) => return None,
                    Err(RecvTimeoutError::Disconnected) => {
                        self.rx = None;
                        self.disconnected = true;
                        return None;
                    }
                }
            }
        };
        self.queue.extend(batch);
        self.queue.pop_front()
    }
}
