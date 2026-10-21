//! Spike 04: the interpreter, a channel and the animator, end to end.
//!
//! See `.scratch/phase-1-spikes/issues/04-end-to-end-animator.md`. Run with
//! `cargo run --release`. Set `SPIKE_AUTOTEST=1` to run a scripted session
//! that measures each question, writes `autotest.log` and screenshots, and
//! quits.

mod canvas;

use std::{
    cell::RefCell,
    fmt::Write as _,
    ops::Range,
    rc::Rc,
    time::{Duration, Instant},
};

use gpui_kit::component::{
    ActiveTheme, Disableable as _, Sizable,
    button::{Button, ButtonVariants as _},
    h_flex,
    highlighter::{Diagnostic, DiagnosticSeverity},
    input::{
        Editor, EditorState, FoldRange, HighlightStyleResolver, InputEdit, InputHighlighter, Position,
        RangeDecoration, RangeDecorationCollection, RangeDecorationStyle, Rope,
    },
    resizable::{h_resizable, resizable_panel},
    slider::{Slider, SliderEvent, SliderState},
    v_flex,
};
use gpui_kit::*;
use pendown_logo::RunError;
use spike_04_end_to_end_animator::{
    animator::{Animator, Mode, Speed},
    run::{self, Channel, DEFAULT_CHANNEL, Run},
};

use canvas::PixelCanvas;

const SQUARE: &str = "; Watch the highlight follow the turtle\nREPEAT 4 [FD 300 RT 90]\n";

const FLOWER: &str = r#"; Procedures, and a line per instruction
TO SQUARE :size
  REPEAT 4 [
    FD :size
    RT 90
  ]
END

TO FLOWER :size
  REPEAT 12 [
    SQUARE :size
    RT 30
  ]
END

SETPC 6
FLOWER 250
"#;

const FOREVER: &str = "; Never ends: try Stop\nFOREVER [FD 3 RT 1.5]\n";

const TREE: &str = r#"; 131,070 moves: try Instant
TO TREE :size :depth
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

const STEPPER: &str = r#"; Step through it a line at a time
MAKE "n 0
REPEAT 6 [
  MAKE "n :n + 1
  FD 50 * :n
  RT 60
  PRINT :n
]
PRINT "done
"#;

const EXAMPLES: &[(&str, &str)] =
    &[("Square", SQUARE), ("Flower", FLOWER), ("Stepper", STEPPER), ("Forever", FOREVER), ("Tree", TREE)];

// ---------------------------------------------------------------- speed

/// Speed levels run from 1 to 11. 11 is Instant, and each level below it
/// doubles the speed of the one before, from 5 steps a second at level 1 to
/// 2,560 at level 10.
const INSTANT_LEVEL: f32 = 11.0;

fn level_speed(level: f32) -> Speed {
    let level = level.round().clamp(1.0, INSTANT_LEVEL);
    if level >= INSTANT_LEVEL { Speed::Instant } else { Speed::Animated(5.0 * 2f64.powf(level as f64 - 1.0)) }
}

fn speed_level(speed: Speed) -> f32 {
    match speed {
        Speed::Instant => INSTANT_LEVEL,
        Speed::Animated(steps) => ((steps / 5.0).log2() + 1.0).round().clamp(1.0, INSTANT_LEVEL as f64 - 1.0) as f32,
    }
}

fn describe(speed: Speed) -> String {
    match speed {
        Speed::Instant => "Instant".into(),
        Speed::Animated(steps) => format!("level {} · {steps:.0} steps/s", speed_level(speed)),
    }
}

// ---------------------------------------------------------------- syntax colours (from spike 01)

fn lex(text: &str) -> Vec<(Range<usize>, &'static str)> {
    const COMMANDS: &[&str] = &[
        "fd", "forward", "bk", "back", "rt", "right", "lt", "left", "repeat", "to", "end", "make", "if", "ifelse",
        "stop", "output", "op", "print", "pr", "pu", "pd", "penup", "pendown", "setpc", "setpencolor", "forever",
        "show", "type", "repcount",
    ];
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        let c = bytes[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let name = match c {
            b';' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                "comment"
            }
            b'[' | b']' | b'(' | b')' => {
                i += 1;
                "punctuation.bracket"
            }
            b'+' | b'-' | b'*' | b'/' | b'=' | b'<' | b'>' => {
                i += 1;
                "operator"
            }
            _ => {
                while i < bytes.len() && !b" \t\r\n[]();".contains(&bytes[i]) {
                    i += 1;
                }
                let word = &text[start..i];
                if word.starts_with(':') {
                    "variable"
                } else if word.starts_with('"') {
                    "string"
                } else if word.parse::<f64>().is_ok() {
                    "number"
                } else if COMMANDS.contains(&word.to_lowercase().as_str()) {
                    "keyword"
                } else {
                    "function"
                }
            }
        };
        out.push((start..i, name));
    }
    out
}

#[derive(Default)]
struct LogoHighlighter {
    runs: Vec<(Range<usize>, &'static str)>,
}

impl InputHighlighter for LogoHighlighter {
    fn language(&self) -> SharedString {
        "logo".into()
    }

    fn update(&mut self, _: Option<InputEdit>, text: &Rope, _: bool, _: &mut Window, _: &mut Context<EditorState>) {
        self.runs = lex(&text.to_string());
    }

    fn styles(&self, range: &Range<usize>, resolver: &dyn HighlightStyleResolver) -> Vec<(Range<usize>, HighlightStyle)> {
        let mut out = Vec::new();
        let mut at = range.start;
        let first = self.runs.partition_point(|(r, _)| r.end <= range.start);
        for (r, name) in &self.runs[first..] {
            if r.start >= range.end {
                break;
            }
            let (start, end) = (r.start.max(range.start), r.end.min(range.end));
            if start > at {
                out.push((at..start, HighlightStyle::default()));
            }
            out.push((start..end, resolver.style(name).unwrap_or_default()));
            at = end;
        }
        if at < range.end {
            out.push((at..range.end, HighlightStyle::default()));
        }
        out
    }

    fn fold_ranges(&self, _: &Rope) -> Vec<FoldRange> {
        Vec::new()
    }
}

// ---------------------------------------------------------------- measurements

#[derive(Default, Clone)]
struct Samples(Vec<Duration>);

impl Samples {
    fn push(&mut self, d: Duration) {
        self.0.push(d);
    }

    fn summary(&self) -> String {
        if self.0.is_empty() {
            return "no samples".into();
        }
        let mut v = self.0.clone();
        v.sort();
        let ms = |d: Duration| d.as_secs_f64() * 1000.0;
        format!(
            "n={} p50={:.1}ms p95={:.1}ms max={:.1}ms",
            v.len(),
            ms(v[v.len() / 2]),
            ms(v[v.len() * 95 / 100]),
            ms(v[v.len() - 1])
        )
    }
}

// ---------------------------------------------------------------- the scripted session

/// One step of the scripted session.
#[derive(Clone)]
#[allow(clippy::enum_variant_names)]
enum Step {
    Say(&'static str),
    Run { program: &'static str, speed: Speed, channel: Channel },
    Wait(Duration),
    WaitFinished(Duration),
    WaitPaused,
    SetSpeed(Speed),
    Pause,
    Resume,
    StepLine,
    StepInstruction,
    Stop,
    /// Log how long the interpreter thread takes to exit after Stop.
    WaitExited,
    /// Log where the highlight is, and how far ahead the interpreter is.
    Where,
    /// Start or stop logging every highlight change, with the turtle.
    Trace(bool),
    /// Start or stop logging the turtle's position every frame.
    Track(bool),
    Report,
    Screenshot(&'static str),
    /// Resize the window, which re-renders the drawing (ADR 0003).
    Resize(f32, f32),
    /// Start or stop logging each frame's timings.
    Frames(bool),
    Quit,
}

fn script() -> Vec<Step> {
    use Step::*;
    let ms = Duration::from_millis;
    let s = Duration::from_secs;
    if std::env::var("SPIKE_SCRIPT").as_deref() == Ok("tree") {
        return vec![
            Say("## TREE 230 16 at Instant, frame by frame"),
            Frames(true),
            Run { program: TREE, speed: Speed::Instant, channel: DEFAULT_CHANNEL },
            WaitFinished(s(60)),
            Frames(false),
            Report,
            Quit,
        ];
    }
    let instant = Speed::Instant;
    let mut steps = vec![
        Say("## 4. Highlighting: REPEAT 4 [FD 300 RT 90] at 100 steps/s"),
        Trace(true),
        Run { program: SQUARE, speed: Speed::Animated(100.0), channel: DEFAULT_CHANNEL },
        Wait(ms(1500)),
        Screenshot("highlight-mid-move"),
        WaitFinished(s(30)),
        Trace(false),
        Report,
        Say("## 5. Speed change in the middle of FD 600: 50, then 200 steps/s"),
        Track(true),
        Run { program: "FD 600", speed: Speed::Animated(50.0), channel: DEFAULT_CHANNEL },
        Wait(s(2)),
        SetSpeed(Speed::Animated(200.0)),
        Wait(ms(500)),
        Track(false),
        SetSpeed(instant),
        WaitFinished(s(5)),
        Say("## 3. Stepping: the Stepper example at 100 steps/s"),
        Run { program: STEPPER, speed: Speed::Animated(100.0), channel: DEFAULT_CHANNEL },
        Wait(ms(1200)),
        Pause,
        Wait(ms(200)),
        Where,
        StepLine,
        WaitPaused,
        Wait(ms(100)),
        Where,
        StepLine,
        WaitPaused,
        Wait(ms(100)),
        Where,
        StepLine,
        WaitPaused,
        Wait(ms(100)),
        Where,
        Screenshot("paused-after-step"),
        StepInstruction,
        WaitPaused,
        Where,
        StepInstruction,
        WaitPaused,
        Where,
        Resume,
        SetSpeed(instant),
        WaitFinished(s(10)),
        Say("### Pausing from Instant"),
        Run { program: TREE, speed: instant, channel: DEFAULT_CHANNEL },
        Wait(ms(300)),
        Pause,
        Wait(ms(200)),
        Where,
        StepLine,
        WaitPaused,
        Wait(ms(100)),
        Where,
        Stop,
        WaitExited,
        Say("## 1. Stop"),
        Say("### FOREVER at Instant"),
        Run { program: FOREVER, speed: instant, channel: DEFAULT_CHANNEL },
        Wait(s(1)),
        Stop,
        WaitExited,
        Say("### FOREVER at 20 steps/s: one line, so the interpreter is blocked on the full channel"),
        Run { program: FOREVER, speed: Speed::Animated(20.0), channel: DEFAULT_CHANNEL },
        Wait(s(1)),
        Where,
        Stop,
        WaitExited,
        Say("### Paused, with the interpreter waiting at a line"),
        Run { program: STEPPER, speed: Speed::Animated(100.0), channel: DEFAULT_CHANNEL },
        Wait(ms(500)),
        Pause,
        StepLine,
        WaitPaused,
        Wait(ms(200)),
        Where,
        Stop,
        WaitExited,
        Say("## 2. Huge program: TREE 230 16 (131,070 moves) at Instant"),
        Frames(true),
        Run { program: TREE, speed: instant, channel: DEFAULT_CHANNEL },
        WaitFinished(s(120)),
        Frames(false),
        Report,
        Screenshot("tree-instant"),
        Say("## 6. Channel sizes: TREE 230 16 at Instant"),
    ];
    for channel in [
        Channel { batch: 1, capacity: 1024, max_wait: ms(4) },
        Channel { batch: 64, capacity: 4, max_wait: ms(4) },
        Channel { batch: 256, capacity: 4, max_wait: ms(4) },
        Channel { batch: 1024, capacity: 4, max_wait: ms(4) },
        Channel { batch: 256, capacity: 16, max_wait: ms(4) },
        Channel { batch: 256, capacity: 32, max_wait: ms(4) },
        Channel { batch: 1024, capacity: 16, max_wait: ms(4) },
    ] {
        steps.push(Say(Box::leak(format!("### batches of {}, channel of {}", channel.batch, channel.capacity).into_boxed_str())));
        steps.push(Run { program: TREE, speed: instant, channel });
        steps.push(WaitFinished(s(120)));
        steps.push(Report);
    }
    steps.extend([
        Say("## Lockstep cost: 20,000 lines of FD 1 / RT 1, Instant against 1,000,000 steps/s (in lockstep)"),
        Run { program: "REPEAT 10000 [\n  FD 1\n  RT 1\n]", speed: instant, channel: DEFAULT_CHANNEL },
        WaitFinished(s(60)),
        Report,
        Run { program: "REPEAT 10000 [\n  FD 1\n  RT 1\n]", speed: Speed::Animated(1_000_000.0), channel: DEFAULT_CHANNEL },
        WaitFinished(s(60)),
        Report,
        Say("## Resize: re-rendering the tree"),
        Run { program: TREE, speed: instant, channel: DEFAULT_CHANNEL },
        WaitFinished(s(60)),
        Report,
        Resize(1100.0, 750.0),
        Wait(s(1)),
        Report,
        Resize(1400.0, 900.0),
        Wait(s(1)),
        Report,
        Quit,
    ]);
    steps
}

struct Autotest {
    steps: Vec<Step>,
    next: usize,
    waiting: Option<(Step, Instant)>,
    log: String,
    trace: bool,
    track: Vec<(Instant, f64)>,
    tracking: bool,
    frames: bool,
    screenshots: usize,
}

// ---------------------------------------------------------------- the app

struct App {
    editor: Entity<EditorState>,
    highlight: RangeDecorationCollection,
    levels: Entity<SliderState>,
    speed: Speed,
    canvas: Rc<RefCell<PixelCanvas>>,
    animator: Option<Animator>,
    run: Option<Run>,
    /// The text that was run, which spans point into (ADR 0006).
    run_text: String,
    finished_at: Option<Instant>,
    stop_at: Option<Instant>,
    console: Vec<SharedString>,
    last_frame: Option<Instant>,
    frame_intervals: Samples,
    frame_work: Samples,
    events: usize,
    last_render: Option<Instant>,
    auto: Option<Autotest>,
    _subscriptions: Vec<Subscription>,
}

impl App {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> App {
        let editor = cx.new(|cx| {
            EditorState::new(window, cx).language("logo").line_number(true).soft_wrap(false).default_value(FLOWER)
        });
        let highlight = editor.update(cx, |state, cx| {
            state.set_highlighter_factory(
                Rc::new(|language| (language == "logo").then(|| Box::new(LogoHighlighter::default()) as Box<dyn InputHighlighter>)),
                cx,
            );
            state.create_range_decorations_collection(vec![], cx)
        });
        let speed = Speed::Animated(80.0);
        let levels = cx.new(|_| SliderState::new().min(1.0).max(INSTANT_LEVEL).step(1.0).default_value(speed_level(speed)));
        let subscriptions = vec![
            cx.subscribe_in(&levels, window, |this, _, event: &SliderEvent, window, cx| {
                if let SliderEvent::Change(value) = event {
                    this.set_speed(level_speed(value.end()), window, cx);
                }
            }),
        ];
        let auto = std::env::var_os("SPIKE_AUTOTEST").map(|_| Autotest {
            steps: script(),
            next: 0,
            waiting: None,
            log: String::new(),
            trace: false,
            track: Vec::new(),
            tracking: false,
            frames: false,
            screenshots: 0,
        });
        App {
            editor,
            highlight,
            levels,
            speed,
            canvas: Rc::new(RefCell::new(PixelCanvas::new())),
            animator: None,
            run: None,
            run_text: String::new(),
            finished_at: None,
            stop_at: None,
            console: Vec::new(),
            last_frame: None,
            frame_intervals: Samples::default(),
            frame_work: Samples::default(),
            events: 0,
            last_render: None,
            auto,
            _subscriptions: subscriptions,
        }
    }

    fn say(&mut self, line: impl Into<String>) {
        let line = line.into();
        println!("{line}");
        if let Some(auto) = &mut self.auto {
            auto.log.push_str(&line);
            auto.log.push('\n');
        }
    }

    /// Sets the speed, and moves the slider to match. The scripted session
    /// also sets speeds between the levels.
    fn set_speed(&mut self, speed: Speed, window: &mut Window, cx: &mut Context<Self>) {
        if speed == self.speed {
            return;
        }
        self.speed = speed;
        if let Some(animator) = &mut self.animator {
            animator.set_speed(speed);
        }
        let level = speed_level(speed);
        if level_speed(self.levels.read(cx).value().end()) != level_speed(level) {
            self.levels.update(cx, |s, cx| s.set_value(level, window, cx));
        }
        cx.notify();
    }

    fn load(&mut self, program: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.stop(cx);
        self.editor.update(cx, |state, cx| state.set_value(program.to_string(), window, cx));
    }

    fn running(&self) -> bool {
        self.animator.as_ref().is_some_and(|a| !a.finished())
    }

    fn start(&mut self, channel: Channel, cx: &mut Context<Self>) {
        self.stop(cx);
        let text = self.editor.read(cx).value().to_string();
        self.run_text = text.clone();
        self.canvas.borrow_mut().clean();
        self.console.clear();
        self.set_diagnostics(Vec::new(), cx);
        let (mut animator, run) = run::start(text, self.speed, channel);
        if let Some(ms) = std::env::var("SPIKE_SLICE_MS").ok().and_then(|v| v.parse().ok()) {
            animator.instant_slice = Duration::from_millis(ms);
        }
        self.animator = Some(animator);
        self.run = Some(run);
        self.finished_at = None;
        self.stop_at = None;
        self.last_frame = None;
        self.frame_intervals = Samples::default();
        self.frame_work = Samples::default();
        self.events = 0;
        cx.notify();
    }

    fn stop(&mut self, cx: &mut Context<Self>) {
        if let Some(animator) = &mut self.animator
            && !animator.finished()
        {
            animator.stop();
            self.stop_at = Some(Instant::now());
            self.console.push("Stopped.".into());
        }
        cx.notify();
    }

    fn set_diagnostics(&mut self, diagnostics: Vec<Diagnostic>, cx: &mut Context<Self>) {
        self.editor.update(cx, |state, cx| {
            let text = state.text().clone();
            if let Some(set) = state.diagnostics_mut() {
                set.reset(&text);
                set.extend(diagnostics);
            }
            cx.notify();
        });
    }

    fn position(&self, offset: usize) -> Position {
        let before = &self.run_text[..offset.min(self.run_text.len())];
        let line = before.matches('\n').count() as u32;
        let column = (before.len() - before.rfind('\n').map_or(0, |i| i + 1)) as u32;
        Position::new(line, column)
    }

    /// Shows an error as a diagnostic on the code and in the console (ADR 0005).
    fn show_errors(&mut self, errors: Vec<pendown_logo::Error>, cx: &mut Context<Self>) {
        let mut diagnostics = Vec::new();
        for e in &errors {
            self.console.push(format!("{e}").into());
            if let Some(span) = e.span {
                let range = self.position(span.start as usize)..self.position(span.end as usize);
                diagnostics.push(Diagnostic::new(range, e.message.clone()).with_severity(DiagnosticSeverity::Error));
            }
        }
        self.set_diagnostics(diagnostics, cx);
    }

    /// Moves the highlight to the running instruction, scrolling to keep it
    /// in view (spike 01).
    fn show_highlight(&mut self, cx: &mut Context<Self>) {
        let Some(span) = self.animator.as_ref().and_then(|a| a.highlight) else {
            self.highlight.set(vec![], cx);
            return;
        };
        let color = cx.theme().warning.opacity(0.35);
        self.highlight.set(vec![RangeDecoration::new(span.range()).with_style(RangeDecorationStyle::Fill).with_color(color)], cx);
        let line = span.line as usize;
        self.editor.update(cx, |state, cx| {
            let (Some(rows), Some(line_height)) = (state.visible_row_range(), state.line_height()) else { return };
            if line < rows.start + 2 || line + 2 >= rows.end {
                let offset = state.scroll_offset();
                state.set_scroll_offset(point(offset.x, -line_height * line.saturating_sub(3) as f32), cx);
            }
        });
    }

    /// One animation frame: play events, draw what finished, and check
    /// whether the run has ended.
    fn tick(&mut self, cx: &mut Context<Self>) {
        let now = Instant::now();
        let Some(animator) = &mut self.animator else { return };
        if !animator.wants_frames() {
            self.last_frame = None;
        } else {
            let dt = self.last_frame.map_or(Duration::from_millis(16), |t| now - t);
            if self.last_frame.is_some() {
                self.frame_intervals.push(dt);
            }
            self.last_frame = Some(now);
            // Instant mode draws at most about 6 ms of segments a frame.
            let per_segment = self.canvas.borrow().per_segment.max(Duration::from_nanos(200));
            let max_segments = (Duration::from_millis(6).as_nanos() / per_segment.as_nanos()).clamp(500, 100_000) as usize;
            let frame = animator.frame(dt, max_segments);
            self.events += frame.events;
            self.canvas.borrow_mut().push(&frame.segments);
            for text in frame.printed {
                for line in text.lines() {
                    self.console.push(line.to_string().into());
                }
            }
            if frame.highlight_changed {
                if let Some(auto) = &mut self.auto
                    && auto.trace
                {
                    let a = self.animator.as_ref().unwrap();
                    let text = a.highlight.map_or("", |s| &self.run_text[s.range()]).replace('\n', " ");
                    let t = self.run.as_ref().unwrap().started.elapsed().as_secs_f64();
                    let line = format!("- {t:6.3} s: `{text}` with the turtle at ({:.0}, {:.0}), heading {:.0}", a.pos.x, a.pos.y, a.heading);
                    self.say(line);
                }
                self.show_highlight(cx);
            }
            if let Some(auto) = &mut self.auto
                && auto.tracking
            {
                let y = self.animator.as_ref().unwrap().pos.y;
                auto.track.push((now, y));
            }
            self.frame_work.push(now.elapsed());
        }
        let animator = self.animator.as_ref().unwrap();
        if animator.finished() && self.finished_at.is_none() && !animator.stopped {
            self.finished_at = Some(now);
            if let Some(Err(e)) = self.run.as_ref().and_then(|r| r.result()) {
                match e {
                    RunError::Parse(errors) => self.show_errors(errors, cx),
                    RunError::Runtime(e) => self.show_errors(vec![e], cx),
                    RunError::Halted => {}
                }
            }
        }
    }

    fn status(&self) -> String {
        let mut s = describe(self.speed);
        if let Some(animator) = &self.animator {
            let state = if animator.stopped {
                "stopped"
            } else if animator.finished() {
                "finished"
            } else {
                match animator.mode() {
                    Mode::Paused => "paused",
                    Mode::Running => "running",
                    _ => "stepping",
                }
            };
            let run = self.run.as_ref().unwrap();
            let elapsed = self.finished_at.unwrap_or_else(Instant::now) - run.started;
            let _ = write!(
                s,
                " · {state} · {:.1} s · {} segments · {} lines ahead",
                elapsed.as_secs_f64(),
                self.canvas.borrow().segments(),
                animator.lines_ahead()
            );
        }
        s
    }

    // ------------------------------------------------------------ the scripted session

    fn report(&mut self) {
        let run = self.run.as_ref();
        let elapsed = match (run, self.finished_at) {
            (Some(run), Some(at)) => format!("{:.2} s", (at - run.started).as_secs_f64()),
            _ => "not finished".into(),
        };
        let canvas = self.canvas.borrow();
        let line = format!(
            "- finished in {elapsed}; {} segments, {} events; frame intervals {}; work per frame {}; last draw {:.1} ms, upload {:.1} ms, {:.2} µs a segment",
            canvas.segments(),
            self.events,
            self.frame_intervals.summary(),
            self.frame_work.summary(),
            canvas.last_draw.as_secs_f64() * 1e3,
            canvas.last_upload.as_secs_f64() * 1e3,
            canvas.per_segment.as_secs_f64() * 1e6,
        );
        let rerender = canvas.rerender.map(|(n, d)| format!("- last re-render: {n} segments in {:.0} ms", d.as_secs_f64() * 1e3));
        drop(canvas);
        self.say(line);
        if let Some(rerender) = rerender {
            self.say(rerender);
        }
    }

    fn where_is(&mut self) {
        let Some(animator) = &self.animator else { return };
        let text = animator.highlight.map_or("", |s| &self.run_text[s.range()]).replace('\n', " ");
        let line = format!(
            "- {:?}: highlight on `{text}`; interpreter has started {} lines, the highlight has reached {} ({} ahead)",
            animator.mode(),
            animator.lines_started(),
            animator.lines_shown,
            animator.lines_ahead()
        );
        self.say(line);
    }

    fn screenshot(&mut self, name: &str) {
        let Some(auto) = &mut self.auto else { return };
        auto.screenshots += 1;
        let path = format!("screenshot-{}-{name}.png", auto.screenshots);
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("xwd -silent -name 'Pendown spike 04' | convert xwd:- '{path}'"))
            .status();
        self.say(format!("- screenshot `{path}`: {:?}", status.map(|s| s.success())));
    }

    fn drive(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        loop {
            let Some(auto) = &mut self.auto else { return };
            if let Some((step, since)) = auto.waiting.clone() {
                let done = match step {
                    Step::Wait(d) => since.elapsed() >= d,
                    Step::WaitFinished(timeout) => {
                        let finished = self.animator.as_ref().is_none_or(|a| a.finished());
                        if !finished && since.elapsed() >= timeout {
                            self.say("- timed out");
                        }
                        finished || since.elapsed() >= timeout
                    }
                    Step::WaitPaused => {
                        self.animator.as_ref().is_none_or(|a| a.mode() == Mode::Paused || a.finished())
                            || since.elapsed() >= Duration::from_secs(10)
                    }
                    Step::WaitExited => {
                        let exited = self.run.as_ref().and_then(|r| r.exited());
                        if let (Some(exited), Some(stop)) = (exited, self.stop_at) {
                            let took = exited.saturating_duration_since(stop);
                            let result = self.run.as_ref().unwrap().result();
                            self.say(format!("- the interpreter thread exited {:.3} ms after Stop: {result:?}", took.as_secs_f64() * 1e3));
                            true
                        } else if since.elapsed() >= Duration::from_secs(5) {
                            self.say("- the interpreter thread didn't exit within 5 s");
                            true
                        } else {
                            false
                        }
                    }
                    _ => true,
                };
                if !done {
                    return;
                }
                self.auto.as_mut().unwrap().waiting = None;
                continue;
            }
            let Some(step) = auto.steps.get(auto.next).cloned() else { return };
            auto.next += 1;
            match step.clone() {
                Step::Say(text) => self.say(format!("\n{text}\n")),
                Step::Run { program, speed, channel } => {
                    self.set_speed(speed, window, cx);
                    self.editor.update(cx, |state, cx| state.set_value(program.to_string(), window, cx));
                    self.start(channel, cx);
                }
                Step::Wait(_) | Step::WaitFinished(_) | Step::WaitPaused | Step::WaitExited => {
                    self.auto.as_mut().unwrap().waiting = Some((step, Instant::now()));
                }
                Step::SetSpeed(speed) => {
                    self.set_speed(speed, window, cx);
                    let line = format!("- speed set to {}", describe(speed));
                    self.say(line);
                }
                Step::Pause => self.animator.as_mut().map_or((), |a| a.pause()),
                Step::Resume => self.animator.as_mut().map_or((), |a| a.resume()),
                Step::StepLine => self.animator.as_mut().map_or((), |a| a.step_line()),
                Step::StepInstruction => self.animator.as_mut().map_or((), |a| a.step_instruction()),
                Step::Stop => self.stop(cx),
                Step::Where => self.where_is(),
                Step::Trace(on) => self.auto.as_mut().unwrap().trace = on,
                Step::Track(on) => {
                    let auto = self.auto.as_mut().unwrap();
                    auto.tracking = on;
                    if !on {
                        let track = std::mem::take(&mut auto.track);
                        self.report_track(&track);
                    }
                }
                Step::Report => self.report(),
                Step::Screenshot(name) => self.screenshot(name),
                Step::Frames(on) => self.auto.as_mut().unwrap().frames = on,
                Step::Resize(w, h) => {
                    window.resize(size(px(w), px(h)));
                    self.say(format!("- resized the window to {w} × {h}"));
                }
                Step::Quit => {
                    let log = self.auto.as_ref().unwrap().log.clone();
                    std::fs::write("autotest.log", log).ok();
                    cx.quit();
                    return;
                }
            }
        }
    }

    /// The turtle's speed, frame by frame, around the speed change.
    fn report_track(&mut self, track: &[(Instant, f64)]) {
        let Some(&(t0, _)) = track.first() else { return };
        let mut lines = Vec::new();
        for pair in track.windows(2) {
            let ((ta, ya), (tb, yb)) = (pair[0], pair[1]);
            let dt = (tb - ta).as_secs_f64();
            if dt > 0.0 {
                lines.push((((ta - t0).as_secs_f64()), (yb - ya) / dt));
            }
        }
        // Average speed over each 250 ms, and the frames either side of the change.
        let mut buckets: Vec<(f64, f64, usize)> = Vec::new();
        for &(t, v) in &lines {
            let b = (t / 0.25) as usize;
            if buckets.len() <= b {
                buckets.resize(b + 1, (0.0, 0.0, 0));
            }
            buckets[b].0 = b as f64 * 0.25;
            buckets[b].1 += v;
            buckets[b].2 += 1;
        }
        for (start, sum, n) in buckets {
            if n > 0 {
                self.say(format!("- {start:.2} s: {:.0} steps/s over {n} frames", sum / n as f64));
            }
        }
        let change = lines.iter().position(|w| w.1 > 120.0);
        if let Some(i) = change {
            let around: Vec<_> = lines[i.saturating_sub(2)..(i + 3).min(lines.len())].iter().map(|(t, v)| format!("{t:.3} s: {v:.0}")).collect();
            self.say(format!("- frames around the change (steps/s): {}", around.join(", ")));
        }
    }
}

impl Render for App {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let render_started = Instant::now();
        if let Some(auto) = &self.auto
            && auto.frames
            && self.running()
        {
            let canvas = self.canvas.borrow();
            let line = format!(
                "- frame at {:7.1} ms: previous paint {:.1} ms (draw {:.1}, upload {:.1}), since last render start {:.1} ms",
                self.run.as_ref().unwrap().started.elapsed().as_secs_f64() * 1e3,
                canvas.last_paint.as_secs_f64() * 1e3,
                canvas.last_draw.as_secs_f64() * 1e3,
                canvas.last_upload.as_secs_f64() * 1e3,
                self.last_render.map_or(0.0, |t| (render_started - t).as_secs_f64() * 1e3),
            );
            drop(canvas);
            self.say(line);
        }
        self.last_render = Some(render_started);
        self.tick(cx);
        self.drive(window, cx);
        if self.animator.as_ref().is_some_and(|a| a.wants_frames()) || self.auto.is_some() {
            window.request_animation_frame();
        }

        let theme = cx.theme().clone();
        let running = self.running();
        let paused = self.animator.as_ref().is_some_and(|a| a.mode() == Mode::Paused && !a.finished());
        let canvas_state = self.canvas.clone();
        let (partial, turtle) = match &self.animator {
            Some(a) => (a.partial(), Some((a.pos, a.heading))),
            None => (None, Some((pendown_logo::Point { x: 0.0, y: 0.0 }, 0.0))),
        };
        let status = self.status();

        let mut examples = h_flex().gap_1();
        for (i, (name, program)) in EXAMPLES.iter().enumerate() {
            examples = examples.child(
                Button::new(("example", i))
                    .small()
                    .ghost()
                    .label(*name)
                    .on_click(cx.listener(move |this, _, window, cx| this.load(program, window, cx))),
            );
        }

        v_flex()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(
                h_flex()
                    .gap_2()
                    .p_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        Button::new("run")
                            .small()
                            .primary()
                            .label("Run")
                            .on_click(cx.listener(|this, _, _, cx| this.start(DEFAULT_CHANNEL, cx))),
                    )
                    .child(
                        Button::new("pause")
                            .small()
                            .label(if paused { "Resume" } else { "Pause" })
                            .disabled(!running)
                            .on_click(cx.listener(|this, _, _, cx| {
                                if let Some(a) = &mut this.animator {
                                    if a.mode() == Mode::Paused { a.resume() } else { a.pause() }
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("step")
                            .small()
                            .label("Step")
                            .disabled(!paused)
                            .on_click(cx.listener(|this, _, _, cx| {
                                if let Some(a) = &mut this.animator {
                                    a.step_line();
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("step-instruction")
                            .small()
                            .label("Step instruction")
                            .disabled(!paused)
                            .on_click(cx.listener(|this, _, _, cx| {
                                if let Some(a) = &mut this.animator {
                                    a.step_instruction();
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("stop")
                            .small()
                            .danger()
                            .label("Stop")
                            .disabled(!running)
                            .on_click(cx.listener(|this, _, _, cx| this.stop(cx))),
                    )
                    .child(div().w(px(12.)))
                    .child(div().text_sm().child("Speed"))
                    .child(div().w(px(140.)).child(Slider::new(&self.levels)))
                    .child(div().w(px(8.)))
                    .child(div().text_sm().text_color(theme.muted_foreground).child(status)),
            )
            .child(h_flex().px_2().py_1().border_b_1().border_color(theme.border).child(examples))
            .child(
                div().flex_1().min_h_0().child(
                    h_resizable("main")
                        .child(
                            resizable_panel().size(px(460.)).child(
                                Editor::new(&self.editor)
                                    .bordered(false)
                                    .readonly(running)
                                    .h_full()
                                    .font_family(theme.mono_font_family.clone())
                                    .text_size(theme.mono_font_size),
                            ),
                        )
                        .child(resizable_panel().child(
                            canvas(
                                move |bounds, _, _| bounds,
                                move |_, bounds, window, _| {
                                    canvas_state.borrow_mut().paint(bounds, window, partial, turtle);
                                },
                            )
                            .size_full(),
                        )),
                ),
            )
            .child(
                div()
                    .id("console")
                    .h(px(110.))
                    .p_2()
                    .border_t_1()
                    .border_color(theme.border)
                    .overflow_y_scroll()
                    .font_family(theme.mono_font_family.clone())
                    .text_xs()
                    .children(self.console.iter().skip(self.console.len().saturating_sub(40)).cloned().map(|l| div().child(l))),
            )
    }
}

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions { title: Some("Pendown spike 04".into()), ..Default::default() }),
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, size(px(1400.), px(900.)), cx))),
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, |window, cx| cx.new(|cx| App::new(window, cx))).expect("failed to open window");
        cx.activate(true);
    });
}
