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
    metrics::Snapshot,
    run::DEFAULT_CHANNEL,
    speed::level_speed,
};
use pendown_logo::SourceId;

use crate::app::App;

const BOX: &str = "REPEAT 4 [FD 300 RT 90]";

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

/// At speed 3 it moves about 14 pixels a second, so it needs about that many
/// frames a second, and prints now and then.
const SLOW: &str = "REPEAT 8 [\n  FD 20\n  RT 45\n  PRINT REPCOUNT\n]";

/// Phase 4: a loop in a procedure, called from a template.
const WATCHED: &str = "TO TILE :size
  REPEAT 4 [FD :size RT 90]
END
MAKE \"sizes [40 80]
FOREACH :sizes [TILE ?]
";

const COUNTING: &str = "MAKE \"n 0\nFOREVER [MAKE \"n :n + 1]";

const ERRORS: &str = "TO BOX :n\n  FD :n RT 90\n  FD \"far\n  PRINT 1\nEND\nBOX 100";

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
    Resume,
    StepLine,
    StepInstruction,
    Stop,
    Screenshot(&'static str),
    /// Start recording the turtle's y position at every frame.
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
    /// Type a line into the console and press Enter.
    Console(&'static str),
    /// How many lines, arcs and shapes are drawn.
    CheckDrawn(usize),
    /// Where the turtle sprite is, and its heading.
    CheckTurtle(f64, f64, f64),
    /// Save the program as this file in the autotest folder, as Save As
    /// would once the dialog has a name.
    SaveTo(&'static str),
    OpenFrom(&'static str),
    ExportTo(&'static str),
    /// Replace the editor's text, as typing would.
    Type(&'static str),
    CheckTitle(&'static str),
    /// The editor holds exactly this.
    CheckEditor(&'static str),
    /// The exported picture is a PNG the size of the canvas.
    CheckExported(&'static str),
    /// The exported picture is an SVG with an element for every line and
    /// shape drawn.
    CheckExportedSvg(&'static str),
    /// The editor shows exactly these diagnostics.
    CheckDiagnostics(&'static [&'static str]),
    /// Start counting renders and highlight changes.
    MeasureRenders,
    /// Since `MeasureRenders`: the window rendered at most this many times a
    /// second, and at most once an animation frame; the editor only when the
    /// highlight moved, and the console only when something was printed.
    CheckRenders(f64),
    /// The counters and variables panels show exactly this, in the form
    /// `panels()` gives.
    CheckPanels(&'static str),
    /// They show something starting with this.
    CheckPanelsStart(&'static str),
    /// Both panels have drawn what they should show.
    CheckPanelsDrawn,
    /// The variables panel shows these colours, as `PENCOLOR=red ...`.
    CheckColours(&'static str),
    /// Press F12.
    ToggleMetrics,
    /// Phase 5: put the editor's cursor one character into the first
    /// `marker`.
    CursorAt(&'static str),
    /// The same, then press F1.
    F1At(&'static str),
    /// The help panel shows the page with this title.
    CheckHelpPage(&'static str),
    /// Type into the help panel's search box.
    SearchHelp(&'static str),
    /// The first search result is the command with this title.
    CheckFirstResult(&'static str),
    /// Open a help page by name, as a link does.
    OpenHelp(&'static str),
    /// The Contents button.
    HelpContents,
    /// Close the side panel, as its button does.
    CloseHelp,
    /// Press Run on an example: the command's name, and which example,
    /// counting from 0.
    RunExample(&'static str, usize),
    /// The example's code is highlighted here, in the panel.
    CheckExampleHighlight(&'static str),
    BackToMyDrawing,
    /// Press Copy to editor on an example.
    CopyExample(&'static str, usize),
    /// The editor's text ends with this.
    CheckEditorEnds(&'static str),
    /// The console has a Help button for this page, and pressing it opens
    /// the panel there.
    CheckErrorHelp(&'static str),
    /// Open the Tutorial tab at a lesson, counting from 0.
    Lesson(usize),
    /// Load into editor on a lesson's program, without the question about
    /// unsaved changes.
    LoadLesson(usize, usize),
    /// Run on a lesson's program.
    RunLesson(usize, usize),
    /// The metrics overlay is shown, with numbers in it, or isn't.
    CheckOverlay(bool),
    Quit,
}

const CATCHING: &str = "CATCH \"error [FD \"x]
SHOW ERROR
TO CHECK :n
IF :n < 0 [(THROW \"error [CHECK needs a positive number])]
END
CHECK -1";

fn script() -> Vec<Step> {
    use Step::*;
    let ms = Duration::from_millis;
    vec![
        Say("A console entry before any run"),
        Console("FD 1"),
        WaitFinished,
        CheckDrawn(1),
        Say("A console turn turns where the last entry left the turtle"),
        Console("CS"),
        WaitFinished,
        Console("FD 100"),
        WaitFinished,
        Console("RT 45"),
        WaitFinished,
        CheckTurtle(0.0, 100.0, 45.0),
        Say("The welcome program"),
        Run(crate::app::WELCOME, level_speed(10)),
        WaitFinished,
        CheckFinishedWithin(Duration::from_secs(30)),
        Say("The console uses what the last run left"),
        Console("BOX 50"),
        WaitFinished,
        CheckDrawn(52),
        CheckConsole("> BOX 50"),
        Console("FOO"),
        WaitFinished,
        CheckConsole("In the console: I don't know how to FOO"),
        Console("TO TRI :n"),
        Console("REPEAT 3 [FD :n RT 120]"),
        Console("END"),
        WaitFinished,
        Console("TRI 100"),
        WaitFinished,
        CheckDrawn(55),
        Screenshot("console"),
        Run(crate::app::WELCOME, Speed::Instant),
        WaitFinished,
        CheckConsole("Run cleared the procedures you defined in the console. \"Copy to editor\" keeps them."),
        Console("TRI 10"),
        WaitFinished,
        CheckConsole("In the console: I don't know how to TRI"),
        Say("Errors show while typing"),
        CheckDiagnostics(&[]),
        // The editor sends its change event once the edit is done.
        Type("FD 10\nFOOBAR 3\nREPEAT 4 [FD]"),
        Wait(ms(100)),
        CheckDiagnostics(&["I don't know how to FOOBAR", "Not enough inputs to FD"]),
        Screenshot("errors-while-typing"),
        Type("REPEAT 4 [FD 10 RT 90]"),
        Wait(ms(100)),
        CheckDiagnostics(&[]),
        Type(crate::app::WELCOME),
        Say("Files"),
        CheckTitle("Untitled — Pendown"),
        Type("REPEAT 3 [FD 200 RT 120]"),
        CheckTitle("● Untitled — Pendown"),
        SaveTo("triangle"),
        CheckTitle("triangle.logo — Pendown"),
        Run("REPEAT 5 [FD 200 RT 144]", Speed::Instant),
        WaitFinished,
        CheckTitle("● triangle.logo — Pendown"),
        ExportTo("star.png"),
        CheckExported("star.png"),
        ExportTo("star.svg"),
        CheckExportedSvg("star.svg"),
        OpenFrom("triangle.logo"),
        CheckEditor("REPEAT 3 [FD 200 RT 120]"),
        CheckTitle("triangle.logo — Pendown"),
        Say("Highlighting follows the turtle"),
        Trace,
        Run(BOX, Speed::Animated(100.0)),
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
        CheckConsole("    in BOX (line 6)"),
        Run("FD 10\nRT", Speed::Instant),
        WaitFinished,
        CheckConsole("Line 2: Not enough inputs to RT"),
        Say("Phase 2: filled shapes, FOR and a template, animated"),
        Run("FOR [i 3 1] [SETFC ITEM :i [red orange gold] CIRCLE :i * 50]\nPRINT MAP [? * 2] [1 2 3]", Speed::Animated(2560.0)),
        WaitFinished,
        CheckDrawn(3),
        CheckConsole("2 4 6"),
        Screenshot("filled"),
        Say("Phase 2: CATCH \"ERROR, and an error made by THROW"),
        Run(CATCHING, Speed::Instant),
        WaitFinished,
        CheckConsole("[0 FD doesn't like x as input [] 1]"),
        CheckConsole("Line 6: CHECK needs a positive number"),
        Say("Phase 3: a slow run draws only the frames that change"),
        Run(SLOW, level_speed(3)),
        Wait(ms(300)),
        MeasureRenders,
        Wait(ms(3000)),
        CheckRenders(25.0),
        Stop,
        Say("Phase 3: the metrics overlay"),
        ToggleMetrics,
        Run("REPEAT 36 [REPEAT 4 [FD 200 RT 90] RT 10]", level_speed(10)),
        Wait(ms(1200)),
        CheckOverlay(true),
        Screenshot("metrics"),
        Stop,
        ToggleMetrics,
        CheckOverlay(false),
        Say("Phase 4: the counters and variables keep in step with the highlight"),
        Run(WATCHED, Speed::Animated(100.0)),
        Pause,
        StepInstruction,
        WaitPaused,
        CheckHighlight("MAKE \"sizes [40 80]"),
        CheckPanels(""),
        StepInstruction,
        WaitPaused,
        CheckPanels("sizes=[40 80]"),
        StepInstruction,
        WaitPaused,
        CheckHighlight("TILE ?"),
        CheckPanels("FOREACH 1 of 2* | sizes=[40 80]"),
        StepInstruction,
        WaitPaused,
        StepInstruction,
        WaitPaused,
        CheckHighlight("FD :size"),
        CheckPanels("FOREACH 1 of 2 | REPEAT 1 of 4* | sizes=[40 80] | TILE: size=40"),
        Wait(ms(100)),
        CheckPanelsDrawn,
        Screenshot("panels"),
        SetSpeed(Speed::Instant),
        Resume,
        WaitFinished,
        CheckPanels("sizes=[40 80]"),
        Say("Phase 4: at Instant the panels update as it runs, and Stop leaves the globals"),
        Run(COUNTING, Speed::Instant),
        Wait(ms(300)),
        CheckPanelsStart("FOREVER "),
        Wait(ms(100)),
        CheckPanelsDrawn,
        Screenshot("panels-instant"),
        Stop,
        CheckPanelsStart("n="),
        Wait(ms(100)),
        CheckPanelsDrawn,
        Say("Phase 4: the colours, with a square of each"),
        Run("SETPC \"orange\nSETBG [20 40 80]\nSETFC \"#40c0ff80\nCIRCLE 100", Speed::Animated(200.0)),
        WaitFinished,
        CheckColours("PENCOLOR=orange FILLCOLOR=#40c0ff80 BACKGROUND=[20 40 80]"),
        Screenshot("colours"),
        Say("Phase 5: help for the word under the cursor"),
        Type(crate::app::WELCOME),
        F1At("FD :size"),
        Wait(ms(500)),
        CheckHelpPage("FORWARD"),
        Screenshot("help"),
        CursorAt("RT 90"),
        Wait(ms(100)),
        CheckHelpPage("RIGHT"),
        CursorAt(":size"),
        Wait(ms(100)),
        CheckHelpPage("Variables: :name"),
        Say("Phase 5: search and contents"),
        SearchHelp("setp"),
        Wait(ms(100)),
        CheckFirstResult("SETPENCOLOR"),
        Screenshot("help-search"),
        SearchHelp(""),
        OpenHelp("circle"),
        HelpContents,
        Wait(ms(100)),
        Screenshot("help-contents"),
        Say("Phase 5: an example runs on the canvas, and the user's drawing comes back"),
        Run(crate::app::WELCOME, Speed::Instant),
        WaitFinished,
        CheckDrawn(48),
        OpenHelp("forward"),
        SetSpeed(Speed::Animated(20.0)),
        RunExample("forward", 0),
        Wait(ms(900)),
        CheckExampleHighlight("FD 40"),
        Screenshot("example"),
        SetSpeed(Speed::Instant),
        WaitFinished,
        CheckDrawn(8),
        BackToMyDrawing,
        CheckDrawn(48),
        CopyExample("forward", 1),
        CheckEditorEnds("FLOWER 250\n\nREPEAT 5 [FD 20 PU FD 20 PD]\n"),
        CloseHelp,
        Say("Phase 5: errors offer help on their command"),
        Run("REPEAT 3 [FD]", Speed::Instant),
        WaitFinished,
        CheckErrorHelp("FORWARD"),
        Run("EDIT \"square", Speed::Instant),
        WaitFinished,
        CheckConsole("Line 1: EDIT isn't in Pendown: it opens an external text editor, and Pendown has its own."),
        CheckErrorHelp("EDIT"),
        Wait(ms(100)),
        Screenshot("error-help"),
        Say("Phase 5: a lesson's programs load into the editor and run"),
        Lesson(0),
        Wait(ms(100)),
        Screenshot("tutorial-first"),
        Lesson(1),
        LoadLesson(1, 0),
        CheckEditor("REPEAT 4 [FD 100 RT 90]"),
        CheckTitle("Untitled — Pendown"),
        SetSpeed(level_speed(7)),
        RunLesson(1, 3),
        Wait(ms(1500)),
        Screenshot("tutorial"),
        SetSpeed(Speed::Instant),
        WaitFinished,
        CheckDrawn(32),
        BackToMyDrawing,
        CloseHelp,
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
    /// The metrics, highlight changes and console lines when `MeasureRenders` ran.
    measuring: Option<(Snapshot, u64, usize)>,
    highlights: u64,
}

impl Autotest {
    pub fn from_env() -> Option<Autotest> {
        let value = std::env::var("PENDOWN_AUTOTEST").ok()?;
        let dir = if value == "1" { PathBuf::from("target/autotest") } else { PathBuf::from(value) };
        std::fs::create_dir_all(&dir).ok();
        let steps = script();
        // PENDOWN_AUTOTEST_FROM="Phase 5" starts at the first section whose
        // title has that in it.
        let next = std::env::var("PENDOWN_AUTOTEST_FROM")
            .ok()
            .and_then(|from| steps.iter().position(|s| matches!(s, Step::Say(text) if text.contains(from.as_str()))))
            .unwrap_or(0);
        Some(Autotest {
            steps,
            next,
            waiting: None,
            log: String::new(),
            failures: 0,
            dir,
            screenshots: 0,
            trace: None,
            track: None,
            measuring: None,
            highlights: 0,
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

    /// Called at every frame that plays the animation.
    pub(crate) fn on_frame(&mut self, now: Instant) {
        let y = self.animator.as_ref().map_or(0.0, |a| a.pos.y);
        if let Some(track) = self.autotest.as_mut().and_then(|a| a.track.as_mut()) {
            track.push((now, y));
        }
    }

    /// Called whenever the highlight moves.
    pub(crate) fn on_highlight(&mut self) {
        if let Some(auto) = &mut self.autotest {
            auto.highlights += 1;
        }
        let text = self.highlighted();
        if let Some(trace) = self.autotest.as_mut().and_then(|a| a.trace.as_mut()) {
            trace.push(text);
        }
    }

    /// The editor text the highlight is on. Console entries aren't in the
    /// editor, so their spans don't index `run_text`.
    fn highlighted(&self) -> String {
        self.animator
            .as_ref()
            .and_then(|a| a.highlight)
            .filter(|s| s.source == SourceId::Editor)
            .map_or(String::new(), |s| self.run_text[s.range()].to_string())
    }

    /// Takes the session's next steps, until one has to wait. Returns
    /// whether it took any, so the window should render.
    pub(crate) fn drive_autotest(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.autotest.is_none() {
            return false;
        }
        let mut took = false;
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
                    return took;
                }
                self.autotest.as_mut().unwrap().waiting = None;
                continue;
            }
            let Some(step) = auto.steps.get(auto.next).cloned() else { return took };
            auto.next += 1;
            took = true;
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
                Step::Resume => self.animator.as_mut().map_or((), |a| a.resume()),
                Step::StepLine => self.animator.as_mut().map_or((), |a| a.step_line()),
                Step::StepInstruction => self.animator.as_mut().map_or((), |a| a.step_instruction()),
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
                    let moving = self.animator.as_ref().is_some_and(|a| a.partial(1.0).is_some());
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
                Step::Console(line) => {
                    self.type_into_console(line, window, cx);
                }
                Step::SaveTo(name) => {
                    let path = self.autotest.as_ref().unwrap().dir.join(name);
                    self.save_to(&path, cx);
                }
                Step::OpenFrom(name) => {
                    let path = self.autotest.as_ref().unwrap().dir.join(name);
                    self.open_path(&path, window, cx);
                }
                Step::ExportTo(name) => {
                    let path = self.autotest.as_ref().unwrap().dir.join(name);
                    self.export_to(&path, crate::files::Picture::Png, cx);
                }
                Step::CursorAt(marker) | Step::F1At(marker) => {
                    let text = self.editor.read(cx).value().to_string();
                    let offset = text.find(marker).map_or(0, |i| i + 1);
                    let at = crate::app::position(&text, offset);
                    self.editor.update(cx, |state, cx| state.set_cursor_position(at, window, cx));
                    if matches!(step, Step::F1At(_)) {
                        self.show_help(window, cx);
                    }
                }
                Step::CheckHelpPage(expected) => {
                    let page = self.help_panel.read(cx).page;
                    let title = match page {
                        crate::help_panel::Page::Command(i) => pendown_logo::reference::reference().commands[i].title.clone(),
                        crate::help_panel::Page::Contents => "Contents".to_string(),
                    };
                    self.check(self.help_open && title == expected, format!("the help panel shows {title}"));
                }
                Step::SearchHelp(query) => {
                    let search = self.help_panel.read(cx).search.clone();
                    search.update(cx, |s, cx| s.set_value(query.to_string(), window, cx));
                    self.help_panel.update(cx, |_, cx| cx.notify());
                }
                Step::CheckFirstResult(expected) => {
                    let query = self.help_panel.read(cx).query(cx);
                    let r = pendown_logo::reference::reference();
                    let first = r.search(&query, false).first().map(|&i| r.commands[i].title.clone()).unwrap_or_default();
                    self.check(first == expected, format!("the first result for {query:?} is {first}"));
                }
                Step::OpenHelp(name) => {
                    self.help_panel.update(cx, |p, cx| p.open_name(name, window, cx));
                }
                Step::HelpContents => {
                    self.help_panel.update(cx, |p, cx| p.go(crate::help_panel::Page::Contents, window, cx));
                }
                Step::RunExample(name, n) => {
                    let i = pendown_logo::reference::reference().find_index(name).expect("a help page");
                    self.run_example(i, n, cx);
                }
                Step::CheckExampleHighlight(expected) => {
                    let running = self.help_panel.read(cx).running.clone();
                    let text = running
                        .as_ref()
                        .and_then(|r| r.highlight.clone())
                        .map_or(String::new(), |range| self.run_text.get(range).unwrap_or_default().to_string());
                    self.check(text == expected, format!("the example's highlight is on {text:?}"));
                }
                Step::BackToMyDrawing => self.back_to_my_drawing(cx),
                Step::CopyExample(name, n) => {
                    let r = pendown_logo::reference::reference();
                    let code = r.find(name).expect("a help page").examples[n].code.clone();
                    self.copy_to_editor(&code, window, cx);
                }
                Step::CheckEditorEnds(expected) => {
                    let text = self.editor.read(cx).value().to_string();
                    let tail = &text[text.len().saturating_sub(expected.len() + 20)..];
                    self.check(text.ends_with(expected), format!("the editor ends with {tail:?}"));
                }
                Step::CheckErrorHelp(expected) => {
                    let r = pendown_logo::reference::reference();
                    let title = self.error_help.map_or(String::new(), |i| r.commands[i].title.clone());
                    self.check(title == expected, format!("the console offers help on {title:?}"));
                    if let Some(i) = self.error_help {
                        self.open_help(i, window, cx);
                    }
                }
                Step::Lesson(l) => {
                    if !self.help_open {
                        self.toggle_help(cx);
                    }
                    self.help_panel.update(cx, |p, cx| {
                        p.tab = crate::help_panel::Tab::Tutorial;
                        p.lesson = l;
                        cx.notify();
                    });
                }
                Step::LoadLesson(lesson, program) => self.proceed(crate::files::After::Load { lesson, program }, window, cx),
                Step::RunLesson(lesson, program) => self.run_source(crate::help_panel::Source::Lesson { lesson, program }, cx),
                Step::CloseHelp => {
                    if self.help_open {
                        self.toggle_help(cx);
                    }
                }
                Step::Type(text) => {
                    // Like typing: an edit the user could undo, which sends
                    // a change event. `set_value` sends none.
                    self.editor.update(cx, |state, cx| state.replace_all(text.to_string(), window, cx));
                }
                Step::CheckTitle(expected) => {
                    self.update_title(window, cx);
                    let title = self.title.clone();
                    self.check(title == expected, format!("the title is {title:?}"));
                }
                Step::CheckDiagnostics(expected) => {
                    let found: Vec<String> = self.editor.update(cx, |state, _| {
                        state.diagnostics_mut().map_or(Vec::new(), |set| set.iter().map(|d| d.diagnostic.message.to_string()).collect())
                    });
                    self.check(found == expected, format!("the editor shows {found:?}"));
                }
                Step::CheckEditor(expected) => {
                    let text = self.editor.read(cx).value().to_string();
                    self.check(text == expected, format!("the editor holds {text:?}"));
                }
                Step::CheckExported(name) => {
                    let path = self.autotest.as_ref().unwrap().dir.join(name);
                    let size = {
                        let canvas = self.canvas.borrow();
                        (canvas.drawing.width(), canvas.drawing.height())
                    };
                    let image = image::open(&path).map(|i| i.to_rgba8());
                    let found = image.as_ref().map(|i| i.dimensions());
                    let drawn = image.as_ref().map_or(0, |i| i.pixels().filter(|p| p.0 != [0, 0, 0, 255]).count());
                    let ok = found.as_ref().is_ok_and(|s| *s == size) && drawn > 0;
                    self.check(ok, format!("exported a {found:?} PNG with {drawn} drawn pixels, the canvas is {size:?}"));
                }
                Step::CheckExportedSvg(name) => {
                    let path = self.autotest.as_ref().unwrap().dir.join(name);
                    let svg = std::fs::read_to_string(&path).unwrap_or_default();
                    // The background, then a line for each mark.
                    let marks = svg.lines().filter(|l| l.starts_with('<') && !l.starts_with("</") && !l.starts_with("<svg") && !l.starts_with("<g")).count() - 1;
                    let drawn = self.canvas.borrow().drawing.segments();
                    self.check(svg.starts_with("<svg ") && marks == drawn, format!("exported an SVG with {marks} marks, and {drawn} were drawn"));
                }
                Step::CheckDrawn(expected) => {
                    let drawn = self.canvas.borrow().drawing.segments();
                    self.check(drawn == expected, format!("{drawn} lines and shapes drawn"));
                }
                Step::CheckTurtle(x, y, heading) => {
                    let (fx, fy, found) = self.animator.as_ref().map_or((0.0, 0.0, 0.0), |a| (a.pos.x, a.pos.y, a.heading));
                    let ok = (fx - x).abs() < 0.01 && (fy - y).abs() < 0.01 && (found - heading).abs() < 0.01;
                    self.check(ok, format!("the turtle is at {fx:.1}, {fy:.1}, heading {found:.1}"));
                }
                Step::CheckConsole(expected) => {
                    let found = self.console.iter().any(|l| l.as_ref() == expected);
                    self.check(found, format!("the console shows {expected:?}"));
                }
                Step::MeasureRenders => {
                    pendown_logo::metrics::set_enabled(true);
                    let start = (self.snapshot(), self.autotest.as_ref().unwrap().highlights, self.console.len());
                    self.autotest.as_mut().unwrap().measuring = Some(start);
                }
                Step::CheckRenders(per_second) => {
                    let auto = self.autotest.as_mut().unwrap();
                    let (start, highlights, lines) = auto.measuring.take().expect("MeasureRenders came first");
                    let highlights = auto.highlights - highlights;
                    let printed = self.console.len().saturating_sub(lines) as u64;
                    let i = self.snapshot().since(&start);
                    let rate = i.renders as f64 / i.wall.as_secs_f64();
                    // A frame or two more, for the renders that start and
                    // end the measurement.
                    self.check(
                        rate <= per_second && i.renders <= i.frames + 2,
                        format!("{} renders at {rate:.0}/s, for {} animation frames", i.renders, i.frames),
                    );
                    self.check(
                        i.editor_renders <= highlights + 2 && i.console_renders <= printed + 2,
                        format!(
                            "the editor rendered {} times for {highlights} highlight changes, the console {} times for {printed} lines",
                            i.editor_renders, i.console_renders
                        ),
                    );
                }
                Step::CheckPanels(expected) => {
                    let shown = self.panels(cx);
                    self.check(shown == expected, format!("the panels show {shown:?}"));
                }
                Step::CheckPanelsDrawn => {
                    let watch = self.watched.read(cx).watch.clone();
                    let same = |drawn: &Option<std::sync::Arc<pendown_logo::Watch>>| match (drawn, &watch) {
                        (Some(a), Some(b)) => std::sync::Arc::ptr_eq(a, b),
                        (None, None) => true,
                        _ => false,
                    };
                    let counters = same(&self.counters_pane.read(cx).drawn);
                    let variables = same(&self.variables_pane.read(cx).drawn);
                    self.check(counters && variables, format!("the counters drew the latest: {counters}, the variables: {variables}"));
                }
                Step::CheckColours(expected) => {
                    let watch = self.watched.read(cx).watch.clone();
                    let shown = watch.map_or(String::new(), |w| {
                        w.colours.iter().map(|c| format!("{}={}", c.name, c.value)).collect::<Vec<_>>().join(" ")
                    });
                    self.check(shown == expected, format!("the colours are {shown:?}"));
                }
                Step::CheckPanelsStart(expected) => {
                    let shown = self.panels(cx);
                    self.check(shown.starts_with(expected), format!("the panels show {shown:?}"));
                }
                Step::ToggleMetrics => self.toggle_metrics(window, cx),
                Step::CheckOverlay(shown) => {
                    let lines = self.overlay.as_ref().map(|o| o.lines.clone());
                    let ok = match &lines {
                        Some(lines) => shown && lines.first().is_some_and(|l| l.contains(" renders/s, UI CPU ")),
                        None => !shown,
                    };
                    self.check(ok, format!("the overlay shows {lines:?}"));
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

    /// What the counters and variables panels show, in short: each counter,
    /// with `*` on the one `REPCOUNT` outputs, the globals, then each scope.
    fn panels(&self, cx: &gpui_kit::App) -> String {
        let Some(watch) = self.watched.read(cx).watch.clone() else { return String::new() };
        let mut parts: Vec<String> = watch
            .counters
            .iter()
            .map(|c| {
                let of = c.of.map_or(String::new(), |of| format!(" of {of}"));
                format!("{} {}{of}{}", c.name, c.count, if c.repcount { "*" } else { "" })
            })
            .collect();
        for scope in &watch.scopes {
            let vars: Vec<String> =
                scope.variables.iter().map(|v| format!("{}={}", v.name, v.value.as_deref().unwrap_or("-"))).collect();
            if !scope.name.is_empty() {
                parts.push(format!("{}: {}", scope.name, vars.join(" ")));
            } else if !vars.is_empty() {
                parts.push(vars.join(" "));
            }
        }
        parts.join(" | ")
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
        let title = self.title.replace('\'', "");
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("xwd -silent -name '{title}' | convert xwd:- '{}'", path.display()))
            .status();
        let ok = status.is_ok_and(|s| s.success());
        self.say(format!("  screenshot {}{}", path.display(), if ok { "" } else { " (failed: needs xwd and ImageMagick)" }));
    }
}
