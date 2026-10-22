//! The timed cases: fixed programs through each part of Pendown, with no
//! window. Each returns how long one run took, and `main` keeps the best of
//! several, since anything that goes through a channel varies from run to
//! run.

use std::{
    cell::RefCell,
    rc::Rc,
    sync::{OnceLock, mpsc},
    thread,
    time::{Duration, Instant},
};

use pendown::{
    animator::{Animator, Speed},
    run::{self, DEFAULT_CHANNEL},
};
use pendown_logo::{BatchSender, Event, EventSink, Halted, Limits};
use pendown_render::{Drawing, Mark, Segment};

use crate::{Discard, interpreter, on_stack};

pub struct Case {
    pub name: &'static str,
    pub run: fn() -> Duration,
}

pub const CASES: &[Case] = &[
    Case { name: "interpreter: REPEAT [FD 1 RT 1], no events", run: || interpret(REPEAT, Sink::Discard, false) },
    Case { name: "interpreter: REPEAT [FD 1 RT 1], with Instruction events", run: || interpret(REPEAT, Sink::Discard, true) },
    Case { name: "interpreter: REPEAT [FD 1 RT 1], batches through a channel", run: || interpret(REPEAT, Sink::Batches, true) },
    Case { name: "interpreter: tail-recursive loop", run: || interpret(LOOP, Sink::Discard, false) },
    Case { name: "interpreter: FIB 25", run: || interpret(FIB, Sink::Discard, false) },
    Case { name: "interpreter: MAP and FOREACH over 300,000 members", run: || interpret(TEMPLATES, Sink::Discard, false) },
    Case { name: "renderer: draw the 131,070-line tree", run: draw_tree },
    Case { name: "renderer: the tree as a 3000 × 3000 PNG", run: png_tree },
    Case { name: "animator: the tree at Instant", run: || animate(TREE, Speed::Instant, None) },
    Case { name: "animator: 5,000 lines in lockstep", run: || animate(LINES, Speed::Animated(1e6), None) },
    Case { name: "end to end: the tree at Instant, drawn and uploaded", run: || animate(TREE, Speed::Instant, Some(1000)) },
];

const REPEAT: &str = "REPEAT 1000000 [FD 1 RT 1]";
const LOOP: &str = "TO LOOP :n\n IF :n = 0 [STOP]\n FD 1 RT 1\n LOOP :n - 1\nEND\nLOOP 500000";
const FIB: &str = "TO FIB :n\n IF :n < 2 [OUTPUT :n]\n OUTPUT (FIB :n - 1) + (FIB :n - 2)\nEND\nMAKE \"x FIB 25";
const TEMPLATES: &str = "MAKE \"xs ISEQ 1 300000\nMAKE \"ys MAP [? * 2] :xs\nFOREACH :ys [MAKE \"last ?]";

/// The scripted session's tree: 131,070 lines.
pub const TREE: &str = "TO TREE :size :depth
  IF :depth = 0 [STOP]
  FD :size
  LT 30 TREE :size * 0.72 :depth - 1
  RT 60 TREE :size * 0.72 :depth - 1
  LT 30 BK :size
END
PU BK 450 PD
TREE 230 16
";

/// 5,000 lines, each waiting for the animator in lockstep.
const LINES: &str = "TO STEP\n  FD 1\n  RT 1\nEND\nREPEAT 2500 [STEP]";

#[derive(Clone, Copy)]
enum Sink {
    Discard,
    Batches,
}

fn interpret(src: &'static str, sink: Sink, events: bool) -> Duration {
    on_stack(move || {
        let limits = Limits { instruction_events: events, ..Limits::default() };
        let (sink, drain): (Box<dyn EventSink>, _) = match sink {
            Sink::Discard => (Box::new(Discard), None),
            Sink::Batches => {
                let (tx, rx) = mpsc::sync_channel::<Vec<Event>>(DEFAULT_CHANNEL.capacity);
                let sink = BatchSender::new(tx, DEFAULT_CHANNEL.batch, DEFAULT_CHANNEL.max_wait);
                (Box::new(sink) as Box<dyn EventSink>, Some(thread::spawn(move || rx.into_iter().map(|b| b.len()).sum::<usize>())))
            }
        };
        let mut it = interpreter(sink, limits);
        let started = Instant::now();
        it.run_program(src).unwrap();
        drop(it);
        if let Some(drain) = drain {
            drain.join().unwrap();
        }
        started.elapsed()
    })
}

/// Collects the lines drawn.
struct Lines(Rc<RefCell<Vec<Mark>>>);

impl EventSink for Lines {
    fn send(&mut self, event: Event) -> Result<(), Halted> {
        if let Event::Move { from, to, pen } = event
            && pen.down
        {
            self.0.borrow_mut().push(Segment { from, to, pen }.into());
        }
        Ok(())
    }
}

/// The tree's lines, worked out once.
fn tree() -> &'static [Mark] {
    static LINES: OnceLock<Vec<Mark>> = OnceLock::new();
    LINES.get_or_init(|| {
        on_stack(|| {
            let lines = Rc::new(RefCell::new(Vec::new()));
            let mut it = interpreter(Box::new(Lines(lines.clone())), Limits { instruction_events: false, ..Limits::default() });
            it.run_program(TREE).unwrap();
            drop(it);
            lines.take()
        })
    })
}

/// Draws the tree into a square buffer, in the CLI's batches.
fn drawn_tree(size: u32) -> (Drawing, Duration) {
    let mut drawing = Drawing::new();
    drawing.resize(size, size);
    let started = Instant::now();
    for batch in tree().chunks(4096) {
        drawing.push(batch);
        drawing.draw_pending();
    }
    (drawing, started.elapsed())
}

fn draw_tree() -> Duration {
    drawn_tree(1000).1
}

/// Large enough to take tens of milliseconds, so it can be timed.
fn png_tree() -> Duration {
    let (drawing, _) = drawn_tree(3000);
    let started = Instant::now();
    std::hint::black_box(drawing.png());
    started.elapsed()
}

/// Plays `src` through the animator, a frame after another with no waiting,
/// each frame counting as 16 ms of animation. With `size`, the frames' lines
/// are drawn into a buffer that size and the changed tiles copied out, as
/// the window would upload them.
fn animate(src: &str, speed: Speed, size: Option<u32>) -> Duration {
    let mut drawing = size.map(|size| {
        let mut drawing = Drawing::new();
        drawing.resize(size, size);
        drawing
    });
    let started = Instant::now();
    let (mut animator, run): (Animator, _) = run::start(src.to_string(), speed, DEFAULT_CHANNEL);
    while !animator.finished() {
        let frame = animator.frame(Duration::from_millis(16), 100_000);
        if let Some(drawing) = &mut drawing {
            drawing.push(&frame.marks);
            drawing.draw_pending();
            for tile in drawing.take_dirty_tiles() {
                std::hint::black_box(drawing.tile_bgra(tile));
            }
        }
    }
    let took = started.elapsed();
    while run.exited().is_none() {
        thread::yield_now();
    }
    assert!(matches!(run.result(), Some(Ok(()))), "{src} failed: {:?}", run.result());
    took
}
