//! Measurements first made for spike 03: stack used per call, speed, and
//! memory over a long tail-recursive run. Run with
//! `cargo run --release --bin bench`, and again without `--release`.

use std::{
    sync::{
        Arc,
        atomic::Ordering,
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

use pendown_logo::{BatchSender, Event, EventSink, Halted, Interpreter, Limits, RunError};

const MB: usize = 1024 * 1024;
const STACK: usize = 256 * MB;

/// Throws events away, as fast as possible.
struct Discard;

impl EventSink for Discard {
    fn send(&mut self, _: Event) -> Result<(), Halted> {
        Ok(())
    }
}

fn on_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    thread::Builder::new().stack_size(STACK).spawn(f).unwrap().join().unwrap()
}

fn interpreter(sink: Box<dyn EventSink>, limits: Limits) -> Interpreter {
    let mut it = Interpreter::new(sink, Arc::default());
    it.limits = limits;
    it
}

fn stack_per_call() {
    println!("## Stack used per call that isn't a tail call\n");
    let shapes = [
        ("command, then more", "TO D :n\n IF :n = 0 [STOP]\n D :n - 1\n FD 0\nEND\nD 2000"),
        ("inside an expression", "TO E :n\n IF :n = 0 [OUTPUT 0]\n OUTPUT 1 + E :n - 1\nEND\nPRINT E 2000"),
        ("fib-style, two calls", "TO F :n\n IF :n < 1 [OUTPUT 0]\n OUTPUT (F :n - 1) + (F :n - 2000)\nEND\nPRINT F 2000"),
        (
            "inside REPEAT and IFELSE lists",
            "TO N :n\n IF :n > 0 [REPEAT 1 [IFELSE \"true [N :n - 1 FD 0] []]]\nEND\nN 2000",
        ),
    ];
    let mut worst = 0;
    for (name, src) in shapes {
        let (depth, stack) = on_stack(move || {
            let limits = Limits { max_depth: usize::MAX, stack_bytes: Some(STACK), instruction_events: true, max_instructions: None };
            let mut it = interpreter(Box::new(Discard), limits);
            it.run_program(src).unwrap();
            (it.stats.max_depth, it.stats.max_stack)
        });
        let per_call = stack / (depth - 1);
        worst = worst.max(per_call);
        println!("- {name}: {depth} calls deep used {} KiB, {per_call} bytes a call", stack / 1024);
    }
    let fits = (STACK - pendown_logo::STACK_MARGIN) / worst;
    println!("\nWorst case {worst} bytes a call, so a 256 MiB stack fits about {fits} calls.");

    // How deep it really gets before the stack guard stops it.
    let src = "TO DOWN :n\n IF :n > 0 [REPEAT 1 [IFELSE \"true [DOWN :n + 1 FD 0] []]]\nEND\nDOWN 1";
    let (depth, message) = on_stack(move || {
        let limits = Limits { max_depth: usize::MAX, stack_bytes: Some(STACK), instruction_events: false, max_instructions: None };
        let mut it = interpreter(Box::new(Discard), limits);
        let result = it.run_program(src);
        let Err(RunError::Runtime(e)) = result else { panic!("{result:?}") };
        (it.stats.max_depth, e.message)
    });
    println!("Endless recursion through lists, no depth limit: stopped by the stack guard at {depth} calls: \"{message}\"\n");
}

/// Runs `src` and reports instructions and events per second.
#[derive(Clone, Copy, PartialEq)]
enum Via {
    Direct,
    Channel,
    Batches,
}

fn speed(name: &str, src: &'static str, events: bool, via: Via) {
    let (instructions, sent, elapsed) = on_stack(move || {
        let limits = Limits { instruction_events: events, ..Limits::default() };
        let (sink, drain): (Box<dyn EventSink>, _) = match via {
            Via::Direct => (Box::new(Counted(0)), None),
            Via::Channel => {
                let (tx, rx) = mpsc::sync_channel::<Event>(1024);
                (Box::new(tx), Some(thread::spawn(move || rx.into_iter().count() as u64)))
            }
            Via::Batches => {
                let (tx, rx) = mpsc::sync_channel::<Vec<Event>>(4);
                let sink = BatchSender::new(tx, 256, Duration::from_millis(4));
                (Box::new(sink), Some(thread::spawn(move || rx.into_iter().map(|b| b.len() as u64).sum())))
            }
        };
        let mut it = interpreter(sink, limits);
        let start = Instant::now();
        it.run_program(src).unwrap();
        let instructions = it.stats.instructions;
        drop(it);
        let sent = drain.map_or(0, |d| d.join().unwrap());
        (instructions, sent, start.elapsed())
    });
    let secs = elapsed.as_secs_f64();
    let channel = via != Via::Direct;
    let via = match via {
        Via::Direct => "no channel",
        Via::Channel => "channel, one event a send",
        Via::Batches => "channel, batches of 256",
    };
    let ev = if events { "with Instruction events" } else { "no Instruction events" };
    let sent = if channel { format!(", {:.1} M events/s", sent as f64 / secs / 1e6) } else { String::new() };
    println!(
        "| {name} | {via}, {ev} | {:.1} M | {:.0} ms | {:.1} M/s{sent} |",
        instructions as f64 / 1e6,
        secs * 1e3,
        instructions as f64 / secs / 1e6
    );
}

struct Counted(u64);

impl EventSink for Counted {
    fn send(&mut self, _: Event) -> Result<(), Halted> {
        self.0 += 1;
        Ok(())
    }
}

const REPEAT: &str = "REPEAT 2000000 [FD 1 RT 1]";
const LOOP: &str = "TO LOOP :n\n IF :n = 0 [STOP]\n FD 1 RT 1\n LOOP :n - 1\nEND\nLOOP 1000000";
const FIB: &str = "TO FIB :n\n IF :n < 2 [OUTPUT :n]\n OUTPUT (FIB :n - 1) + (FIB :n - 2)\nEND\nMAKE \"x FIB 25";
const SPIRAL: &str = "TO SPIRAL :n FD :n RT 90 SPIRAL :n + 1 END\nSPIRAL 1";

fn speeds() {
    println!("## Speed\n");
    println!("| Program | Events | Instructions | Time | Instructions/s |");
    println!("|---|---|---|---|---|");
    for (name, src) in [("REPEAT [FD 1 RT 1]", REPEAT), ("tail-recursive loop", LOOP), ("FIB 25", FIB)] {
        speed(name, src, false, Via::Direct);
        speed(name, src, true, Via::Direct);
        speed(name, src, true, Via::Channel);
        speed(name, src, true, Via::Batches);
    }
    println!();
}

fn rss_mib() -> f64 {
    let statm = std::fs::read_to_string("/proc/self/statm").unwrap();
    let pages: f64 = statm.split_whitespace().nth(1).unwrap().parse().unwrap();
    pages * 4096.0 / MB as f64
}

fn memory() {
    println!("## Memory: SPIRAL for 20 s, through a channel, with Instruction events\n");
    let control: Arc<pendown_logo::Control> = Arc::default();
    let flag = control.clone();
    let (tx, rx) = mpsc::sync_channel::<Event>(1024);
    let interp = thread::Builder::new()
        .stack_size(STACK)
        .spawn(move || {
            let mut it = Interpreter::new(Box::new(tx), flag);
            it.limits.stack_bytes = Some(STACK);
            let result = it.run_program(SPIRAL);
            (result, it.stats)
        })
        .unwrap();
    let drain = thread::spawn(move || rx.into_iter().count());
    for s in 0..=20 {
        if s % 5 == 0 {
            println!("- {s:>2} s: RSS {:.1} MiB", rss_mib());
        }
        thread::sleep(Duration::from_secs(1));
    }
    control.cancel.store(true, Ordering::Relaxed);
    let (result, stats) = interp.join().unwrap();
    let events = drain.join().unwrap();
    println!(
        "\n{result:?} after {:.0} M instructions and {:.0} M events; deepest {} call",
        stats.instructions as f64 / 1e6,
        events as f64 / 1e6,
        stats.max_depth
    );
}

fn main() {
    let profile = if cfg!(debug_assertions) { "debug" } else { "release" };
    println!("# pendown-logo bench, {profile} build\n");
    stack_per_call();
    speeds();
    if std::env::args().any(|a| a == "--memory") {
        memory();
    }
}
