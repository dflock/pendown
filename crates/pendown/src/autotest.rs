//! A scripted session for what can only be checked with a window (issue 03).
//! `PENDOWN_AUTOTEST=1 cargo run --release -p pendown` runs it, writes
//! `autotest.log` and screenshots into `target/autotest/`, and exits with an
//! error if any check fails. Set `PENDOWN_AUTOTEST` to a directory to write
//! there instead.

use std::{
    fmt::Write as _,
    path::PathBuf,
    time::{Duration, Instant},
};

use gpui_kit::*;
use pendown::{
    animator::{Mode, Speed},
    run::DEFAULT_CHANNEL,
    speed::level_speed,
};

use crate::app::App;

const SQUARE: &str = "REPEAT 4 [FD 300 RT 90]";

const STEPPER: &str = r#"MAKE "n 0
REPEAT 6 [
  MAKE "n :n + 1
  FD 50 * :n
  RT 60
  PRINT :n
]
PRINT "done
"#;

const FOREVER: &str = "FOREVER [FD 3 RT 1.5]";

const FOREVER_LINES: &str = "FOREVER [\n  FD 3\n  RT 1.5\n]";

const TREE: &str = r#"TO TREE :size :depth
  IF :depth = 0 [STOP]
  FD :size
  LT 30 TREE :size * 0.72 :depth - 1
  RT 60 TREE :size * 0.72 :depth - 1
  LT 30 BK :size
END
PU BK 450 PD
SETPC 10
TREE 230 16
"#;

const ERRORS: &str = "TO SQUARE :n\n  FD :n RT 90\n  FD \"far\n  PRINT 1\nEND\nSQUARE 100";

/// One step of the session.
#[derive(Clone)]
#[allow(clippy::enum_variant_names)]
enum Step {
    Say(&'static str),
    Run(&'static str, Speed),
    Wait(Duration),
    WaitFinished,
    WaitPaused,
    SetSpeed(Speed),
    Pause,
    StepLine,
    Stop,
    Screenshot(&'static str),
    /// Start recording the turtle's y position every frame.
    Track,
    /// Stop recording, and check the turtle's speed before and after the
    /// given time: about the two speeds, in steps a second.
    CheckSpeeds(Duration, f64, f64),
    /// Start recording every highlight change.
    Trace,
    /// Stop recording, and check the highlighted texts in order.
    CheckTrace(&'static [&'static str]),
    CheckHighlight(&'static str),
    /// The turtle is part-way through a move.
    CheckMoving,
    /// The interpreter has started exactly as many lines as the highlight
    /// has reached.
    CheckNothingAhead,
    /// Stop was pressed, and the interpreter thread exits within this long.
    CheckStopsWithin(Duration),
    CheckFinishedWithin(Duration),
    CheckFramesAtMost(Duration),
    CheckConsole(&'static str),
    Quit,
}

fn script() -> Vec<Step> {
    use Step::*;
    let ms = Duration::from_millis;
    vec![
        Say("The welcome program"),
        Run(crate::app::WELCOME, level_speed(10)),
        WaitFinished,
        CheckFinishedWithin(Duration::from_secs(30)),
        Say("Highlighting follows the turtle"),
        Trace,
        Run(SQUARE, Speed::Animated(100.0)),
        Wait(ms(1500)),
        CheckHighlight("FD 300"),
        CheckMoving,
        Screenshot("highlight-mid-move"),
        WaitFinished,
        CheckTrace(&["REPEAT 4 [FD 300 RT 90]", "FD 300", "RT 90", "FD 300", "RT 90", "FD 300", "RT 90", "FD 300", "RT 90"]),
        Say("A speed change applies in the middle of a move"),
        Track,
        Run("FD 600", Speed::Animated(50.0)),
        Wait(ms(1500)),
        SetSpeed(Speed::Animated(200.0)),
        Wait(ms(600)),
        CheckSpeeds(ms(1500), 50.0, 200.0),
        SetSpeed(Speed::Instant),
        WaitFinished,
        Say("Step stops at each line, with nothing run ahead"),
        Run(STEPPER, Speed::Animated(100.0)),
        Wait(ms(1200)),
        Pause,
        StepLine,
        WaitPaused,
        Wait(ms(50)),
        CheckNothingAhead,
        StepLine,
        WaitPaused,
        Wait(ms(50)),
        CheckNothingAhead,
        StepLine,
        WaitPaused,
        Wait(ms(50)),
        CheckNothingAhead,
        Screenshot("paused"),
        Stop,
        CheckStopsWithin(ms(50)),
        Say("Stop at Instant"),
        Run(FOREVER, Speed::Instant),
        Wait(ms(500)),
        Stop,
        CheckStopsWithin(ms(50)),
        Say("Stop with the interpreter blocked on a full channel"),
        Run(FOREVER, Speed::Animated(20.0)),
        Wait(ms(500)),
        Stop,
        CheckStopsWithin(ms(50)),
        Say("Stop while paused at a line"),
        Run(FOREVER_LINES, Speed::Animated(100.0)),
        Wait(ms(300)),
        Pause,
        Wait(ms(200)),
        Stop,
        CheckStopsWithin(ms(50)),
        Say("A 131,070-move tree at Instant"),
        Run(TREE, Speed::Instant),
        WaitFinished,
        CheckFinishedWithin(Duration::from_secs(3)),
        CheckFramesAtMost(ms(20)),
        Screenshot("tree"),
        Say("Errors show with their call chain"),
        Run(ERRORS, Speed::Instant),
        WaitFinished,
        CheckConsole("Line 3: FD doesn't like far as input"),
        CheckConsole("    in SQUARE (line 6)"),
        Run("FD 10\nRT", Speed::Instant),
        WaitFinished,
        CheckConsole("Line 2: Not enough inputs to RT"),
        Quit,
    ]
}

pub struct Autotest {
    steps: Vec<Step>,
    next: usize,
    waiting: Option<(Step, Instant)>,
    log: String,
    failures: usize,
    dir: PathBuf,
    screenshots: usize,
    trace: Option<Vec<String>>,
    track: Option<Vec<(Instant, f64)>>,
}

impl Autotest {
    pub fn from_env() -> Option<Autotest> {
        let value = std::env::var("PENDOWN_AUTOTEST").ok()?;
        let dir = if value == "1" { PathBuf::from("target/autotest") } else { PathBuf::from(value) };
        std::fs::create_dir_all(&dir).ok();
        Some(Autotest {
            steps: script(),
            next: 0,
            waiting: None,
            log: String::new(),
            failures: 0,
            dir,
            screenshots: 0,
            trace: None,
            track: None,
        })
    }
}

impl App {
    fn say(&mut self, line: impl Into<String>) {
        let line = line.into();
        println!("{line}");
        if let Some(auto) = &mut self.autotest {
            auto.log.push_str(&line);
            auto.log.push('\n');
        }
    }

    fn check(&mut self, ok: bool, what: String) {
        self.say(format!("  {} {what}", if ok { "ok  " } else { "FAIL" }));
        if !ok {
            self.autotest.as_mut().expect("running the autotest").failures += 1;
        }
    }

    /// Called whenever the highlight moves.
    pub(crate) fn on_highlight(&mut self) {
        let text = self.highlighted();
        if let Some(trace) = self.autotest.as_mut().and_then(|a| a.trace.as_mut()) {
            trace.push(text);
        }
    }

    fn highlighted(&self) -> String {
        self.animator.as_ref().and_then(|a| a.highlight).map_or(String::new(), |s| self.run_text[s.range()].to_string())
    }

    pub(crate) fn drive_autotest(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.autotest.is_none() {
            return;
        }
        if let (Some(track), Some(animator)) = (self.autotest.as_mut().unwrap().track.as_mut(), &self.animator) {
            track.push((Instant::now(), animator.pos.y));
        }
        loop {
            let auto = self.autotest.as_mut().unwrap();
            if let Some((step, since)) = auto.waiting.clone() {
                let done = match step {
                    Step::Wait(d) => since.elapsed() >= d,
                    Step::WaitFinished => self.animator.as_ref().is_none_or(|a| a.finished()) || since.elapsed() > Duration::from_secs(60),
                    Step::WaitPaused => {
                        self.animator.as_ref().is_none_or(|a| a.mode() == Mode::Paused || a.finished())
                            || since.elapsed() > Duration::from_secs(10)
                    }
                    Step::CheckStopsWithin(limit) => {
                        let exited = self.run.as_ref().and_then(|r| r.exited());
                        match (exited, self.stop_at) {
                            (Some(exited), Some(stop)) => {
                                let took = exited.saturating_duration_since(stop);
                                self.check(took <= limit, format!("the interpreter stopped {:.3} ms after Stop", took.as_secs_f64() * 1e3));
                                true
                            }
                            _ if since.elapsed() > Duration::from_secs(2) => {
                                self.check(false, "the interpreter didn't stop within 2 s".into());
                                true
                            }
                            _ => false,
                        }
                    }
                    _ => true,
                };
                if !done {
                    return;
                }
                self.autotest.as_mut().unwrap().waiting = None;
                continue;
            }
            let Some(step) = auto.steps.get(auto.next).cloned() else { return };
            auto.next += 1;
            match step.clone() {
                Step::Say(text) => self.say(format!("\n{text}")),
                Step::Run(program, speed) => {
                    self.set_speed(speed, window, cx);
                    self.editor.update(cx, |state, cx| state.set_value(program.to_string(), window, cx));
                    self.start(DEFAULT_CHANNEL, cx);
                }
                Step::Wait(_) | Step::WaitFinished | Step::WaitPaused | Step::CheckStopsWithin(_) => {
                    self.autotest.as_mut().unwrap().waiting = Some((step, Instant::now()));
                }
                Step::SetSpeed(speed) => self.set_speed(speed, window, cx),
                Step::Pause => self.animator.as_mut().map_or((), |a| a.pause()),
                Step::StepLine => self.animator.as_mut().map_or((), |a| a.step_line()),
                Step::Stop => self.stop(cx),
                Step::Screenshot(name) => self.screenshot(name),
                Step::Track => self.autotest.as_mut().unwrap().track = Some(Vec::new()),
                Step::CheckSpeeds(at, before, after) => {
                    let track = self.autotest.as_mut().unwrap().track.take().unwrap_or_default();
                    self.check_speeds(&track, at, before, after);
                }
                Step::Trace => self.autotest.as_mut().unwrap().trace = Some(Vec::new()),
                Step::CheckTrace(expected) => {
                    let trace = self.autotest.as_mut().unwrap().trace.take().unwrap_or_default();
                    self.check(trace == expected, format!("the highlight went through {trace:?}"));
                }
                Step::CheckHighlight(expected) => {
                    let text = self.highlighted();
                    self.check(text == expected, format!("the highlight is on {text:?}"));
                }
                Step::CheckMoving => {
                    let moving = self.animator.as_ref().is_some_and(|a| a.partial().is_some());
                    self.check(moving, "the turtle is part-way through the move".into());
                }
                Step::CheckNothingAhead => {
                    let (ahead, text) = (self.animator.as_ref().map_or(0, |a| a.lines_ahead()), self.highlighted());
                    self.check(ahead == 0, format!("paused at {text:?} with {ahead} lines run ahead"));
                }
                Step::CheckFinishedWithin(limit) => {
                    let took = match (&self.run, self.finished_at) {
                        (Some(run), Some(at)) => Some(at - run.started),
                        _ => None,
                    };
                    let ok = took.is_some_and(|t| t <= limit);
                    let what = took.map_or("didn't finish".into(), |t| format!("finished in {:.2} s", t.as_secs_f64()));
                    let detail = format!("{what}, {} lines drawn, {} events", self.canvas.borrow().drawing.segments(), self.events);
                    self.check(ok, detail);
                }
                Step::CheckFramesAtMost(limit) => {
                    let p50 = self.frame_intervals.percentile(50).unwrap_or_default();
                    let summary = self.frame_intervals.summary();
                    self.check(p50 <= limit, format!("frame intervals {summary}"));
                }
                Step::CheckConsole(expected) => {
                    let found = self.console.iter().any(|l| l.as_ref() == expected);
                    self.check(found, format!("the console shows {expected:?}"));
                }
                Step::Quit => {
                    let auto = self.autotest.as_ref().unwrap();
                    let failures = auto.failures;
                    let mut log = auto.log.clone();
                    let _ = writeln!(log, "\n{failures} checks failed");
                    let path = auto.dir.join("autotest.log");
                    std::fs::write(&path, log).ok();
                    println!("\n{failures} checks failed; the log is in {}", path.display());
                    std::process::exit(if failures == 0 { 0 } else { 1 });
                }
            }
        }
    }

    fn check_speeds(&mut self, track: &[(Instant, f64)], at: Duration, before: f64, after: f64) {
        let Some(&(start, _)) = track.first() else {
            self.check(false, "no positions recorded".into());
            return;
        };
        // Average speeds over 0.4 s either side of the change, leaving out
        // the frames around it.
        let speed = |from: Duration, to: Duration| {
            let window: Vec<_> = track.iter().filter(|(t, _)| (from..to).contains(&(*t - start))).collect();
            match (window.first(), window.last()) {
                (Some(a), Some(b)) if b.0 > a.0 => (b.1 - a.1) / (b.0 - a.0).as_secs_f64(),
                _ => 0.0,
            }
        };
        let gap = Duration::from_millis(50);
        let measured_before = speed(at - Duration::from_millis(450), at - gap);
        let measured_after = speed(at + gap, at + Duration::from_millis(450));
        let close = |measured: f64, expected: f64| (measured - expected).abs() <= expected * 0.1;
        self.check(
            close(measured_before, before) && close(measured_after, after),
            format!("the turtle moved at {measured_before:.0} steps/s, then {measured_after:.0} steps/s"),
        );
    }

    fn screenshot(&mut self, name: &str) {
        let auto = self.autotest.as_mut().unwrap();
        auto.screenshots += 1;
        let path = auto.dir.join(format!("{}-{name}.png", auto.screenshots));
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("xwd -silent -name Pendown | convert xwd:- '{}'", path.display()))
            .status();
        let ok = status.is_ok_and(|s| s.success());
        self.say(format!("  screenshot {}{}", path.display(), if ok { "" } else { " (failed: needs xwd and ImageMagick)" }));
    }
}
