//! Runs a program with no timing and draws it (ADR 0009). The binary in
//! `main.rs` handles the command line; this part is what the tests use.

use std::{
    cell::RefCell,
    fmt::Write as _,
    rc::Rc,
    sync::Arc,
    thread,
};

use pendown_logo::{Error, Event, EventSink, Halted, Interpreter, Limits, RunError};
use pendown_render::{Drawing, Segment};

/// The interpreter's stack (ADR 0007).
const STACK: usize = 256 * 1024 * 1024;

/// Segments are drawn in batches of this many, so a long program's pending
/// list stays small.
const DRAW_BATCH: usize = 4096;

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub width: u32,
    pub height: u32,
    /// Stop with an error after this many instructions.
    pub limit: u64,
}

pub struct Output {
    /// The picture, as far as the program got.
    pub png: Vec<u8>,
    /// Everything the program printed.
    pub printed: String,
    pub result: Result<(), RunError>,
}

/// Turns events into segments in the drawing. The CLI has no animation, so
/// turns and `Instruction` events don't matter.
struct DrawingSink {
    drawing: Rc<RefCell<Drawing>>,
    printed: Rc<RefCell<String>>,
    pending: Vec<Segment>,
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
            Event::Move { from, to, pen } if pen.down => {
                self.pending.push(Segment { from, to, pen });
                if self.pending.len() >= DRAW_BATCH {
                    self.draw();
                }
            }
            Event::Print(text) => self.printed.borrow_mut().push_str(&text),
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
            let result = it.run_program(&source);
            drop(it);
            let png = drawing.borrow().png();
            let printed = printed.borrow().clone();
            Output { png, printed, result }
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
