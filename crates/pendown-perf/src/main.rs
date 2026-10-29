//! `perf` (ADR 0020): times fixed programs through the interpreter, the
//! renderer and the animator, with no window, and compares the times with
//! `perf/baseline.json`. It fails if anything is more than 20% slower.
//!
//! ```text
//! cargo run --release -p pendown-perf             # compare with the baseline
//! cargo run --release -p pendown-perf -- --save   # write a new baseline
//! cargo run --release -p pendown-perf -- tree     # only the cases named *tree*
//! cargo run --release -p pendown-perf -- --stack --memory   # spike 03's reports
//! ```
//!
//! A baseline belongs to the machine it was measured on. On another machine,
//! `perf` says so and doesn't fail; save a baseline for that machine first.
//! Saving a new baseline is a deliberate commit, saying why.

mod cases;
mod reports;

use std::{path::PathBuf, process::ExitCode, sync::Arc, thread, time::Duration};

use pendown_logo::{Event, EventSink, Halted, Interpreter, Limits};
use serde_json::{Map, Value, json};

pub(crate) const MB: usize = 1024 * 1024;
pub(crate) const STACK: usize = 256 * MB;

/// How much slower than the baseline counts as a regression.
const SLOWER: f64 = 1.2;

/// Each case runs this many times, and the fastest counts.
const RUNS: usize = 7;

/// Throws events away, as fast as possible.
pub(crate) struct Discard;

impl EventSink for Discard {
    fn send(&mut self, _: Event) -> Result<(), Halted> {
        Ok(())
    }
}

pub(crate) fn on_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    thread::Builder::new()
        .stack_size(STACK)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap()
}

pub(crate) fn interpreter(sink: Box<dyn EventSink>, limits: Limits) -> Interpreter {
    let mut it = Interpreter::new(sink, Arc::default());
    it.limits = Limits {
        stack_bytes: Some(STACK),
        ..limits
    };
    it
}

fn baseline_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../perf/baseline.json")
}

/// The CPU's model and how many there are, from `/proc/cpuinfo` on Linux.
fn machine() -> String {
    let model = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|info| {
            info.lines().find_map(|l| {
                l.strip_prefix("model name")?
                    .split_once(':')
                    .map(|(_, m)| m.trim().to_string())
            })
        })
        .unwrap_or_else(|| "unknown CPU".into());
    let cpus = thread::available_parallelism().map_or(1, |n| n.get());
    format!("{model}, {cpus} CPUs, {}", std::env::consts::OS)
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().any(|a| a == name);
    if cfg!(debug_assertions) {
        eprintln!("perf times release builds: cargo run --release -p pendown-perf");
        return ExitCode::from(2);
    }
    if flag("--stack") || flag("--memory") {
        if flag("--stack") {
            reports::stack_per_call();
        }
        if flag("--memory") {
            reports::memory();
        }
        return ExitCode::SUCCESS;
    }
    let filter = args.iter().find(|a| !a.starts_with("--"));
    let save = flag("--save");

    let baseline: Option<Value> = std::fs::read_to_string(baseline_path())
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok());
    let machine = machine();
    let same_machine = baseline
        .as_ref()
        .is_some_and(|b| b["machine"] == machine.as_str());
    println!("# perf, best of {RUNS} runs, on {machine}\n");
    match &baseline {
        None => println!("There's no baseline yet: `perf --save` makes one.\n"),
        Some(b) if !same_machine => println!(
            "The baseline is from another machine ({}), so nothing fails: `perf --save` makes one for this machine.\n",
            b["machine"]
        ),
        Some(_) => {}
    }
    println!("| Case | Time | Baseline | Change |");
    println!("|---|---|---|---|");

    let mut results = Map::new();
    let mut regressions = Vec::new();
    for case in cases::CASES
        .iter()
        .filter(|c| filter.is_none_or(|f| c.name.contains(f.as_str())))
    {
        let best = (0..RUNS).map(|_| (case.run)()).min().expect("RUNS > 0");
        results.insert(case.name.into(), json!((ms(best) * 100.0).round() / 100.0));
        let was = baseline
            .as_ref()
            .and_then(|b| b["results_ms"][case.name].as_f64());
        let (was_text, change) = match was {
            Some(was) => {
                let ratio = ms(best) / was;
                let verdict = if ratio > SLOWER {
                    if same_machine {
                        regressions.push(case.name);
                    }
                    " **slower**"
                } else if ratio < 1.0 / SLOWER {
                    " faster"
                } else {
                    ""
                };
                (
                    format!("{was:.1} ms"),
                    format!("{:+.0}%{verdict}", (ratio - 1.0) * 100.0),
                )
            }
            None => ("".into(), "new".into()),
        };
        println!(
            "| {} | {:.1} ms | {was_text} | {change} |",
            case.name,
            ms(best)
        );
    }

    if save {
        // Saving some cases keeps the baseline's others.
        let mut all = baseline
            .filter(|_| same_machine)
            .and_then(|b| b["results_ms"].as_object().cloned())
            .unwrap_or_default();
        all.extend(results);
        let out = json!({ "machine": machine, "results_ms": all });
        let text = serde_json::to_string_pretty(&out).expect("JSON") + "\n";
        if let Err(e) = std::fs::write(baseline_path(), text) {
            eprintln!("Can't save the baseline: {e}");
            return ExitCode::FAILURE;
        }
        println!("\nSaved the baseline. Commit it on its own, saying why.");
        return ExitCode::SUCCESS;
    }
    if regressions.is_empty() {
        println!(
            "\nNothing is more than {:.0}% slower than the baseline.",
            (SLOWER - 1.0) * 100.0
        );
        ExitCode::SUCCESS
    } else {
        println!(
            "\n{} slower than the baseline by more than {:.0}%:",
            regressions.len(),
            (SLOWER - 1.0) * 100.0
        );
        for name in regressions {
            println!("- {name}");
        }
        println!(
            "\nFix the regression, or if it's deliberate, save a new baseline in its own commit saying why."
        );
        ExitCode::FAILURE
    }
}
