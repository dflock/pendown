//! The animator (ADR 0011): plays the interpreter's events at the chosen
//! speed, and implements Pause, Single Step and Stop (ADR 0012). It has no gpui
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

use pendown_logo::arc_point;
use pendown_logo::{Control, Event, Pen, Point, Span, Watch};
use pendown_render::{Mark, Segment};

use crate::metrics::AnimatorMetrics;

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

/// The longest the window goes without a frame while a run plays: a frame's
/// animation time is capped at this too, so nothing is lost.
pub const MAX_FRAME_GAP: Duration = Duration::from_millis(100);

/// Instant mode takes events for at most this long each frame.
pub const INSTANT_SLICE: Duration = Duration::from_millis(8);

/// After replying to a `Sync`, the animator waits up to this long in a frame
/// for the interpreter to send the next line's events, so lockstep doesn't
/// limit the program to one line a frame.
pub const SYNC_WAIT: Duration = Duration::from_millis(4);

/// How long Kame's last thought takes to fade once the next move or turn
/// starts, in seconds.
pub const FADE: f64 = 1.0 / 3.0;

/// What Kame is thinking about, for "Show Kame thinking": the turn or move
/// it's making, so far.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Thought {
    /// Turning `at` a point, from heading `from`, by `by` degrees so far:
    /// positive for right, and past 360 for more than a whole turn. `done`
    /// once it's turned all the way.
    Turn {
        at: Point,
        from: f64,
        by: f64,
        done: bool,
    },
    /// Moving `from` a point, to where it's got `to`, `length` steps so far,
    /// facing `heading`: backwards for `BK`. `done` once it's got there.
    Move {
        from: Point,
        to: Point,
        length: f64,
        heading: f64,
        done: bool,
    },
}

/// What to show of Kame thinking in a frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Thinking {
    /// The turn or move in progress, or the last one, until the next starts.
    pub current: Option<Thought>,
    /// The one before, fading: how much of it is left, from 1 to 0.
    pub fading: Option<(Thought, f64)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Running,
    Paused,
    /// Single Step: run to the next instruction, then pause.
    StepInstruction,
    /// Pause was pressed: finish the current instruction at once, then
    /// pause before the next.
    Pausing,
}

/// A replay (pause-and-edit issue 04): play out of sight, as at Instant
/// but in lockstep, until `to` instructions have been shown, then carry on
/// in `then`'s mode at the slider's speed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Replay {
    pub to: u64,
    pub then: Mode,
}

enum Current {
    Move {
        from: Point,
        to: Point,
        pen: Pen,
        done: f64,
    },
    /// An arc, which moves the turtle, or a shape, which doesn't: `done` of
    /// its `length` in steps so far.
    Mark {
        mark: Mark,
        length: f64,
        done: f64,
    },
    Turn {
        from: f64,
        by: f64,
        done: f64,
    },
    /// Keeps the highlight on an instruction that drew nothing.
    Dwell {
        left: f64,
    },
    /// `WAIT`: seconds left, counted in real time at every speed.
    Wait {
        left: f64,
    },
}

/// What a frame produced.
#[derive(Default)]
pub struct Frame {
    /// Lines, arcs and shapes finished this frame, to draw into the pixel
    /// buffer.
    pub marks: Vec<Mark>,
    /// A `CLEAN` happened this frame: clear the drawing before adding
    /// `marks`, which only holds what came after it.
    pub cleaned: bool,
    /// A `SETSPEED` changed the speed this frame, to this, so the slider
    /// should move.
    pub speed: Option<Speed>,
    /// The background changed this frame, to this. It goes under
    /// everything, so it doesn't matter where among `marks` it came.
    pub background: Option<[u8; 4]>,
    pub printed: Vec<String>,
    pub highlight_changed: bool,
    /// The counters and variables changed this frame: see `Animator::watch`.
    pub watch_changed: bool,
    /// Events taken this frame.
    pub events: usize,
}

pub struct Animator {
    rx: Option<Receiver<Vec<Event>>>,
    queue: VecDeque<Event>,
    current: Option<Current>,
    mode: Mode,
    speed: Speed,
    /// Replaying, until it gets there.
    replay: Option<Replay>,
    control: Arc<Control>,
    /// How long Instant mode takes events for each frame.
    pub instant_slice: Duration,
    pub pos: Point,
    pub heading: f64,
    /// Whether the turtle sprite is shown.
    pub shown: bool,
    pub highlight: Option<Span>,
    /// What the counters and variables panels show: the last `Watch` the
    /// animator has reached, which belongs with the highlight (ADR 0022).
    pub watch: Option<Arc<Watch>>,
    /// The last instruction hasn't moved or turned the turtle yet.
    idle_instruction: bool,
    /// The next instruction starts a breakpoint's line: pause there
    /// (ADR 0029).
    at_breakpoint: bool,
    /// Line starts shown, to compare with `Control::lines_started`.
    pub lines_shown: u64,
    pub instructions_shown: u64,
    /// What Kame is thinking about, and the last thought fading, with its
    /// seconds left: only at animated speeds.
    thought: Option<Thought>,
    fading: Option<(Thought, f64)>,
    /// The interpreter has finished, and every event has been taken.
    disconnected: bool,
    pub stopped: bool,
    pub metrics: AnimatorMetrics,
}

impl Animator {
    pub fn new(rx: Receiver<Vec<Event>>, control: Arc<Control>, speed: Speed) -> Animator {
        Animator::replaying(rx, control, speed, None)
    }

    /// An animator that replays its program up to a point first, if
    /// `replay` says so. It's set before the job starts, so the interpreter
    /// is in lockstep from its first line.
    pub fn replaying(
        rx: Receiver<Vec<Event>>,
        control: Arc<Control>,
        speed: Speed,
        replay: Option<Replay>,
    ) -> Animator {
        let replay = replay.filter(|r| r.to > 0);
        let animator = Animator {
            rx: Some(rx),
            queue: VecDeque::new(),
            current: None,
            mode: Mode::Running,
            speed,
            replay,
            control,
            instant_slice: INSTANT_SLICE,
            pos: Point { x: 0.0, y: 0.0 },
            heading: 0.0,
            shown: true,
            highlight: None,
            watch: None,
            idle_instruction: false,
            at_breakpoint: false,
            lines_shown: 0,
            instructions_shown: 0,
            thought: None,
            fading: None,
            disconnected: false,
            stopped: false,
            metrics: AnimatorMetrics::default(),
        };
        animator
            .control
            .speed
            .store(crate::speed::speed_level(speed), Ordering::Relaxed);
        animator.update_lockstep();
        animator
    }

    /// Whether a replay is still on its way to where it's going: nothing is
    /// shown until it gets there.
    pub fn replaying_now(&self) -> bool {
        self.replay.is_some()
    }

    /// The program ended, or stopped with an error, before a replay got
    /// where it was going.
    pub fn ended_early(&self) -> bool {
        self.replay.is_some() && self.finished()
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
        !self.finished()
            && matches!(
                self.mode,
                Mode::Running | Mode::StepInstruction | Mode::Pausing
            )
    }

    /// How long until the picture changes by about a device pixel, so the
    /// window can skip frames that would look the same (phase 3 issue 08).
    /// Zero means at the next frame: at Instant, or when there are events to
    /// take. `px_per_step` is the canvas's scale, and `turtle_px` how far
    /// the turtle reaches from its position, which says how far its head
    /// moves as it turns.
    pub fn next_change(&self, px_per_step: f64, turtle_px: f64) -> Duration {
        let Speed::Animated(steps) = self.speed else {
            return Duration::ZERO;
        };
        let secs = |s: f64| Duration::from_secs_f64(s.max(0.0));
        // The time for `remaining` steps (or degrees, at `rate`) to play,
        // and for one pixel's change at `px_per_unit`.
        let gap = |remaining: f64, rate: f64, px_per_unit: f64| {
            let to_end = remaining / rate;
            let pixel = if px_per_unit > 0.0 {
                1.0 / (rate * px_per_unit)
            } else {
                to_end
            };
            secs(to_end.min(pixel))
        };
        let gap = match &self.current {
            None => Duration::ZERO,
            Some(Current::Move {
                from,
                to,
                pen,
                done,
            }) => {
                let length = ((to.x - from.x).powi(2) + (to.y - from.y).powi(2)).sqrt();
                let seen = pen.down || self.shown;
                gap(length - done, steps, if seen { px_per_step } else { 0.0 })
            }
            Some(Current::Mark { mark, length, done }) => {
                let seen = mark.pen().down || (self.shown && matches!(mark, Mark::Arc { .. }));
                gap(length - done, steps, if seen { px_per_step } else { 0.0 })
            }
            Some(Current::Turn { by, done, .. }) => {
                let tip = if self.shown {
                    turtle_px * std::f64::consts::PI / 180.0
                } else {
                    0.0
                };
                gap(by.abs() - done, steps * DEGREES_PER_STEP, tip)
            }
            Some(Current::Dwell { left } | Current::Wait { left }) => secs(*left),
        };
        gap.min(MAX_FRAME_GAP)
    }

    /// What to show of Kame thinking: nothing at Instant, where a thousand
    /// arcs a second would be noise.
    pub fn thinking(&self) -> Thinking {
        Thinking {
            current: self.thought,
            fading: self
                .fading
                .map(|(t, left)| (t, (left / FADE).clamp(0.0, 1.0))),
        }
    }

    /// The run is over, so Kame stops thinking about its last move or turn,
    /// which starts to fade. Driving keeps it, for the entry it shows.
    pub fn stop_thinking(&mut self) {
        if let Some(last) = self.thought.take() {
            self.fading = Some((last, FADE));
        }
    }

    /// Fades the last thought by `dt`. `frame` does this while a run plays,
    /// and the window once it's over, when there are no more frames.
    pub fn fade_thinking(&mut self, dt: Duration) {
        if let Some((thought, left)) = self.fading {
            let left = left - dt.as_secs_f64();
            self.fading = (left > 0.0).then_some((thought, left));
        }
    }

    /// A move or turn starts: it's what Kame thinks about now, and the last
    /// thought starts to fade.
    fn think(&mut self, thought: Thought) {
        if self.speed == Speed::Instant || self.replay.is_some() {
            return;
        }
        if let Some(last) = self.thought.replace(thought) {
            self.fading = Some((last, FADE));
        }
    }

    /// Whether a move, turn or dwell is part-way through.
    pub fn in_progress(&self) -> bool {
        self.current.is_some()
    }

    /// What's drawn so far of the move, arc or shape in progress, as a
    /// polyline within `tolerance` turtle steps of the curve, with its pen.
    pub fn partial(&self, tolerance: f64) -> Option<(Vec<Point>, Pen)> {
        match &self.current {
            Some(Current::Move { from, pen, .. }) if pen.down => {
                Some((vec![*from, self.pos], *pen))
            }
            Some(Current::Mark { mark, length, done }) if mark.pen().down => {
                let fraction = if *length > 0.0 { done / length } else { 1.0 };
                Some((mark.partial(fraction, tolerance), mark.pen()))
            }
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
        let free =
            self.mode == Mode::Running && self.speed == Speed::Instant && self.replay.is_none();
        self.control.lockstep.store(!free, Ordering::Relaxed);
    }

    /// Changes the speed, and tells the interpreter the level, for `SPEED`.
    pub fn set_speed(&mut self, speed: Speed) {
        self.speed = speed;
        self.control
            .speed
            .store(crate::speed::speed_level(speed), Ordering::Relaxed);
        self.update_lockstep();
    }

    /// Pause: the next frame finishes the instruction being drawn, at once,
    /// so Kame is never left part-way along a move, and pauses before the
    /// next one (pause-and-edit issue 02).
    pub fn pause(&mut self) {
        if self.mode != Mode::Paused {
            self.mode = Mode::Pausing;
            self.update_lockstep();
        }
    }

    /// Paused, the animator doesn't take the next line's `Sync`, so the
    /// interpreter waits there until Resume or Single Step takes it.
    pub fn resume(&mut self) {
        self.mode = Mode::Running;
        self.update_lockstep();
    }

    /// Single Step: run to the next instruction, inside brackets and into
    /// procedures, then pause.
    pub fn single_step(&mut self) {
        self.mode = Mode::StepInstruction;
        self.update_lockstep();
    }

    /// Stop (ADR 0011): set the cancel flag, and drop the channel and any
    /// `Sync` in it, which wakes the interpreter wherever it's blocked.
    pub fn stop(&mut self) {
        self.control.cancel.store(true, Ordering::Relaxed);
        self.rx = None;
        self.queue.clear();
        self.current = None;
        self.stopped = true;
    }

    /// Plays one frame's worth of events. `dt` is the time since the last
    /// frame.
    pub fn frame(&mut self, dt: Duration, max_marks: usize) -> Frame {
        // At Instant there's no lockstep to send a `Watch` when it changes,
        // so ask for one a frame (ADR 0022).
        if self.speed == Speed::Instant && self.is_playing() {
            self.control.watch.store(true, Ordering::Relaxed);
        }
        let timing = self.metrics.frames.start();
        let out = self.play(dt, max_marks);
        self.metrics.frames.stop(timing);
        self.metrics.events.add(out.events as u64);
        out
    }

    fn play(&mut self, dt: Duration, max_marks: usize) -> Frame {
        let started = Instant::now();
        let mut out = Frame::default();
        // A replay plays as Instant does, out of sight.
        let replaying = self.replay.is_some();
        let instant = self.speed == Speed::Instant || replaying;
        // Running up to an instruction: Single Step, or Pause finishing one.
        let to_instruction = matches!(self.mode, Mode::StepInstruction | Mode::Pausing);
        let slice_end = started + self.instant_slice;
        let sync_wait_end = started + SYNC_WAIT;
        // Animation time left this frame, in seconds. A long frame (the
        // window was hidden, say) doesn't make the turtle jump.
        let mut budget = dt.min(MAX_FRAME_GAP).as_secs_f64();
        let mut replied = false;
        if instant {
            (self.thought, self.fading) = (None, None);
        } else {
            self.fade_thinking(dt);
        }

        loop {
            if !self.is_playing() {
                break;
            }
            if let Some(current) = self.current.take()
                && !self.advance(current, &mut budget, &mut out)
            {
                break;
            }
            if instant && (Instant::now() >= slice_end || out.marks.len() >= max_marks) {
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
            let Some(event) = self.next_event(wait_until) else {
                break;
            };
            out.events += 1;
            match event {
                Event::Instruction { span, starts_line } => {
                    if self.idle_instruction && !instant && !to_instruction {
                        // Show the instruction that did nothing for a moment,
                        // then come back to this one.
                        self.idle_instruction = false;
                        self.queue
                            .push_front(Event::Instruction { span, starts_line });
                        out.events -= 1;
                        self.current = Some(Current::Dwell { left: self.dwell() });
                        continue;
                    }
                    self.highlight = Some(span);
                    out.highlight_changed = !replaying;
                    self.idle_instruction = true;
                    self.instructions_shown += 1;
                    if starts_line {
                        self.lines_shown += 1;
                    }
                    if let Some(replay) = self.replay
                        && self.instructions_shown >= replay.to
                    {
                        // There: show it all, and carry on as asked from the
                        // next frame, at the slider's speed.
                        self.replay = None;
                        self.mode = replay.then;
                        self.update_lockstep();
                        out.highlight_changed = true;
                        out.watch_changed = true;
                        break;
                    }
                    let breakpoint = std::mem::take(&mut self.at_breakpoint);
                    if to_instruction || breakpoint {
                        self.mode = Mode::Paused;
                        self.update_lockstep();
                    }
                    continue;
                }
                Event::Move { from, to, pen } => {
                    self.idle_instruction = false;
                    if from != to {
                        self.think(Thought::Move {
                            from,
                            to: from,
                            length: 0.0,
                            heading: self.heading,
                            done: false,
                        });
                    }
                    self.current = Some(Current::Move {
                        from,
                        to,
                        pen,
                        done: 0.0,
                    });
                }
                Event::Turn { from, by } => {
                    self.idle_instruction = false;
                    if by != 0.0 {
                        self.think(Thought::Turn {
                            at: self.pos,
                            from,
                            by: 0.0,
                            done: false,
                        });
                    }
                    self.current = Some(Current::Turn {
                        from,
                        by,
                        done: 0.0,
                    });
                }
                Event::Arc {
                    center,
                    radius,
                    start_heading,
                    angle,
                    pen,
                } => {
                    self.idle_instruction = false;
                    let mark = Mark::Arc {
                        center,
                        radius,
                        start_heading,
                        angle,
                        pen,
                    };
                    self.current = Some(Current::Mark {
                        mark,
                        length: mark.length(),
                        done: 0.0,
                    });
                }
                Event::Ellipse {
                    center,
                    radii,
                    heading,
                    pen,
                    fill,
                } => {
                    self.start_shape(Mark::Ellipse {
                        center,
                        radii,
                        heading,
                        pen,
                        fill,
                    });
                }
                Event::Rect {
                    center,
                    size,
                    corner,
                    heading,
                    pen,
                    fill,
                } => {
                    self.start_shape(Mark::Rect {
                        center,
                        size,
                        corner,
                        heading,
                        pen,
                        fill,
                    });
                }
                Event::Watch(watch) => {
                    // It belongs to the next instruction, so it waits for
                    // the dwell on one that did nothing, as that does.
                    if self.idle_instruction && !instant && !to_instruction {
                        self.idle_instruction = false;
                        self.queue.push_front(Event::Watch(watch));
                        out.events -= 1;
                        self.current = Some(Current::Dwell { left: self.dwell() });
                        continue;
                    }
                    self.watch = Some(watch);
                    out.watch_changed = !replaying;
                    continue;
                }
                Event::Print(text) => out.printed.push(text),
                Event::ShowTurtle => self.shown = true,
                Event::HideTurtle => self.shown = false,
                Event::Clean => {
                    out.marks.clear();
                    out.cleaned = true;
                }
                Event::SetBackground { color } => out.background = Some(color),
                Event::Wait(duration) => {
                    self.idle_instruction = false;
                    self.current = Some(Current::Wait {
                        left: duration.as_secs_f64(),
                    });
                }
                Event::SetSpeed(level) => {
                    let speed = crate::speed::level_speed(level);
                    self.set_speed(speed);
                    out.speed = Some(speed);
                }
                // A replay passes breakpoints until it gets there.
                Event::Breakpoint => self.at_breakpoint = !replaying,
                Event::Sync(sync) => {
                    sync.reply();
                    replied = true;
                }
            }
        }
        out
    }

    /// A shape sweeps round as the pen draws it, and its fill, if it has
    /// one, appears when it's done. A filled shape with the pen up takes the
    /// same time, with nothing to see until the fill. With the pen up and no
    /// fill, there's nothing to see at all, so it takes no time.
    fn start_shape(&mut self, mark: Mark) {
        if mark.visible() {
            self.idle_instruction = false;
            self.current = Some(Current::Mark {
                mark,
                length: mark.length(),
                done: 0.0,
            });
        }
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
        // Pause finishes the instruction at once, and a replay plays out of
        // sight, so neither waits for the slider's speed.
        let at_once = self.mode == Mode::Pausing || self.replay.is_some();
        let rate = match self.speed {
            Speed::Animated(steps) if !at_once => Some(steps),
            _ => None,
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
            Current::Move {
                from,
                to,
                pen,
                done,
            } => {
                let length = ((to.x - from.x).powi(2) + (to.y - from.y).powi(2)).sqrt();
                let done = (done + progress(length - done, rate)).min(length);
                if done >= length {
                    self.pos = to;
                    self.thought_moved(done, true);
                    if pen.down {
                        out.marks.push(Segment { from, to, pen }.into());
                    }
                    return true;
                }
                let t = done / length;
                self.pos = Point {
                    x: from.x + (to.x - from.x) * t,
                    y: from.y + (to.y - from.y) * t,
                };
                self.thought_moved(done, false);
                self.current = Some(Current::Move {
                    from,
                    to,
                    pen,
                    done,
                });
                false
            }
            Current::Mark { mark, length, done } => {
                let done = done + progress(length - done, rate);
                let t = if length > 0.0 {
                    (done / length).min(1.0)
                } else {
                    1.0
                };
                // An arc carries the turtle with it; a shape leaves it put.
                if let Mark::Arc {
                    center,
                    radius,
                    start_heading,
                    angle,
                    ..
                } = mark
                {
                    (self.pos, self.heading) = arc_point(center, radius, start_heading, angle, t);
                }
                if done >= length {
                    out.marks.push(mark);
                    return true;
                }
                self.current = Some(Current::Mark { mark, length, done });
                false
            }
            Current::Turn { from, by, done } => {
                let done = done + progress(by.abs() - done, rate.map(|r| r * DEGREES_PER_STEP));
                let done = done.min(by.abs());
                self.heading = (from + by.signum() * done).rem_euclid(360.0);
                if let Some(Thought::Turn {
                    by: so_far,
                    done: finished,
                    ..
                }) = &mut self.thought
                {
                    *so_far = by.signum() * done;
                    *finished = done >= by.abs();
                }
                if done >= by.abs() {
                    return true;
                }
                self.current = Some(Current::Turn { from, by, done });
                false
            }
            // Real time, whatever the speed: a program that says WAIT
            // means it, even at Instant. Pause ends it, and a replay skips it.
            Current::Wait { left } => {
                if at_once || *budget >= left {
                    *budget -= left;
                    return true;
                }
                self.current = Some(Current::Wait {
                    left: left - *budget,
                });
                *budget = 0.0;
                false
            }
            Current::Dwell { left } => {
                let done = progress(left, Some(1.0));
                if at_once || done >= left {
                    return true;
                }
                self.current = Some(Current::Dwell { left: left - done });
                false
            }
        }
    }

    /// The move Kame is thinking about has got to where Kame is, `done`
    /// steps along.
    fn thought_moved(&mut self, done: f64, there: bool) {
        if let Some(Thought::Move {
            to,
            length,
            done: finished,
            ..
        }) = &mut self.thought
        {
            (*to, *length, *finished) = (self.pos, done, there);
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
