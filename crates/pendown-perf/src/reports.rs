//! Reports first made for spike 03's `bench`: stack used per call, and
//! memory over a long tail-recursive run. `perf --stack` and `perf --memory`
//! print them. The speed measurements are timed cases now (`cases.rs`).

use std::{
    sync::{Arc, atomic::Ordering, mpsc},
    thread,
    time::Duration,
};

use pendown_logo::{Event, Interpreter, Limits, RunError};

use crate::{Discard, MB, STACK, interpreter, on_stack};

pub fn stack_per_call() {
    println!("## Stack used per call that isn't a tail call\n");
    let shapes = [
        (
            "command, then more",
            "TO D :n\n IF :n = 0 [STOP]\n D :n - 1\n FD 0\nEND\nD 2000",
        ),
        (
            "inside an expression",
            "TO E :n\n IF :n = 0 [OUTPUT 0]\n OUTPUT 1 + E :n - 1\nEND\nPRINT E 2000",
        ),
        (
            "fib-style, two calls",
            "TO F :n\n IF :n < 1 [OUTPUT 0]\n OUTPUT (F :n - 1) + (F :n - 2000)\nEND\nPRINT F 2000",
        ),
        (
            "inside REPEAT and IFELSE lists",
            "TO N :n\n IF :n > 0 [REPEAT 1 [IFELSE \"true [N :n - 1 FD 0] []]]\nEND\nN 2000",
        ),
    ];
    let mut worst = 0;
    for (name, src) in shapes {
        let (depth, stack) = on_stack(move || {
            let limits = Limits {
                max_depth: usize::MAX,
                stack_bytes: Some(STACK),
                instruction_events: true,
                max_instructions: None,
            };
            let mut it = interpreter(Box::new(Discard), limits);
            it.run_program(src).unwrap();
            (it.stats.max_depth, it.stats.max_stack)
        });
        let per_call = stack / (depth - 1);
        worst = worst.max(per_call);
        println!(
            "- {name}: {depth} calls deep used {} KiB, {per_call} bytes a call",
            stack / 1024
        );
    }
    let fits = (STACK - pendown_logo::STACK_MARGIN) / worst;
    println!("\nWorst case {worst} bytes a call, so a 256 MiB stack fits about {fits} calls.");

    // How deep it really gets before the stack guard stops it.
    let src =
        "TO DOWN :n\n IF :n > 0 [REPEAT 1 [IFELSE \"true [DOWN :n + 1 FD 0] []]]\nEND\nDOWN 1";
    let (depth, message) = on_stack(move || {
        let limits = Limits {
            max_depth: usize::MAX,
            stack_bytes: Some(STACK),
            instruction_events: false,
            max_instructions: None,
        };
        let mut it = interpreter(Box::new(Discard), limits);
        let result = it.run_program(src);
        let Err(RunError::Runtime(e)) = result else {
            panic!("{result:?}")
        };
        (it.stats.max_depth, e.message)
    });
    println!(
        "Endless recursion through lists, no depth limit: stopped by the stack guard at {depth} calls: \"{message}\"\n"
    );
}

fn rss_mib() -> f64 {
    let statm = std::fs::read_to_string("/proc/self/statm").unwrap();
    let pages: f64 = statm.split_whitespace().nth(1).unwrap().parse().unwrap();
    pages * 4096.0 / MB as f64
}

const SPIRAL: &str = "TO SPIRAL :n FD :n RT 90 SPIRAL :n + 1 END\nSPIRAL 1";

pub fn memory() {
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
