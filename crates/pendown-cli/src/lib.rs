//! Runs a program with no timing and draws it (ADR 0009). The binary in
//! `main.rs` handles the command line; this part is what the tests use.

use std::{
    cell::RefCell,
    fmt::Write as _,
    rc::Rc,
    sync::Arc,
    thread,
    time::Instant,
};

use pendown_logo::{
    Error, Event, EventSink, Halted, Interpreter, Limits, RunError,
    metrics::{self, ThreadCpu},
};
use pendown_render::{Drawing, Mark, Segment};

/// The interpreter's stack (ADR 0007).
const STACK: usize = 256 * 1024 * 1024;

/// Marks are drawn in batches of this many, so a long program's pending
/// list stays small.
const DRAW_BATCH: usize = 4096;

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub width: u32,
    pub height: u32,
    /// Stop with an error after this many instructions.
    pub limit: u64,
    /// The seed for `RANDOM`, for a repeatable picture. Otherwise each run
    /// is different.
    pub seed: Option<u64>,
    /// Zoom in on what's drawn, rather than showing the whole turtle area
    /// (`Drawing::fit`).
    pub fit: bool,
}

/// The smallest area, in turtle steps, that `fit` zooms in to.
pub const FIT_MIN: f64 = 100.0;

pub struct Output {
    /// The picture, as far as the program got.
    pub png: Vec<u8>,
    /// Everything the program printed.
    pub printed: String,
    pub result: Result<(), RunError>,
    /// How many lines, arcs and shapes were drawn since the last `CLEAN`.
    pub drawn: usize,
    /// A one-line summary of where the time went, when metrics are on
    /// (ADR 0020).
    pub summary: Option<String>,
}

/// Turns events into lines, arcs and shapes in the drawing. The CLI has no
/// animation, so turns and `Instruction` events don't matter.
struct DrawingSink {
    drawing: Rc<RefCell<Drawing>>,
    printed: Rc<RefCell<String>>,
    pending: Vec<Mark>,
}

impl DrawingSink {
    fn add(&mut self, mark: Mark) {
        if mark.visible() {
            self.pending.push(mark);
            if self.pending.len() >= DRAW_BATCH {
                self.draw();
            }
        }
    }
}

impl DrawingSink {
    fn draw(&mut self) {
        let mut drawing = self.drawing.borrow_mut();
        drawing.push(&self.pending);
        drawing.draw_pending();
        self.pending.clear();
    }
}

impl EventSink for DrawingSink {
    fn send(&mut self, event: Event) -> Result<(), Halted> {
        match event {
            Event::Move { from, to, pen } => self.add(Segment { from, to, pen }.into()),
            Event::Arc { center, radius, start_heading, angle, pen } => {
                self.add(Mark::Arc { center, radius, start_heading, angle, pen })
            }
            Event::Ellipse { center, radii, heading, pen, fill } => self.add(Mark::Ellipse { center, radii, heading, pen, fill }),
            Event::Rect { center, size, corner, heading, pen, fill } => {
                self.add(Mark::Rect { center, size, corner, heading, pen, fill })
            }
            Event::Print(text) => self.printed.borrow_mut().push_str(&text),
            Event::Clean => {
                self.pending.clear();
                self.drawing.borrow_mut().clean();
            }
            Event::SetBackground { color } => {
                self.draw();
                self.drawing.borrow_mut().set_background(color);
            }
            // A Sync only comes in lockstep, which the CLI never sets.
            Event::Sync(sync) => sync.reply(),
            _ => {}
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Halted> {
        self.draw();
        Ok(())
    }
}

/// Runs `source` on its own thread, with the large stack the interpreter
/// needs, and draws it at the given size.
pub fn render(source: &str, options: Options) -> Output {
    let source = source.to_string();
    thread::Builder::new()
        .name("interpreter".into())
        .stack_size(STACK)
        .spawn(move || {
            let (started, cpu) = (Instant::now(), ThreadCpu::now());
            let drawing = Rc::new(RefCell::new(Drawing::new()));
            drawing.borrow_mut().resize(options.width, options.height);
            let printed = Rc::new(RefCell::new(String::new()));
            let sink = DrawingSink { drawing: drawing.clone(), printed: printed.clone(), pending: Vec::new() };
            let mut it = Interpreter::new(Box::new(sink), Arc::default());
            it.limits = Limits {
                stack_bytes: Some(STACK),
                instruction_events: false,
                max_instructions: Some(options.limit),
                ..Limits::default()
            };
            it.set_seed(options.seed);
            let result = it.run_program(&source);
            let instructions = it.stats.instructions;
            drop(it);
            if options.fit {
                drawing.borrow_mut().fit(FIT_MIN);
            }
            let drawing = drawing.borrow();
            let png = drawing.png();
            let printed = printed.borrow().clone();
            let summary = metrics::enabled().then(|| {
                let wall = started.elapsed();
                let m = &drawing.metrics;
                format!(
                    "run {:.2} s: {instructions} instructions at {:.1} M/s, {} marks drawn in {:.0} ms, PNG in {:.0} ms; CPU {}",
                    wall.as_secs_f64(),
                    instructions as f64 / wall.as_secs_f64().max(1e-9) / 1e6,
                    m.marks.get(),
                    m.drawing.total().as_secs_f64() * 1e3,
                    m.png.total().as_secs_f64() * 1e3,
                    ThreadCpu::now().since(&cpu).percentages(wall, "main"),
                )
            });
            Output { png, printed, result, drawn: drawing.segments(), summary }
        })
        .expect("can start the interpreter thread")
        .join()
        .expect("the interpreter thread doesn't panic")
}

/// Reads `250x300` as a width and a height in pixels.
pub fn parse_size(text: &str) -> Result<(u32, u32), String> {
    let bad = || format!("expected a size like 250x300, not {text:?}");
    let (w, h) = text.split_once(['x', 'X']).ok_or_else(bad)?;
    let (w, h): (u32, u32) = (w.trim().parse().map_err(|_| bad())?, h.trim().parse().map_err(|_| bad())?);
    if w == 0 || h == 0 || w > u16::MAX as u32 || h > u16::MAX as u32 {
        return Err(format!("the width and height must be from 1 to {}, not {text:?}", u16::MAX));
    }
    Ok((w, h))
}

/// An error as `program.logo:LINE:COLUMN: message`, with its chain of calls
/// on the lines after it (ADR 0005).
pub fn describe(path: &str, source: &str, error: &Error) -> String {
    let mut out = match error.span {
        Some(span) => {
            let start = (span.start as usize).min(source.len());
            let line_start = source[..start].rfind('\n').map_or(0, |i| i + 1);
            let column = source[line_start..start].chars().count() + 1;
            format!("{path}:{}:{column}: {}", span.line + 1, error.message)
        }
        None => format!("{path}: {}", error.message),
    };
    for (i, call) in error.calls.iter().enumerate() {
        let how = if i == 0 { "in" } else { "called from" };
        let _ = write!(out, "\n  {how} {} (line {})", call.procedure, call.span.line + 1);
    }
    out
}
