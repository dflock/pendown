//! A recursive tree-walking evaluator with tail calls eliminated by
//! trampolining, and a limit on calls that aren't tail calls (ADR 0007).

use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering},
        mpsc,
    },
};

use smallvec::SmallVec;

use crate::{
    colour::{PALETTE, Rgba},
    error::{CallSite, Error, RunError},
    event::{Event, EventSink, Pen, Point, SyncPoint},
    geometry::{Area, TurtleMode},
    metrics::InterpreterMetrics,
    names::{Names, Sym},
    parse::{self, Arity, Block, Call, Cx, Expr, Instr, Target},
    prims::{self, Args, Imp, PRIMS},
    random::Random,
    span::{SourceId, Span},
    value::{List, Style, Value},
    watch::{self, Scope, ScopeKind, Variable, Watch},
};

/// The pen, fill and background colours' values, and the colours formatted
/// from them for a `Watch`.
type ColoursShown = ([Value; 3], Arc<[watch::Colour]>);

/// How the evaluator leaves an instruction other than by finishing it.
pub enum Flow {
    Stop(Span),
    Output(Value, Span),
    Error(Box<Error>),
    /// `THROW tag`, on its way to a `CATCH` with the same tag (ADR 0021).
    Throw {
        tag: Rc<str>,
        value: Option<Value>,
        span: Span,
    },
    /// Stopped from outside (ADR 0007): `CATCH` must never handle this.
    Halt,
}

impl From<Error> for Flow {
    fn from(e: Error) -> Flow {
        Flow::Error(Box::new(e))
    }
}

pub struct Proc {
    pub name: Rc<str>,
    pub inputs: Vec<Sym>,
    pub body: Block,
    /// The `TO` line, for jumping to the definition.
    pub title: Span,
}

type ArgVec = SmallVec<[Value; 4]>;

/// "Call this next": a call in tail position, handed back to the call loop
/// in `call_user` so it runs without growing the Rust stack.
pub struct Tail {
    proc: Rc<Proc>,
    args: ArgVec,
    span: Span,
    /// False for a tail call written as an instruction, whose output, if
    /// any, has nowhere to go. True after `OUTPUT`.
    wants_output: bool,
}

/// What an expression in tail position gives back.
enum Ret {
    Value(Value),
    Tail(Tail),
}

/// One active procedure call. Variables use shallow binding: `vars` holds
/// each name's current value, and a frame saves the values its inputs
/// replaced, putting them back when it ends. That's dynamic scope with O(1)
/// lookups. A tail call adds its inputs to the current frame instead of
/// pushing one, so the frame's size is bounded by the number of names.
///
/// Lambda templates and `FOR` push frames with no procedure, for their
/// local variables, labelled with the command.
struct Frame {
    proc: Option<Rc<Proc>>,
    label: Option<Rc<str>>,
    call_span: Span,
    saved: SmallVec<[(Sym, Option<Value>); 4]>,
}

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// The most procedure calls that can be active at once, not counting
    /// tail calls, before a friendly error.
    pub max_depth: usize,
    /// The stack available to the interpreter, if known. Calls stop with the
    /// same error when fewer than `STACK_MARGIN` bytes are left, whatever
    /// `max_depth` says, so a deep recursion can never crash the process.
    pub stack_bytes: Option<usize>,
    /// Send an `Instruction` event before each instruction. The GUI needs
    /// them; the CLI doesn't.
    pub instruction_events: bool,
    /// Stop with an error after this many instructions. The CLI uses it so
    /// an endless program, such as `FOREVER`, still ends; the GUI has Stop.
    pub max_instructions: Option<u64>,
}

impl Default for Limits {
    fn default() -> Limits {
        Limits {
            max_depth: 10_000,
            stack_bytes: None,
            instruction_events: true,
            max_instructions: None,
        }
    }
}

/// Room left for primitives and the error path after the last call allowed.
pub const STACK_MARGIN: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub instructions: u64,
    /// The most procedure calls active at once.
    pub max_depth: usize,
    /// The most stack used, in bytes, measured at each procedure call.
    pub max_stack: usize,
}

#[derive(Clone)]
pub struct Turtle {
    pub pos: Point,
    /// Degrees clockwise from north.
    pub heading: f64,
    pub pen: Pen,
    /// The pen colour in the form it was set, for `PENCOLOR`.
    pub pen_colour: Value,
    /// What shapes are filled with, if anything (ADR 0017), and the form it
    /// was set in, for `FILLCOLOR`.
    pub fill: Option<Rgba>,
    pub fill_colour: Value,
    pub shown: bool,
    pub mode: TurtleMode,
}

impl Default for Turtle {
    fn default() -> Turtle {
        Turtle {
            pos: Point { x: 0.0, y: 0.0 },
            heading: 0.0,
            // UCBLogo's defaults: a white pen on a black background.
            pen: Pen {
                down: true,
                color: PALETTE[7],
                size: 1.0,
            },
            pen_colour: Value::Num(7.0),
            fill: None,
            fill_colour: Value::word("none"),
            shown: true,
            mode: TurtleMode::Wrap,
        }
    }
}

/// Flags shared between the interpreter thread and the front end.
#[derive(Debug, Default)]
pub struct Control {
    /// Stop: checked before every instruction (ADR 0011).
    pub cancel: AtomicBool,
    /// Wait for the front end before running each line, with a `Sync` after
    /// the line's first `Instruction` (ADR 0012). The GUI sets this while
    /// paused or stepping, and at animated speeds.
    pub lockstep: AtomicBool,
    /// How many lines the interpreter has started, so the front end can see
    /// how far ahead of the highlight it is.
    pub lines_started: AtomicU64,
    /// The speed level from 1 to 11 that the slider shows, for `SPEED`. The
    /// GUI keeps it up to date; 0 means there's no slider, as in the CLI,
    /// and `SPEED` outputs 11.
    pub speed: AtomicU8,
    /// The front end wants a `Watch` at the next instruction. Lockstep sends
    /// one whenever it changes; at Instant the GUI sets this each frame.
    pub watch: AtomicBool,
    /// The editor's lines with a breakpoint, counting from 0 (ADR 0029):
    /// the interpreter waits at each, even out of lockstep. `any_breakpoints`
    /// saves looking when there are none.
    breakpoints: Mutex<Vec<u32>>,
    any_breakpoints: AtomicBool,
    pub metrics: InterpreterMetrics,
}

impl Control {
    /// Sets the editor's lines that have a breakpoint, counting from 0. It
    /// can change while a program runs, and takes effect at the next line.
    pub fn set_breakpoints(&self, lines: impl IntoIterator<Item = u32>) {
        let mut lines: Vec<u32> = lines.into_iter().collect();
        lines.sort_unstable();
        lines.dedup();
        self.any_breakpoints
            .store(!lines.is_empty(), Ordering::Relaxed);
        *self.breakpoints.lock().unwrap() = lines;
    }

    /// Whether the editor's line `line` has a breakpoint.
    pub fn is_breakpoint(&self, line: u32) -> bool {
        self.any_breakpoints.load(Ordering::Relaxed)
            && self
                .breakpoints
                .lock()
                .unwrap()
                .binary_search(&line)
                .is_ok()
    }
}

/// The special variables (issue 03): ordinary variables that change how
/// Pendown behaves, interned once.
struct Specials {
    caseignoredp: Sym,
    erract: Sym,
    fullprintp: Sym,
    printdepthlimit: Sym,
    printwidthlimit: Sym,
    logoversion: Sym,
}

pub const SPECIAL_VARIABLES: &[&str] = &[
    "CASEIGNOREDP",
    "ERRACT",
    "FULLPRINTP",
    "PRINTDEPTHLIMIT",
    "PRINTWIDTHLIMIT",
    "LOGOVERSION",
];

/// `LOGOVERSION`: Pendown's version, such as 0.1, as a number.
fn logo_version() -> f64 {
    let mut parts = env!("CARGO_PKG_VERSION").split('.');
    let major = parts.next().unwrap_or("0");
    let minor = parts.next().unwrap_or("0");
    format!("{major}.{minor}").parse().unwrap_or(0.0)
}

/// A `REPEAT`, `FOREVER`, `FOR` or template running: which pass it's on,
/// out of how many if that's known. `FOR`'s doesn't count for `REPCOUNT`.
pub(crate) struct Counter {
    pub(crate) name: Rc<str>,
    pub(crate) count: i64,
    pub(crate) of: Option<i64>,
    pub(crate) repcount: bool,
}

/// The slot fillers of the explicit-slot template running: `?1` is
/// `values[0]`. `(?REST 1)` is what comes after member `index` of the first
/// data input, worked out only if it's used.
pub(crate) struct Slots {
    pub(crate) values: Vec<Value>,
    pub(crate) data: Rc<Vec<crate::prims::Data>>,
    pub(crate) index: usize,
}

pub struct Interpreter {
    names: Names,
    specials: Specials,
    pub(crate) slots: Vec<Slots>,
    /// The error `CATCH "ERROR` caught last, as `ERROR` outputs it.
    pub(crate) caught: Option<Value>,
    /// How many `CATCH "ERROR`s are running: errors inside them don't run
    /// `ERRACT`.
    pub(crate) catching_errors: usize,
    pub(crate) vars: Vec<Option<Value>>,
    procs: Vec<Option<Rc<Proc>>>,
    frames: Vec<Frame>,
    /// The loops and templates running, outermost first, for `REPCOUNT`
    /// and the counters panel.
    pub(crate) counters: Vec<Counter>,
    pub(crate) turtle: Turtle,
    /// `RANDOM`'s generator, seeded afresh on each Run unless `seed` is set.
    pub(crate) random: Random,
    seed: Option<u64>,
    /// The colours that numbers stand for, which `SETPALETTE` changes.
    pub(crate) palette: [Rgba; 16],
    /// The background colour, and the form it was set in.
    pub(crate) background: (Rgba, Value),
    /// The turtle area, which Run doesn't change.
    pub(crate) area: Area,
    sink: Box<dyn EventSink>,
    control: Arc<Control>,
    last_line: Option<(SourceId, u32)>,
    console_entries: u32,
    /// Kame and the background as each console entry since the last Run
    /// found them, newest last, for Undo (ADR 0025).
    before_entries: Vec<(Turtle, (Rgba, Value))>,
    stack_base: usize,
    true_word: Value,
    false_word: Value,
    pub limits: Limits,
    pub stats: Stats,
    /// `stats.instructions` when the current job began.
    job_start: u64,
    /// The last `Watch` sent, so an unchanged one isn't sent again.
    last_watch: Option<Arc<Watch>>,
    /// The colours `watch` last formatted, and what they were made from, so
    /// they're formatted again only when they change: a `Watch` is built for
    /// every instruction in lockstep.
    colours_shown: RefCell<Option<ColoursShown>>,
}

/// The address of something on the stack, for measuring how much is used.
/// Stacks grow down on every platform we target.
#[inline(never)]
fn stack_addr() -> usize {
    let marker = 0u8;
    std::hint::black_box(&marker) as *const u8 as usize
}

/// A `THROW` that no `CATCH` caught. `THROW "ERROR` is an error by design.
fn uncaught(tag: &str, value: Option<Value>, span: Span) -> Error {
    let message = match value {
        _ if !tag.eq_ignore_ascii_case("error") => {
            format!("Can't find a CATCH for {}", tag.to_uppercase())
        }
        Some(v) => v.print_form(),
        None => "THROW \"ERROR".to_string(),
    };
    Error::new(message, Some(span))
}

fn dont_say(v: &Value, span: Span) -> Flow {
    Error::new(
        format!("You don't say what to do with {}", v.show()),
        Some(span),
    )
    .into()
}

fn didnt_output(name: &str, to: &str, span: Span) -> Flow {
    let message = if to.is_empty() {
        format!("{} didn't output a value", name.to_uppercase())
    } else {
        format!(
            "{} didn't output to {}",
            name.to_uppercase(),
            to.to_uppercase()
        )
    };
    Error::new(message, Some(span)).into()
}

impl Interpreter {
    pub fn new(sink: Box<dyn EventSink>, control: Arc<Control>) -> Interpreter {
        let mut names = Names::default();
        let specials = Specials {
            caseignoredp: names.define("CASEIGNOREDP"),
            erract: names.define("ERRACT"),
            fullprintp: names.define("FULLPRINTP"),
            printdepthlimit: names.define("PRINTDEPTHLIMIT"),
            printwidthlimit: names.define("PRINTWIDTHLIMIT"),
            logoversion: names.define("LOGOVERSION"),
        };
        let mut it = Interpreter {
            names,
            specials,
            slots: Vec::new(),
            caught: None,
            catching_errors: 0,
            vars: Vec::new(),
            procs: Vec::new(),
            frames: Vec::new(),
            counters: Vec::new(),
            turtle: Turtle::default(),
            area: Area::default(),
            background: (PALETTE[0], Value::Num(0.0)),
            palette: PALETTE,
            random: Random::new(Random::fresh_seed()),
            seed: None,
            sink,
            control,
            last_line: None,
            console_entries: 0,
            before_entries: Vec::new(),
            stack_base: 0,
            true_word: Value::word("true"),
            false_word: Value::word("false"),
            limits: Limits::default(),
            stats: Stats::default(),
            job_start: 0,
            last_watch: None,
            colours_shown: RefCell::new(None),
        };
        it.grow();
        it.set_specials();
        it
    }

    /// The special variables' starting values. The others have none.
    fn set_specials(&mut self) {
        self.vars[self.specials.caseignoredp as usize] = Some(self.true_word.clone());
        self.vars[self.specials.logoversion as usize] = Some(Value::Num(logo_version()));
    }

    fn special(&self, sym: Sym) -> Option<&Value> {
        self.vars[sym as usize].as_ref()
    }

    /// `CASEIGNOREDP`: whether words compare without case. Anything but
    /// `FALSE` counts as true, since it starts true.
    pub(crate) fn ignore_case(&self) -> bool {
        self.special(self.specials.caseignoredp)
            .and_then(Value::as_bool)
            != Some(false)
    }

    /// `EQUALP`, with `CASEIGNOREDP`.
    pub(crate) fn equal(&self, a: &Value, b: &Value) -> bool {
        crate::value::equal(a, b, self.ignore_case())
    }

    /// How `PRINT` and friends print, from `FULLPRINTP`, `PRINTDEPTHLIMIT`
    /// and `PRINTWIDTHLIMIT`. A limit that isn't a whole number of zero or
    /// more is ignored.
    pub(crate) fn print_style(&self) -> Style {
        let limit = |sym| {
            let n = self.special(sym)?.as_num()?;
            (n >= 0.0 && n.fract() == 0.0).then_some(n as usize)
        };
        Style {
            depth: limit(self.specials.printdepthlimit),
            width: limit(self.specials.printwidthlimit),
            full: self
                .special(self.specials.fullprintp)
                .and_then(Value::as_bool)
                == Some(true),
            length: None,
        }
    }

    /// Runs `ERRACT`'s list, if it has one, for an error that no `CATCH`
    /// will handle, where the error happened, so it can see the variables
    /// there. `ERRACT` has no value while it runs. An error in it replaces
    /// the first.
    fn run_erract(&mut self, error: &Error) -> Result<(), Flow> {
        if self.catching_errors > 0 {
            return Ok(());
        }
        let sym = self.specials.erract as usize;
        let list = match &self.vars[sym] {
            Some(Value::List(list)) if !list.items.is_empty() => list.clone(),
            _ => return Ok(()),
        };
        let span = error.span.unwrap_or(Span {
            source: SourceId::Editor,
            start: 0,
            end: 0,
            line: 0,
        });
        let saved = self.vars[sym].take();
        let result = self.run_list(&list, span, false);
        self.vars[sym] = saved;
        result.map(|_| ())
    }

    /// The innermost procedure running, if any, for `ERROR`.
    pub(crate) fn current_procedure(&self) -> Option<Rc<str>> {
        self.frames
            .iter()
            .rev()
            .find_map(|f| f.proc.as_ref().map(|p| p.name.clone()))
    }

    /// Sends events somewhere else, and takes Stop and lockstep from a new
    /// `Control`, keeping everything else: procedures, variables, the turtle.
    /// The GUI does this before each run or console entry, since each has
    /// its own channel and can be stopped on its own.
    pub fn set_output(&mut self, sink: Box<dyn EventSink>, control: Arc<Control>) {
        self.sink = sink;
        self.control = control;
    }

    /// The Run button (ADR 0006): clear all procedures and variables, reset
    /// the turtle, parse the whole source, define every `TO`, then run the
    /// top-level instructions.
    pub fn run_program(&mut self, text: &str) -> Result<(), RunError> {
        self.reset();
        self.run_source(text, SourceId::Editor)
    }

    /// Back to how a run starts: no variables or procedures, Kame home with
    /// the default pen, the default background and nothing to undo. Reset
    /// does this without running anything (ADR 0028).
    pub fn reset(&mut self) {
        self.vars.clear();
        self.procs.clear();
        self.names.forget_definitions();
        self.grow();
        self.set_specials();
        self.slots.clear();
        self.caught = None;
        self.catching_errors = 0;
        self.counters.clear();
        self.turtle = Turtle::default();
        self.background = (PALETTE[0], Value::Num(0.0));
        self.palette = PALETTE;
        self.random = Random::new(self.seed.unwrap_or_else(Random::fresh_seed));
        self.last_line = None;
        self.console_entries = 0;
        self.before_entries.clear();
    }

    /// A console entry, which runs against whatever the last run left.
    /// Kame and the background are saved first, even for an entry that
    /// doesn't parse, so each entry has one for Undo to take back.
    pub fn run_console(&mut self, text: &str) -> Result<(), RunError> {
        self.before_entries
            .push((self.turtle.clone(), self.background.clone()));
        self.define_console(text)
    }

    /// Runs `text` at the console without saving Kame for Undo: for what
    /// Pendown types there itself, such as Teach Kame's procedure, which
    /// isn't one of the user's entries.
    pub fn define_console(&mut self, text: &str) -> Result<(), RunError> {
        self.console_entries += 1;
        self.run_source(text, SourceId::Console(self.console_entries))
    }

    /// Undo at the console (ADR 0025): puts Kame and the background back as
    /// the newest console entry not yet undone found them. Variables and
    /// procedures stay as they are. With nothing to undo, nothing changes.
    /// Like a job, it ends with a `Watch`, so the colours panel follows.
    pub fn undo_console(&mut self) -> Result<(), RunError> {
        if let Some((turtle, background)) = self.before_entries.pop() {
            self.turtle = turtle;
            self.background = background;
        }
        self.last_watch = None;
        if self.limits.instruction_events {
            let _ = self.send_watch();
        }
        self.sink.flush().map_err(|_| RunError::Halted)
    }

    /// Makes every Run use this seed for `RANDOM`, so its drawings repeat:
    /// for tests and the CLI. `None` picks a new seed for each Run, as the
    /// GUI does (ADR 0006).
    pub fn set_seed(&mut self, seed: Option<u64>) {
        self.seed = seed;
    }

    /// Sets the turtle area's size in turtle steps, centred on 0, 0. The
    /// default is 1000 × 1000 (ADR 0003).
    pub fn set_area(&mut self, width: f64, height: f64) {
        self.area = Area {
            half_width: width / 2.0,
            half_height: height / 2.0,
        };
    }

    pub fn area(&self) -> Area {
        self.area
    }

    /// Where a procedure is defined, if it is.
    pub fn definition(&mut self, name: &str) -> Option<Span> {
        let sym = self.intern(name);
        self.procs[sym as usize].as_ref().map(|p| p.title)
    }

    /// Parses `text` as if it were the editor buffer and renders the
    /// top-level instructions as s-expressions, one per instruction.
    pub fn parse_to_sexp(&mut self, text: &str) -> Result<Vec<String>, Vec<Error>> {
        let program = self.parse(text, SourceId::Editor)?;
        Ok(program
            .top
            .iter()
            .map(|i| i.expr.sexp(&self.names))
            .collect())
    }

    fn parse(&mut self, text: &str, source: SourceId) -> Result<parse::Program, Vec<Error>> {
        let procs = &self.procs;
        let known = |sym: Sym| -> Option<Arity> {
            procs
                .get(sym as usize)?
                .as_ref()
                .map(|p| Arity::fixed(p.inputs.len()))
        };
        let mut cx = Cx {
            names: &mut self.names,
            known: &known,
            defining: Default::default(),
        };
        let program = parse::parse_program(text, source, &mut cx);
        self.grow();
        program
    }

    fn run_source(&mut self, text: &str, source: SourceId) -> Result<(), RunError> {
        let started = self.control.metrics.running.start();
        self.job_start = self.stats.instructions;
        // Each job has its own front end, which hasn't seen any `Watch` yet.
        self.last_watch = None;
        let result = self.run_source_unflushed(text, source);
        // What the run left, for the variables panel. If Stop was pressed,
        // nothing is listening.
        if self.limits.instruction_events && !matches!(result, Err(RunError::Halted)) {
            let _ = self.send_watch();
        }
        let result = match self.sink.flush() {
            Ok(()) => result,
            Err(_) => Err(RunError::Halted),
        };
        self.publish_instructions();
        self.control.metrics.running.stop(started);
        result
    }

    fn publish_instructions(&self) {
        self.control
            .metrics
            .instructions
            .set(self.stats.instructions - self.job_start);
    }

    fn run_source_unflushed(&mut self, text: &str, source: SourceId) -> Result<(), RunError> {
        let program = self.parse(text, source).map_err(RunError::Parse)?;
        for def in program.procs {
            let name = self.names.spelling(def.sym).clone();
            self.procs[def.sym as usize] = Some(Rc::new(Proc {
                name,
                inputs: def.inputs,
                body: def.body,
                title: def.title,
            }));
        }
        self.stack_base = stack_addr();
        for instr in &program.top {
            match self.exec_instr(instr, false) {
                Ok(_) => {}
                Err(Flow::Halt) => return Err(RunError::Halted),
                Err(Flow::Error(e)) => {
                    // An error outside any procedure: inside one, `ERRACT`
                    // has already run.
                    if e.calls.is_empty() {
                        match self.run_erract(&e) {
                            Err(Flow::Error(other)) => return Err(RunError::Runtime(*other)),
                            Err(Flow::Halt) => return Err(RunError::Halted),
                            _ => {}
                        }
                    }
                    return Err(RunError::Runtime(*e));
                }
                // `THROW "TOPLEVEL` ends the program, as if it had finished.
                Err(Flow::Throw { tag, .. }) if tag.eq_ignore_ascii_case("toplevel") => {
                    return Ok(());
                }
                Err(Flow::Throw { tag, value, span }) => {
                    return Err(RunError::Runtime(uncaught(&tag, value, span)));
                }
                Err(Flow::Stop(span)) => {
                    let e = Error::new("STOP can only be used inside a procedure", Some(span))
                        .about("STOP");
                    return Err(RunError::Runtime(e));
                }
                Err(Flow::Output(_, span)) => {
                    let e = Error::new("OUTPUT can only be used inside a procedure", Some(span))
                        .about("OUTPUT");
                    return Err(RunError::Runtime(e));
                }
            }
        }
        Ok(())
    }

    /// Makes room for every interned name in the variable and procedure tables.
    fn grow(&mut self) {
        let n = self.names.len();
        self.vars.resize(n, None);
        self.procs.resize(n, None);
    }

    pub(crate) fn intern(&mut self, name: &str) -> Sym {
        let sym = self.names.intern(name);
        self.grow();
        sym
    }

    /// Makes a variable local to the running procedure, with no value yet
    /// (`LOCAL`). With shallow binding, that means saving its current value
    /// in the procedure's frame, to put back when it ends. A tail-called
    /// procedure shares its caller's frame, so its locals go there too.
    /// Returns false outside any procedure.
    pub(crate) fn make_local(&mut self, sym: Sym) -> bool {
        let Some(frame) = self.frames.last_mut() else {
            return false;
        };
        let slot = &mut self.vars[sym as usize];
        if frame.saved.iter().any(|(s, _)| *s == sym) {
            *slot = None;
        } else {
            frame.saved.push((sym, slot.take()));
        }
        true
    }

    /// Like `intern`, for a name `MAKE` is creating (ADR 0004).
    pub(crate) fn define(&mut self, name: &str) -> Sym {
        let sym = self.names.define(name);
        self.grow();
        sym
    }

    pub(crate) fn control(&self) -> &Control {
        &self.control
    }

    pub(crate) fn bool(&self, b: bool) -> Value {
        if b {
            self.true_word.clone()
        } else {
            self.false_word.clone()
        }
    }

    pub(crate) fn emit(&mut self, event: Event) -> Result<(), Flow> {
        self.sink.send(event).map_err(|_| Flow::Halt)
    }

    /// Sends a `Watch` for the panels, unless nothing has changed since the
    /// last one.
    fn send_watch(&mut self) -> Result<(), Flow> {
        let watch = self.watch();
        if self.last_watch.as_deref() == Some(&watch) {
            return Ok(());
        }
        let watch = Arc::new(watch);
        self.last_watch = Some(watch.clone());
        self.emit(Event::Watch(watch))
    }

    /// The counters running and the variables the running instruction can
    /// see. With shallow binding, `vars` holds what's visible; each frame
    /// saved the values its own variables hid, so a frame's variables have
    /// the values saved by the next frame up that has the same name.
    pub fn watch(&self) -> Watch {
        let style = Style {
            depth: Some(4),
            width: Some(20),
            full: true,
            length: Some(watch::VALUE_LIMIT),
        };
        let show = |v: &Option<Value>| {
            v.as_ref().map(|v| {
                let mut text = v.show_styled(style);
                if text.len() > watch::VALUE_LIMIT {
                    let end = (0..=watch::VALUE_LIMIT)
                        .rev()
                        .find(|&i| text.is_char_boundary(i))
                        .unwrap_or(0);
                    text.truncate(end);
                    text.push_str("...");
                }
                text
            })
        };
        let counters = self
            .counters
            .iter()
            .enumerate()
            .map(|(i, c)| watch::Counter {
                name: c.name.to_uppercase(),
                count: c.count,
                of: c.of,
                repcount: c.repcount && !self.counters[i + 1..].iter().any(|later| later.repcount),
            })
            .collect();

        // From the innermost frame out: the value each name has in the scope
        // below the ones seen so far.
        let mut below: Vec<(Sym, &Option<Value>)> = Vec::new();
        let mut scopes = Vec::with_capacity(self.frames.len() + 1);
        for frame in self.frames.iter().rev() {
            let variables = frame
                .saved
                .iter()
                .map(|(sym, old)| {
                    let above = below.iter().rposition(|(s, _)| s == sym);
                    let value = match above {
                        Some(i) => below.remove(i).1,
                        None => &self.vars[*sym as usize],
                    };
                    below.push((*sym, old));
                    Variable {
                        name: self.names.spelling(*sym).to_string(),
                        value: show(value),
                        hidden: above.is_some(),
                    }
                })
                .collect();
            let (kind, name) = match (&frame.proc, &frame.label) {
                (Some(proc), _) => (ScopeKind::Procedure, proc.name.to_string()),
                (None, label) => (
                    ScopeKind::Loop,
                    label.as_deref().unwrap_or("").to_uppercase(),
                ),
            };
            scopes.push(Scope {
                kind,
                name,
                variables,
            });
        }

        let mut globals: Vec<Variable> = (0..self.vars.len() as Sym)
            .filter_map(|sym| {
                let hidden = below.iter().rposition(|(s, _)| *s == sym);
                let value = match hidden {
                    Some(i) => below[i].1,
                    None => &self.vars[sym as usize],
                };
                let value = value.as_ref()?;
                if self.is_default_special(sym, value) {
                    return None;
                }
                Some(Variable {
                    name: self.names.spelling(sym).to_string(),
                    value: show(&Some(value.clone())),
                    hidden: hidden.is_some(),
                })
            })
            .collect();
        globals.sort_by_cached_key(|v| v.name.to_lowercase());
        scopes.push(Scope {
            kind: ScopeKind::Globals,
            name: String::new(),
            variables: globals,
        });
        scopes.reverse();
        let sources = [
            self.turtle.pen_colour.clone(),
            self.turtle.fill_colour.clone(),
            self.background.1.clone(),
        ];
        // A palette number can look different with the same value, after
        // SETPALETTE, so the colours themselves are compared too.
        let rgbas = [
            Some(self.turtle.pen.color),
            self.turtle.fill,
            Some(self.background.0),
        ];
        let mut shown = self.colours_shown.borrow_mut();
        let colours = match shown.as_ref() {
            Some((old, colours))
                if old.iter().zip(&sources).all(|(a, b)| a.same(b))
                    && colours.iter().zip(&rgbas).all(|(c, rgba)| c.rgba == *rgba) =>
            {
                colours.clone()
            }
            _ => {
                let names = ["PENCOLOR", "FILLCOLOR", "BACKGROUND"];
                let colours: Arc<[watch::Colour]> = (0..3)
                    .map(|i| watch::Colour {
                        name: names[i].into(),
                        value: sources[i].show_styled(style),
                        rgba: rgbas[i],
                    })
                    .collect();
                *shown = Some((sources, colours.clone()));
                colours
            }
        };
        Watch {
            counters,
            scopes,
            colours,
        }
    }

    /// A special variable still at its starting value, which the panel
    /// doesn't show.
    fn is_default_special(&self, sym: Sym, value: &Value) -> bool {
        if sym == self.specials.logoversion {
            return true;
        }
        sym == self.specials.caseignoredp && value.as_bool() == Some(true)
    }

    /// Checks the cancel flag and the limit on instructions. Loops do this
    /// on every pass, so even `FOREVER []` can be stopped.
    pub(crate) fn tick(&mut self, span: Span) -> Result<(), Flow> {
        if self.control.cancel.load(Ordering::Relaxed) {
            return Err(Flow::Halt);
        }
        self.stats.instructions += 1;
        self.publish_instructions();
        if let Some(max) = self.limits.max_instructions
            && self.stats.instructions > max
        {
            let message = format!("Stopped after {max} instructions: does this program ever end?");
            return Err(Error::new(message, Some(span)).into());
        }
        Ok(())
    }

    /// Checks the cancel flag and sends the `Instruction` event, after a
    /// `Watch` of what the instruction sees if that's wanted. In lockstep,
    /// waits for the front end before starting a line.
    fn begin(&mut self, span: Span) -> Result<(), Flow> {
        self.tick(span)?;
        if self.limits.instruction_events {
            let line = Some((span.source, span.line));
            let starts_line = self.last_line != line;
            self.last_line = line;
            let lockstep = self.control.lockstep.load(Ordering::Relaxed);
            // Only the editor's program stops at breakpoints (ADR 0029).
            let breakpoint = starts_line
                && span.source == SourceId::Editor
                && self.control.is_breakpoint(span.line);
            // The panels show what a paused instruction sees, even out of
            // lockstep.
            if lockstep || breakpoint || self.control.watch.load(Ordering::Relaxed) {
                self.control.watch.store(false, Ordering::Relaxed);
                self.send_watch()?;
            }
            if breakpoint {
                self.emit(Event::Breakpoint)?;
            }
            self.emit(Event::Instruction { span, starts_line })?;
            if starts_line {
                self.control.lines_started.fetch_add(1, Ordering::Relaxed);
                if lockstep || breakpoint {
                    self.sync()?;
                }
            }
        }
        Ok(())
    }

    /// Sends a `Sync`, flushing any batched events so the front end can
    /// reach it, and waits for the reply.
    pub(crate) fn sync(&mut self) -> Result<(), Flow> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.emit(Event::Sync(SyncPoint(tx)))?;
        self.sink.flush().map_err(|_| Flow::Halt)?;
        let waiting = self.control.metrics.lockstep.start();
        let reply = rx.recv();
        self.control.metrics.lockstep.stop(waiting);
        reply.map_err(|_| Flow::Halt)?;
        if self.control.cancel.load(Ordering::Relaxed) {
            return Err(Flow::Halt);
        }
        Ok(())
    }

    fn exec_instr(&mut self, instr: &Instr, tail: bool) -> Result<Option<Tail>, Flow> {
        self.begin(instr.span)?;
        self.exec(&instr.expr, tail)
    }

    /// Runs an expression as an instruction, which must not output. In tail
    /// position, a call to a user procedure is handed back instead.
    fn exec(&mut self, expr: &Expr, tail: bool) -> Result<Option<Tail>, Flow> {
        let Expr::Call(call) = expr else {
            let v = self.eval(expr, "")?;
            return Err(dont_say(&v, expr.span()));
        };
        match call.target {
            Target::User(sym) => {
                let proc = self.lookup(sym, call)?;
                let args = self.eval_args(call)?;
                if tail {
                    return Ok(Some(Tail {
                        proc,
                        args,
                        span: call.span,
                        wants_output: false,
                    }));
                }
                match self.call_user(proc, args, call.span)? {
                    Some(v) => Err(dont_say(&v, call.span)),
                    None => Ok(None),
                }
            }
            Target::Prim(id) => self.exec_prim(id, call, tail),
            Target::Infix(_) => {
                let v = self.eval(expr, "")?;
                Err(dont_say(&v, call.span))
            }
        }
    }

    fn exec_prim(&mut self, id: usize, call: &Call, tail: bool) -> Result<Option<Tail>, Flow> {
        match PRIMS[id].imp {
            Imp::F(f) => {
                let vals = self.eval_args(call)?;
                match f(self, &Args { call, vals: &vals })? {
                    Some(v) => Err(dont_say(&v, call.span)),
                    None => Ok(None),
                }
            }
            Imp::If if call.args.len() == 3 => self.exec_ifelse(call, tail),
            Imp::If => {
                if self.eval_bool(call)? {
                    let list = self.eval_list(call, 1)?;
                    return self.run_list(&list, call.span, tail);
                }
                Ok(None)
            }
            Imp::IfElse => self.exec_ifelse(call, tail),
            Imp::Repeat => {
                let count_arg = [self.eval(&call.args[0], &call.name)?];
                let args = Args {
                    call,
                    vals: &count_arg,
                };
                let count = args.int(0)?;
                if count < 0 {
                    return Err(args.doesnt_like(0));
                }
                let list = self.eval_list(call, 1)?;
                let block = self.list_block(&list, call.span)?;
                self.counting(&call.name, Some(count), true, |it| {
                    for i in 1..=count {
                        it.tick(call.span)?;
                        it.set_count(i);
                        it.run_block(&block, false)?;
                    }
                    Ok(None)
                })
            }
            Imp::Forever => {
                let list = self.eval_list(call, 0)?;
                let block = self.list_block(&list, call.span)?;
                self.counting(&call.name, None, true, |it| {
                    for i in 1.. {
                        it.tick(call.span)?;
                        it.set_count(i);
                        it.run_block(&block, false)?;
                    }
                    unreachable!("FOREVER only ends with an error, STOP or a halt")
                })
            }
            Imp::Stop => Err(Flow::Stop(call.span)),
            Imp::Output => {
                let arg = &call.args[0];
                if tail {
                    return match self.eval_tail(arg, &call.name)? {
                        Ret::Value(v) => Err(Flow::Output(v, call.span)),
                        Ret::Tail(t) => Ok(Some(t)),
                    };
                }
                Err(Flow::Output(self.eval(arg, &call.name)?, call.span))
            }
        }
    }

    fn exec_ifelse(&mut self, call: &Call, tail: bool) -> Result<Option<Tail>, Flow> {
        let which = if self.eval_bool(call)? { 1 } else { 2 };
        let list = self.eval_list(call, which)?;
        self.run_list(&list, call.span, tail)
    }

    fn eval_bool(&mut self, call: &Call) -> Result<bool, Flow> {
        let v = self.eval(&call.args[0], &call.name)?;
        v.as_bool().ok_or_else(|| {
            Args {
                call,
                vals: std::slice::from_ref(&v),
            }
            .doesnt_like(0)
        })
    }

    fn eval_list(&mut self, call: &Call, i: usize) -> Result<Rc<List>, Flow> {
        match self.eval(&call.args[i], &call.name)? {
            Value::List(list) => Ok(list),
            v => {
                let mut vals = vec![Value::Num(0.0); i];
                vals.push(v);
                Err(Args { call, vals: &vals }.doesnt_like(i))
            }
        }
    }

    /// The list's instructions, parsed the first time it runs. Lists from the
    /// source keep the parse; lists built at run time are parsed each time,
    /// since their fallback span depends on where they run.
    fn list_block(&mut self, list: &List, fallback: Span) -> Result<Rc<Block>, Flow> {
        if let Some(block) = list.parsed.get() {
            return Ok(block.clone());
        }
        let used = self.stack_base.saturating_sub(stack_addr());
        if self
            .limits
            .stack_bytes
            .is_some_and(|bytes| used + STACK_MARGIN > bytes)
        {
            return Err(Error::new(
                "Lists are being run inside each other too deeply",
                Some(fallback),
            )
            .into());
        }
        let procs = &self.procs;
        let known = |sym: Sym| -> Option<Arity> {
            procs
                .get(sym as usize)?
                .as_ref()
                .map(|p| Arity::fixed(p.inputs.len()))
        };
        let mut cx = Cx {
            names: &mut self.names,
            known: &known,
            defining: Default::default(),
        };
        let block = Rc::new(parse::parse_list(list, fallback, &mut cx)?);
        self.grow();
        if list.spans.is_some() {
            let _ = list.parsed.set(block.clone());
        }
        Ok(block)
    }

    fn run_list(&mut self, list: &List, fallback: Span, tail: bool) -> Result<Option<Tail>, Flow> {
        let block = self.list_block(list, fallback)?;
        self.run_block(&block, tail)
    }

    /// Runs a list as instructions, as the loops do with their bodies.
    pub(crate) fn run_commands(&mut self, list: &List, fallback: Span) -> Result<(), Flow> {
        self.run_list(list, fallback, false).map(|_| ())
    }

    /// Runs a list as `RUN` does: it outputs if its last instruction does.
    pub(crate) fn run_maybe(&mut self, list: &List, fallback: Span) -> Result<Option<Value>, Flow> {
        let block = self.list_block(list, fallback)?;
        self.run_block_maybe(&block)
    }

    /// Parses a list as instructions, for templates, which are parsed once
    /// and run many times.
    pub(crate) fn parse_block(&mut self, list: &List, fallback: Span) -> Result<Rc<Block>, Flow> {
        self.list_block(list, fallback)
    }

    pub(crate) fn run_block_maybe(&mut self, block: &Block) -> Result<Option<Value>, Flow> {
        let Some((last, rest)) = block.split_last() else {
            return Ok(None);
        };
        self.run_block_slice(rest)?;
        self.begin(last.span)?;
        self.eval_maybe(&last.expr)
    }

    /// Evaluates an expression that may or may not output, as the last
    /// instruction of a `RUN` list.
    pub(crate) fn eval_maybe(&mut self, expr: &Expr) -> Result<Option<Value>, Flow> {
        let Expr::Call(call) = expr else {
            return Ok(Some(self.eval(expr, "")?));
        };
        match call.target {
            Target::Infix(_) => Ok(Some(self.eval(expr, "")?)),
            Target::User(sym) => {
                let proc = self.lookup(sym, call)?;
                let args = self.eval_args(call)?;
                self.call_user(proc, args, call.span)
            }
            Target::Prim(id) => match PRIMS[id].imp {
                Imp::F(f) => {
                    let vals = self.eval_args(call)?;
                    f(self, &Args { call, vals: &vals })
                }
                Imp::IfElse => {
                    let which = if self.eval_bool(call)? { 1 } else { 2 };
                    let list = self.eval_list(call, which)?;
                    self.run_maybe(&list, call.span)
                }
                Imp::If if call.args.len() == 3 => {
                    let which = if self.eval_bool(call)? { 1 } else { 2 };
                    let list = self.eval_list(call, which)?;
                    self.run_maybe(&list, call.span)
                }
                _ => self.exec_prim(id, call, false).map(|_| None),
            },
        }
    }

    /// Runs `f` with `syms` as local variables holding `values`, as a lambda
    /// template's inputs and `FOR`'s control variable are. They get their
    /// old values back however `f` ends.
    pub(crate) fn with_locals<T>(
        &mut self,
        syms: &[Sym],
        values: Vec<Value>,
        label: &Rc<str>,
        span: Span,
        f: impl FnOnce(&mut Interpreter) -> Result<T, Flow>,
    ) -> Result<T, Flow> {
        let mut saved = SmallVec::new();
        for (&sym, v) in syms.iter().zip(values) {
            saved.push((sym, self.vars[sym as usize].replace(v)));
        }
        self.frames.push(Frame {
            proc: None,
            label: Some(label.clone()),
            call_span: span,
            saved,
        });
        let result = f(self);
        let frame = self.frames.pop().expect("pushed above");
        for (sym, old) in frame.saved.into_iter().rev() {
            self.vars[sym as usize] = old;
        }
        result
    }

    /// Runs `f` with a counter for the loop or template `name`, which
    /// `set_count` moves on. `repcount` says whether `REPCOUNT` sees it.
    pub(crate) fn counting<T>(
        &mut self,
        name: &Rc<str>,
        of: Option<i64>,
        repcount: bool,
        f: impl FnOnce(&mut Interpreter) -> Result<T, Flow>,
    ) -> Result<T, Flow> {
        self.counters.push(Counter {
            name: name.clone(),
            count: 0,
            of,
            repcount,
        });
        let result = f(self);
        self.counters.pop();
        result
    }

    /// The innermost counter's pass.
    pub(crate) fn set_count(&mut self, count: i64) {
        self.counters.last_mut().expect("inside counting").count = count;
    }

    /// What `REPCOUNT` outputs: the innermost `REPEAT`, `FOREVER` or
    /// template's pass.
    pub(crate) fn repcount(&self) -> Option<i64> {
        self.counters
            .iter()
            .rev()
            .find(|c| c.repcount)
            .map(|c| c.count)
    }

    /// What a template's name stands for, if it's a procedure: for `APPLY
    /// "SUM` and the like.
    pub(crate) fn procedure_named(&mut self, name: &str) -> Option<(Target, Arity)> {
        if let Some(id) = prims::lookup(name) {
            return Some((Target::Prim(id), PRIMS[id].arity));
        }
        let sym = self.intern(name);
        self.procs[sym as usize]
            .as_ref()
            .map(|p| (Target::User(sym), Arity::fixed(p.inputs.len())))
    }

    /// Calls a procedure with values already worked out, as a template does.
    pub(crate) fn call_with(
        &mut self,
        target: Target,
        name: Rc<str>,
        args: Vec<Value>,
        span: Span,
    ) -> Result<Option<Value>, Flow> {
        let args = args.into_iter().map(|v| Expr::Lit(v, span)).collect();
        let call = Expr::Call(Box::new(Call {
            target,
            name,
            args,
            span,
        }));
        self.eval_maybe(&call)
    }

    /// Calls a procedure made from a template's text.
    pub(crate) fn call_proc(
        &mut self,
        proc: Rc<Proc>,
        args: Vec<Value>,
        span: Span,
    ) -> Result<Option<Value>, Flow> {
        self.call_user(proc, args.into_iter().collect(), span)
    }

    /// Parses lines of procedure text into a procedure, for a
    /// procedure-text template, `[[x] [line] [line]]`.
    pub(crate) fn make_proc(
        &mut self,
        name: &str,
        inputs: Vec<Sym>,
        lines: &[Value],
        fallback: Span,
    ) -> Result<Rc<Proc>, Flow> {
        let mut body = Vec::new();
        for line in lines {
            let Value::List(list) = line else {
                unreachable!("procedure text is lists")
            };
            let block = self.list_block(list, fallback)?;
            body.extend(block.iter().cloned());
        }
        Ok(Rc::new(Proc {
            name: name.into(),
            inputs,
            body,
            title: fallback,
        }))
    }

    /// Runs instructions. With `tail`, the last one is in tail position.
    fn run_block(&mut self, block: &Block, tail: bool) -> Result<Option<Tail>, Flow> {
        let last = block.len().wrapping_sub(1);
        for (i, instr) in block.iter().enumerate() {
            if let Some(t) = self.exec_instr(instr, tail && i == last)? {
                return Ok(Some(t));
            }
        }
        Ok(None)
    }

    /// Runs a list as an expression that outputs a value, as `AND`, `OR` and
    /// `NOT` do with list inputs.
    pub(crate) fn run_expression(&mut self, list: &List, call: &Call) -> Result<Value, Flow> {
        match self.run_list_value(list, call, false)? {
            Ret::Value(v) => Ok(v),
            Ret::Tail(_) => unreachable!("not in tail position"),
        }
    }

    /// Runs a list whose last instruction outputs, as `IFELSE` does when
    /// it's used as an operation.
    fn run_list_value(&mut self, list: &List, call: &Call, tail: bool) -> Result<Ret, Flow> {
        let block = self.list_block(list, call.span)?;
        let Some((last, rest)) = block.split_last() else {
            return Err(didnt_output(&call.name, "", call.span));
        };
        self.run_block_slice(rest)?;
        self.begin(last.span)?;
        if tail {
            self.eval_tail(&last.expr, &call.name)
        } else {
            Ok(Ret::Value(self.eval(&last.expr, &call.name)?))
        }
    }

    fn run_block_slice(&mut self, instrs: &[Instr]) -> Result<(), Flow> {
        for instr in instrs {
            self.exec_instr(instr, false)?;
        }
        Ok(())
    }

    /// Evaluates the input to `OUTPUT` in tail position: a call to a user
    /// procedure, directly or as the last thing an `IFELSE` runs, is handed
    /// back as a tail call.
    fn eval_tail(&mut self, expr: &Expr, to: &str) -> Result<Ret, Flow> {
        if let Expr::Call(call) = expr {
            match call.target {
                Target::User(sym) => {
                    let proc = self.lookup(sym, call)?;
                    let args = self.eval_args(call)?;
                    return Ok(Ret::Tail(Tail {
                        proc,
                        args,
                        span: call.span,
                        wants_output: true,
                    }));
                }
                Target::Prim(id) if matches!(PRIMS[id].imp, Imp::IfElse) => {
                    let which = if self.eval_bool(call)? { 1 } else { 2 };
                    let list = self.eval_list(call, which)?;
                    return self.run_list_value(&list, call, true);
                }
                _ => {}
            }
        }
        Ok(Ret::Value(self.eval(expr, to)?))
    }

    /// Evaluates an expression that must output. `to` names what the value
    /// is for, for the "didn't output to" error.
    fn eval(&mut self, expr: &Expr, to: &str) -> Result<Value, Flow> {
        match expr {
            Expr::Lit(v, _) => Ok(v.clone()),
            Expr::Var(sym, span) => match &self.vars[*sym as usize] {
                Some(v) => Ok(v.clone()),
                None => Err(Error::new(
                    format!(":{} has no value", self.names.spelling(*sym)),
                    Some(*span),
                )
                .into()),
            },
            Expr::Neg(operand, span) => {
                let v = self.eval(operand, "-")?;
                match v.as_num() {
                    Some(n) => Ok(Value::Num(-n)),
                    None => Err(Error::new(
                        format!("- doesn't like {} as input", v.show()),
                        Some(*span),
                    )
                    .into()),
                }
            }
            Expr::Call(call) => match call.target {
                Target::Infix(op) => {
                    let vals = [
                        self.eval(&call.args[0], op.symbol())?,
                        self.eval(&call.args[1], op.symbol())?,
                    ];
                    let Imp::F(f) = prims::infix_prim(op).imp else {
                        unreachable!("operators are plain primitives")
                    };
                    Ok(f(self, &Args { call, vals: &vals })?.expect("operators always output"))
                }
                Target::User(sym) => {
                    let proc = self.lookup(sym, call)?;
                    let args = self.eval_args(call)?;
                    match self.call_user(proc, args, call.span)? {
                        Some(v) => Ok(v),
                        None => Err(didnt_output(&call.name, to, call.span)),
                    }
                }
                Target::Prim(id) => match PRIMS[id].imp {
                    Imp::F(f) => {
                        let vals = self.eval_args(call)?;
                        match f(self, &Args { call, vals: &vals })? {
                            Some(v) => Ok(v),
                            None => Err(didnt_output(&call.name, to, call.span)),
                        }
                    }
                    Imp::IfElse => {
                        let which = if self.eval_bool(call)? { 1 } else { 2 };
                        let list = self.eval_list(call, which)?;
                        match self.run_list_value(&list, call, false)? {
                            Ret::Value(v) => Ok(v),
                            Ret::Tail(_) => unreachable!("not in tail position"),
                        }
                    }
                    _ => {
                        self.exec_prim(id, call, false)?;
                        Err(didnt_output(&call.name, to, call.span))
                    }
                },
            },
        }
    }

    fn eval_args(&mut self, call: &Call) -> Result<ArgVec, Flow> {
        let mut args = ArgVec::new();
        for arg in &call.args {
            args.push(self.eval(arg, &call.name)?);
        }
        Ok(args)
    }

    fn lookup(&self, sym: Sym, call: &Call) -> Result<Rc<Proc>, Flow> {
        match &self.procs[sym as usize] {
            Some(proc) => Ok(proc.clone()),
            None => Err(Error::unknown(&call.name, Some(call.span)).into()),
        }
    }

    /// Calls a user procedure, then runs any tail calls it hands back in
    /// the same frame, without growing the Rust stack.
    fn call_user(
        &mut self,
        proc: Rc<Proc>,
        args: ArgVec,
        span: Span,
    ) -> Result<Option<Value>, Flow> {
        self.check_depth(&proc, span)?;
        let mut saved = SmallVec::new();
        for (&sym, v) in proc.inputs.iter().zip(args) {
            saved.push((sym, self.vars[sym as usize].replace(v)));
        }
        self.frames.push(Frame {
            proc: Some(proc.clone()),
            label: None,
            call_span: span,
            saved,
        });

        let mut current = proc;
        // Set once any tail call was written as an instruction, after which
        // an output has nowhere to go.
        let mut discard: Option<Span> = None;
        let result = loop {
            match self.run_block(&current.body, true) {
                Ok(None) | Err(Flow::Stop(_)) => break Ok(None),
                Err(Flow::Output(v, _)) => break Ok(Some(v)),
                Ok(Some(tail)) => {
                    if !tail.wants_output {
                        discard = Some(tail.span);
                    }
                    let frame = self.frames.last_mut().expect("pushed above");
                    for (&sym, v) in tail.proc.inputs.iter().zip(tail.args) {
                        let slot = &mut self.vars[sym as usize];
                        if frame.saved.iter().any(|(s, _)| *s == sym) {
                            *slot = Some(v);
                        } else {
                            frame.saved.push((sym, slot.replace(v)));
                        }
                    }
                    frame.proc = Some(tail.proc.clone());
                    frame.call_span = tail.span;
                    current = tail.proc;
                }
                // `(THROW "ERROR message)` is an error where this procedure
                // was called, as if it were a primitive.
                Err(Flow::Throw {
                    tag,
                    value: Some(v),
                    ..
                }) if tag.eq_ignore_ascii_case("error") => {
                    let call_span = self.frames.last().expect("pushed above").call_span;
                    let e = Error::new(v.print_form(), Some(call_span));
                    break Err(match self.run_erract(&e) {
                        Ok(()) => Flow::Error(Box::new(e)),
                        Err(other) => other,
                    });
                }
                Err(Flow::Throw { tag, value, span }) => {
                    break Err(Flow::Throw { tag, value, span });
                }
                Err(Flow::Error(mut e)) => {
                    if e.calls.is_empty()
                        && let Err(other) = self.run_erract(&e)
                    {
                        break Err(other);
                    }
                    let frame = self.frames.last().expect("pushed above");
                    let name = frame
                        .proc
                        .as_ref()
                        .expect("a procedure's frame")
                        .name
                        .to_string();
                    e.calls.push(CallSite {
                        procedure: name,
                        span: frame.call_span,
                    });
                    break Err(Flow::Error(e));
                }
                Err(Flow::Halt) => break Err(Flow::Halt),
            }
        };

        let frame = self.frames.pop().expect("pushed above");
        for (sym, old) in frame.saved.into_iter().rev() {
            self.vars[sym as usize] = old;
        }
        match (result?, discard) {
            (Some(v), Some(span)) => Err(dont_say(&v, span)),
            (result, _) => Ok(result),
        }
    }

    fn check_depth(&mut self, proc: &Proc, span: Span) -> Result<(), Flow> {
        let depth = self.frames.len() + 1;
        let used = self.stack_base.saturating_sub(stack_addr());
        self.stats.max_depth = self.stats.max_depth.max(depth);
        self.stats.max_stack = self.stats.max_stack.max(used);
        let out_of_stack = self
            .limits
            .stack_bytes
            .is_some_and(|bytes| used + STACK_MARGIN > bytes);
        if depth <= self.limits.max_depth && !out_of_stack {
            return Ok(());
        }
        let calls_itself = self
            .frames
            .last()
            .and_then(|f| f.proc.as_ref())
            .is_some_and(|p| std::ptr::eq(Rc::as_ptr(p), proc));
        let name = proc.name.to_uppercase();
        let message = if calls_itself {
            format!(
                "{name} called itself too many times: {depth} calls deep. Does it have a way to STOP?"
            )
        } else {
            format!(
                "Too many procedures were running inside each other ({depth} deep) when {name} was called"
            )
        };
        Err(Error::new(message, Some(span)).into())
    }
}
