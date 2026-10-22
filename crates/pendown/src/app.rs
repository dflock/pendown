//! The window: editor, canvas and console, with the run controls (ADRs 0006,
//! 0011 and 0012).

use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use gpui_kit::component::{
    ActiveTheme, Disableable as _, Sizable,
    button::{Button, ButtonVariants as _},
    h_flex,
    highlighter::{Diagnostic, DiagnosticSeverity},
    input::{
        EditorState, InputEvent, InputHighlighter, InputState, MoveDown, MoveUp, Position, RangeDecoration,
        RangeDecorationCollection, RangeDecorationStyle, TextareaState,
    },
    resizable::{h_resizable, resizable_panel},
    slider::{SliderEvent, SliderState},
    v_flex,
};
use gpui_kit::{prelude::FluentBuilder as _, *};
use pendown::{
    animator::{Animator, MAX_FRAME_GAP, Mode, Speed},
    console::{ConsoleInput, Submit, defines},
    document::Document,
    errors,
    metrics::{Snapshot, WindowMetrics},
    run::{Channel, DEFAULT_CHANNEL, Job, Run, Session},
    speed::{INSTANT_LEVEL, level_speed, speed_level},
};
use pendown_logo::{RunError, SourceId};

use crate::{
    autotest::Autotest,
    canvas::{Canvas, Overlay, TURTLE_LENGTH},
    files::{After, ExportPicture, NewFile, OpenFile, RunProgram, SaveFile, SaveFileAs, ToggleMetrics},
    highlighter::LogoHighlighter,
    panes::{ConsolePane, EditorPane, ToolbarPane, ToolbarState},
    watching::{CountersPane, VariablesPane, Watched},
};

/// The program the editor starts with.
pub const WELCOME: &str = r#"; Press Run, and watch the turtle draw.
; Try the speed slider, Pause and Step.

TO BOX :size
  REPEAT 4 [
    FD :size
    RT 90
  ]
END

TO FLOWER :size
  REPEAT 12 [
    BOX :size
    RT 30
  ]
END

SETPC 6
FLOWER 250
"#;

/// Where a byte offset is in `text`, as the editor counts lines and
/// columns: columns in UTF-16 units, as in LSP.
fn position(text: &str, offset: usize) -> Position {
    let before = &text[..offset.min(text.len())];
    let line = before.matches('\n').count() as u32;
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    Position::new(line, before[line_start..].encode_utf16().count() as u32)
}

/// The live numbers shown over the canvas: each line covers about the last
/// `OVERLAY_EVERY`.
pub(crate) struct MetricsOverlay {
    since: Snapshot,
    pub(crate) lines: Vec<String>,
    /// Keeps the overlay refreshing while there are no frames.
    _refresh: Task<()>,
}

const OVERLAY_EVERY: Duration = Duration::from_millis(500);

/// How often the scripted session takes its next steps. It runs on a timer,
/// not on frames, so it doesn't change how often the window renders.
const AUTOTEST_EVERY: Duration = Duration::from_millis(5);

/// The toolbar's height, which its cached view needs, and the width of the
/// status beside it.
const TOOLBAR_HEIGHT: f32 = 41.0;
const STATUS_WIDTH: f32 = 300.0;

/// The console area's height, with the counters and variables beside the
/// console.
const CONSOLE_HEIGHT: f32 = 190.0;

/// A change due sooner than this is left to the next animation frame.
const FRAME_SOON: Duration = Duration::from_millis(12);

/// Instant mode draws at most about this much a frame, so the window stays
/// responsive (ADR 0011).
const INSTANT_DRAWING: Duration = Duration::from_millis(6);

#[derive(Default, Clone)]
pub struct Samples(pub Vec<Duration>);

impl Samples {
    pub fn push(&mut self, d: Duration) {
        self.0.push(d);
    }

    pub fn percentile(&self, p: usize) -> Option<Duration> {
        let mut v = self.0.clone();
        v.sort();
        v.get((v.len() * p / 100).min(v.len().saturating_sub(1))).copied()
    }

    pub fn summary(&self) -> String {
        let ms = |d: Option<Duration>| d.map_or(0.0, |d| d.as_secs_f64() * 1e3);
        format!(
            "n={} p50={:.1}ms p95={:.1}ms max={:.1}ms",
            self.0.len(),
            ms(self.percentile(50)),
            ms(self.percentile(95)),
            ms(self.0.iter().max().copied())
        )
    }
}

pub struct App {
    pub(crate) editor: Entity<EditorState>,
    highlight: RangeDecorationCollection,
    speed_slider: Entity<SliderState>,
    pub(crate) speed: Speed,
    pub(crate) canvas: Rc<RefCell<Canvas>>,
    pub(crate) animator: Option<Animator>,
    pub(crate) run: Option<Run>,
    /// The text that was run, which spans point into (ADR 0006).
    pub(crate) run_text: String,
    pub(crate) finished_at: Option<Instant>,
    pub(crate) stop_at: Option<Instant>,
    /// The interpreter, which lives between runs for the console.
    session: Session,
    /// The program's file, if it has one, and whether it has changed.
    pub(crate) document: Document,
    pub(crate) title: String,
    /// The user has said the window can close.
    pub(crate) closing: bool,
    /// The program has been checked for errors since the window opened.
    checked: bool,
    /// Console output, one line each.
    pub(crate) console: Vec<SharedString>,
    console_dirty: bool,
    console_output: Entity<TextareaState>,
    console_input: Entity<InputState>,
    /// The editor and console, drawn as cached views (issue 08).
    editor_pane: Entity<EditorPane>,
    console_pane: Entity<ConsolePane>,
    toolbar_pane: Entity<ToolbarPane>,
    /// What the counters and variables panels show (phase 4).
    pub(crate) watched: Entity<Watched>,
    pub(crate) counters_pane: Entity<CountersPane>,
    pub(crate) variables_pane: Entity<VariablesPane>,
    console_entry: ConsoleInput,
    /// Definitions typed at the console since the last Run, which "Copy to
    /// editor" keeps.
    console_definitions: Vec<String>,
    /// The console entry that's running, if it defines procedures.
    defining: Option<String>,
    /// A Run has cleared `console_definitions` since the last was added, and
    /// said so.
    definitions_cleared: bool,
    last_frame: Option<Instant>,
    pub(crate) frame_intervals: Samples,
    pub(crate) events: usize,
    pub(crate) autotest: Option<Autotest>,
    pub(crate) metrics: Rc<WindowMetrics>,
    /// The numbers when the current job started, for its summary.
    run_start: Option<Snapshot>,
    /// The metrics overlay, when it's shown (F12).
    pub(crate) overlay: Option<MetricsOverlay>,
    /// Wakes the window for its next frame, when that's more than a frame
    /// away.
    wake: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl App {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> App {
        let editor = cx.new(|cx| {
            EditorState::new(window, cx).language("logo").line_number(true).soft_wrap(false).default_value(WELCOME)
        });
        let highlight = editor.update(cx, |state, cx| {
            state.set_highlighter_factory(
                Rc::new(|language| (language == "logo").then(|| Box::new(LogoHighlighter::default()) as Box<dyn InputHighlighter>)),
                cx,
            );
            state.create_range_decorations_collection(vec![], cx)
        });
        let console_output = cx.new(|cx| TextareaState::new(window, cx));
        let console_input = cx.new(|cx| InputState::new(window, cx).placeholder("Type an instruction and press Enter"));
        let speed = level_speed(5);
        let speed_slider = cx.new(|_| {
            SliderState::new().min(1.0).max(INSTANT_LEVEL as f32).step(1.0).default_value(speed_level(speed) as f32)
        });
        let subscriptions = vec![
            cx.subscribe_in(&speed_slider, window, |this, _, event: &SliderEvent, window, cx| {
                if let SliderEvent::Change(value) = event {
                    this.set_speed(level_speed(value.end().round() as u8), window, cx);
                }
            }),
            // Parse errors show while typing (ADRs 0001 and 0005).
            cx.subscribe(&editor, |this, _, event: &InputEvent, cx| {
                if let InputEvent::Change = event
                    && !this.running()
                {
                    this.check_program(cx);
                }
            }),
            cx.subscribe_in(&console_input, window, |this, _, event: &InputEvent, window, cx| {
                if let InputEvent::PressEnter { .. } = event {
                    this.submit_console(window, cx);
                }
            }),
        ];
        let metrics = Rc::new(WindowMetrics::default());
        let editor_pane = cx.new(|_| EditorPane::new(editor.clone(), metrics.clone()));
        let console_pane = cx.new(|_| ConsolePane::new(console_output.clone(), console_input.clone(), metrics.clone()));
        let app = cx.entity().downgrade();
        let toolbar = ToolbarState { running: false, paused: false, speed };
        let toolbar_pane = cx.new(|_| ToolbarPane::new(app.clone(), speed_slider.clone(), toolbar));
        let watched = cx.new(|_| Watched::default());
        let counters_pane = cx.new(|_| CountersPane::new(watched.clone()));
        let variables_pane = cx.new(|_| VariablesPane::new(watched.clone()));
        window.on_window_should_close(cx, move |window, cx| {
            app.update(cx, |this, cx| this.should_close(window, cx)).unwrap_or(true)
        });
        if std::env::var_os("PENDOWN_AUTOTEST").is_some() {
            cx.spawn_in(window, async move |this, cx| {
                loop {
                    cx.background_executor().timer(AUTOTEST_EVERY).await;
                    let alive = this.update_in(cx, |this: &mut App, window, cx| {
                        if this.drive_autotest(window, cx) {
                            cx.notify();
                        }
                    });
                    if alive.is_err() {
                        break;
                    }
                }
            })
            .detach();
        }
        App {
            editor,
            highlight,
            speed_slider,
            speed,
            canvas: Rc::new(RefCell::new(Canvas::new(metrics.clone()))),
            animator: None,
            run: None,
            run_text: String::new(),
            finished_at: None,
            stop_at: None,
            session: Session::new(),
            document: Document::new(WELCOME),
            title: String::new(),
            closing: false,
            checked: false,
            console: Vec::new(),
            console_dirty: false,
            console_output,
            console_input,
            editor_pane,
            console_pane,
            toolbar_pane,
            watched,
            counters_pane,
            variables_pane,
            console_entry: ConsoleInput::default(),
            console_definitions: Vec::new(),
            defining: None,
            definitions_cleared: false,
            last_frame: None,
            frame_intervals: Samples::default(),
            events: 0,
            autotest: Autotest::from_env(),
            metrics,
            run_start: None,
            overlay: None,
            wake: None,
            _subscriptions: subscriptions,
        }
    }

    /// Sets the speed, and moves the slider to match.
    pub(crate) fn set_speed(&mut self, speed: Speed, window: &mut Window, cx: &mut Context<Self>) {
        if speed == self.speed {
            return;
        }
        self.speed = speed;
        if let Some(animator) = &mut self.animator {
            animator.set_speed(speed);
        }
        let level = speed_level(speed);
        if self.speed_slider.read(cx).value().end().round() as u8 != level {
            self.speed_slider.update(cx, |s, cx| s.set_value(level as f32, window, cx));
        }
        cx.notify();
    }

    pub(crate) fn running(&self) -> bool {
        self.animator.as_ref().is_some_and(|a| !a.finished())
    }

    /// Run (ADR 0006): stop any running program, clear the drawing, and
    /// start the editor's program from a clean slate.
    pub(crate) fn start(&mut self, channel: Channel, cx: &mut Context<Self>) {
        self.stop(cx);
        let text = self.editor.read(cx).value().to_string();
        self.run_text = text.clone();
        self.canvas.borrow_mut().drawing.reset();
        self.console.clear();
        self.console_dirty = true;
        if !self.console_definitions.is_empty() && !std::mem::replace(&mut self.definitions_cleared, true) {
            self.print("Run cleared the procedures you defined in the console. \"Copy to editor\" keeps them.");
        }
        self.set_diagnostics(Vec::new(), cx);
        // Run clears the variables (ADR 0006).
        self.show_watch(cx, |watched| watched.set(None));
        self.start_job(Job::Program(text), channel, cx);
    }

    /// Runs a console entry against whatever the last run left (ADR 0006).
    fn run_console(&mut self, entry: String, cx: &mut Context<Self>) {
        self.stop(cx);
        self.defining = defines(&entry).then(|| entry.clone());
        self.start_job(Job::Console(entry), DEFAULT_CHANNEL, cx);
    }

    fn start_job(&mut self, job: Job, channel: Channel, cx: &mut Context<Self>) {
        let (animator, run) = self.session.start(job, self.speed, channel);
        self.animator = Some(animator);
        self.run = Some(run);
        self.finished_at = None;
        self.stop_at = None;
        self.last_frame = None;
        self.frame_intervals = Samples::default();
        self.events = 0;
        self.run_start = pendown_logo::metrics::enabled().then(|| self.snapshot());
        cx.notify();
    }

    /// Every metric now (ADR 0020).
    pub(crate) fn snapshot(&self) -> Snapshot {
        let canvas = self.canvas.borrow();
        Snapshot::take(
            &self.metrics,
            &canvas.drawing.metrics,
            self.animator.as_ref().map(|a| &a.metrics),
            self.run.as_ref().map(|r| &r.control.metrics),
        )
    }

    /// F12: shows or hides the metrics overlay, turning metrics on if they
    /// were off.
    pub(crate) fn toggle_metrics(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.overlay.take().is_none() {
            pendown_logo::metrics::set_enabled(true);
            let refresh = cx.spawn_in(window, async move |this, cx| {
                loop {
                    cx.background_executor().timer(OVERLAY_EVERY).await;
                    if this.update(cx, |_, cx| cx.notify()).is_err() {
                        break;
                    }
                }
            });
            self.overlay = Some(MetricsOverlay { since: self.snapshot(), lines: vec!["Measuring…".into()], _refresh: refresh });
        }
        cx.notify();
    }

    /// Brings the overlay's numbers up to date, every `OVERLAY_EVERY`.
    fn update_overlay(&mut self) {
        let Some(since) = self.overlay.as_ref().map(|o| o.since) else { return };
        if since.at.elapsed() < OVERLAY_EVERY {
            return;
        }
        let now = self.snapshot();
        let overlay = self.overlay.as_mut().expect("checked above");
        overlay.lines = now.since(&since).lines();
        overlay.since = now;
    }

    /// Asks for the next frame when the picture will next look different
    /// (issue 08): at once while it changes every frame, and otherwise after
    /// a wait. At slow speeds most frames would look the same, and each one
    /// costs the UI thread a few milliseconds.
    fn schedule_frame(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.wake = None;
        let Some(animator) = self.animator.as_ref().filter(|a| a.wants_frames()) else { return };
        let px_per_step = self.canvas.borrow().drawing.scale();
        let mut gap = animator.next_change(px_per_step, TURTLE_LENGTH as f64 * window.scale_factor() as f64);
        if gap < FRAME_SOON {
            window.request_animation_frame();
            // gpui can drop an animation frame, as when it throttles a window
            // that isn't focused, and then nothing would wake the run: so a
            // timer too, as a fallback.
            gap = MAX_FRAME_GAP;
        }
        self.wake = Some(cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(gap).await;
            this.update(cx, |_, cx| cx.notify()).ok();
        }));
    }

    /// Changes what the counters and variables panels show, and redraws
    /// them. They're cached views of `Watched`, and notifying it alone
    /// doesn't redraw them.
    fn show_watch(&mut self, cx: &mut Context<Self>, f: impl FnOnce(&mut Watched)) {
        self.watched.update(cx, |watched, _| f(watched));
        self.counters_pane.update(cx, |_, cx| cx.notify());
        self.variables_pane.update(cx, |_, cx| cx.notify());
    }

    /// Prints the summary of the job that just ended on stderr.
    fn report(&mut self) {
        if let Some(start) = self.run_start.take() {
            eprintln!("{}", self.snapshot().since(&start).summary());
        }
    }

    pub(crate) fn stop(&mut self, cx: &mut Context<Self>) {
        if let Some(animator) = &mut self.animator
            && !animator.finished()
        {
            animator.stop();
            self.show_watch(cx, Watched::stopped);
            self.stop_at = Some(Instant::now());
            self.print("Stopped.");
            self.report();
        }
        cx.notify();
    }

    /// Adds a line to the console.
    pub(crate) fn print(&mut self, line: impl Into<SharedString>) {
        self.console.push(line.into());
        // A long-running program can print a lot: keep the newest.
        if self.console.len() > 2000 {
            self.console.drain(..self.console.len() - 2000);
        }
        self.console_dirty = true;
    }

    /// For the scripted session: as if the user typed `line` and pressed Enter.
    pub(crate) fn type_into_console(&mut self, line: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.console_input.update(cx, |input, cx| input.set_value(line.to_string(), window, cx));
        self.submit_console(window, cx);
    }

    fn submit_console(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.running() {
            return;
        }
        let line = self.console_input.read(cx).value().to_string();
        self.console_input.update(cx, |input, cx| input.set_value("", window, cx));
        let prompt = if self.console_entry.continuing() { "…" } else { ">" };
        self.print(format!("{prompt} {line}"));
        match self.console_entry.submit(&line) {
            Submit::More => {}
            Submit::Run(entry) if entry.trim().is_empty() => {}
            Submit::Run(entry) => self.run_console(entry, cx),
        }
    }

    /// Up and Down in the console's input line go through its history.
    fn recall(&mut self, older: bool, window: &mut Window, cx: &mut Context<Self>) {
        let text = if older { self.console_entry.older() } else { self.console_entry.newer() };
        if let Some(text) = text.map(str::to_string) {
            // A multi-line entry comes back on one line, which runs the same.
            let text = text.replace('\n', " ");
            self.console_input.update(cx, |input, cx| input.set_value(text, window, cx));
        }
    }

    /// Appends the definitions typed at the console to the editor's program.
    fn copy_definitions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut text = self.editor.read(cx).value().to_string();
        for definition in self.console_definitions.drain(..) {
            if !text.ends_with('\n') && !text.is_empty() {
                text.push('\n');
            }
            text.push('\n');
            text.push_str(&definition);
            text.push('\n');
        }
        self.editor.update(cx, |state, cx| state.set_value(text, window, cx));
        self.check_program(cx);
        self.print("Copied the console's definitions to the editor.");
    }

    /// Puts new console lines into the text area, scrolled to the newest.
    fn flush_console(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !std::mem::take(&mut self.console_dirty) {
            return;
        }
        let text = self.console.iter().map(|l| l.as_ref()).collect::<Vec<_>>().join("\n");
        let lines = self.console.len();
        self.console_output.update(cx, |output, cx| {
            output.set_value(text, window, cx);
            if let (Some(rows), Some(line_height)) = (output.visible_row_range(), output.line_height()) {
                let top = lines.saturating_sub(rows.len()) as f32;
                output.set_scroll_offset(point(px(0.), -line_height * top), cx);
            }
        });
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
        position(&self.run_text, offset)
    }

    /// Shows the program's parse errors as it's typed, without running it.
    pub(crate) fn check_program(&mut self, cx: &mut Context<Self>) {
        // The last run's highlight no longer points at the right text.
        self.highlight.set(vec![], cx);
        let text = self.editor.read(cx).value().to_string();
        let diagnostics = pendown_logo::syntax::check(&text)
            .into_iter()
            .filter_map(|e| {
                let span = e.span?;
                let range = position(&text, span.start as usize)..position(&text, span.end as usize);
                Some(Diagnostic::new(range, e.message).with_severity(DiagnosticSeverity::Error))
            })
            .collect();
        self.set_diagnostics(diagnostics, cx);
    }

    /// Shows errors as diagnostics on the code, and in the console with
    /// their call chains (ADR 0005).
    /// An error in a console entry shows only in the console; one in the
    /// editor's program also shows on the code.
    fn show_errors(&mut self, errors: Vec<pendown_logo::Error>, cx: &mut Context<Self>) {
        let mut diagnostics = Vec::new();
        for e in &errors {
            let where_ = match e.span {
                Some(s) if s.source == SourceId::Editor => format!("Line {}: ", s.line + 1),
                Some(_) => "In the console: ".to_string(),
                None => String::new(),
            };
            self.print(format!("{where_}{}", e.message));
            for step in errors::chain(e) {
                self.print(format!("    {step}"));
            }
            if let Some(span) = e.span
                && span.source == SourceId::Editor
            {
                let range = self.position(span.start as usize)..self.position(span.end as usize);
                diagnostics.push(Diagnostic::new(range, e.message.clone()).with_severity(DiagnosticSeverity::Error));
            }
        }
        if !diagnostics.is_empty() {
            self.set_diagnostics(diagnostics, cx);
        }
    }

    /// Moves the highlight to the running instruction, scrolling the editor
    /// to keep it in view (spike 01).
    fn show_highlight(&mut self, cx: &mut Context<Self>) {
        // Console entries aren't in the editor, so they aren't highlighted there.
        let Some(span) = self.animator.as_ref().and_then(|a| a.highlight).filter(|s| s.source == SourceId::Editor) else {
            self.highlight.set(vec![], cx);
            return;
        };
        let color = cx.theme().warning.opacity(0.35);
        self.highlight.set(vec![RangeDecoration::new(span.range()).with_style(RangeDecorationStyle::Fill).with_color(color)], cx);
        let line = span.line as usize;
        let lines = self.run_text.matches('\n').count() + 1;
        self.editor.update(cx, |state, cx| {
            let (Some(rows), Some(line_height)) = (state.visible_row_range(), state.line_height()) else { return };
            // Only scroll where there's more text to see, and never past the
            // end: the editor springs back from there, animating for a third
            // of a second each time (issue 08).
            let near_top = line < rows.start + 2 && rows.start > 0;
            let near_bottom = line + 2 >= rows.end && rows.end < lines;
            let top = line.saturating_sub(3).min(lines.saturating_sub(rows.len()));
            if (near_top || near_bottom) && top != rows.start {
                let offset = state.scroll_offset();
                state.set_scroll_offset(point(offset.x, -line_height * top as f32), cx);
            }
        });
    }

    /// One animation frame: play events, hand finished moves to the canvas,
    /// and notice when the run has ended.
    fn tick(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
            let per_segment = self.canvas.borrow().drawing.per_segment.max(Duration::from_nanos(200));
            let max_marks = (INSTANT_DRAWING.as_nanos() / per_segment.as_nanos()).clamp(500, 100_000) as usize;
            let frame = animator.frame(dt, max_marks);
            self.events += frame.events;
            if frame.watch_changed {
                let watch = animator.watch.clone();
                self.show_watch(cx, |watched| watched.set(watch));
                // The panes were notified mid-draw, so they redraw in the
                // next frame: make sure there is one, even if paused.
                window.request_animation_frame();
            }

            let mut canvas = self.canvas.borrow_mut();
            if frame.cleaned {
                canvas.drawing.clean();
            }
            if let Some(color) = frame.background {
                canvas.drawing.set_background(color);
            }
            canvas.drawing.push(&frame.marks);
            drop(canvas);
            for text in frame.printed {
                for line in text.lines() {
                    self.print(line.to_string());
                }
            }
            if let Some(speed) = frame.speed {
                // The animator has changed already: this moves the slider.
                self.set_speed(speed, window, cx);
            }
            self.on_frame(now);
            if frame.highlight_changed {
                self.on_highlight();
                self.show_highlight(cx);
            }
        }
        let animator = self.animator.as_ref().expect("checked above");
        if animator.finished() && self.finished_at.is_none() && !animator.stopped {
            self.finished_at = Some(now);
            self.report();
            let defining = self.defining.take();
            match self.run.as_ref().and_then(|r| r.result()) {
                Some(Err(RunError::Parse(errors))) => self.show_errors(errors, cx),
                Some(Err(RunError::Runtime(e))) => self.show_errors(vec![e], cx),
                Some(Ok(())) => {
                    if let Some(definition) = defining {
                        self.console_definitions.push(definition);
                        self.definitions_cleared = false;
                    }
                }
                _ => {}
            }
        }
    }

    fn status(&self) -> String {
        let Some(animator) = &self.animator else { return String::new() };
        let state = if animator.stopped {
            "Stopped"
        } else if animator.finished() {
            "Finished"
        } else {
            match animator.mode() {
                Mode::Paused => "Paused",
                Mode::Running => "Running",
                _ => "Stepping",
            }
        };
        let run = self.run.as_ref().expect("a run goes with an animator");
        let elapsed = self.finished_at.or(self.stop_at).unwrap_or_else(Instant::now) - run.started;
        format!("{state} · {:.1} s · {} lines drawn", elapsed.as_secs_f64(), self.canvas.borrow().drawing.segments())
    }

    /// Step, or Step instruction.
    pub(crate) fn step(&mut self, instruction: bool, cx: &mut Context<Self>) {
        if let Some(a) = &mut self.animator {
            if instruction { a.step_instruction() } else { a.step_line() }
        }
        cx.notify();
    }

    pub(crate) fn toggle_pause(&mut self, cx: &mut Context<Self>) {
        if let Some(a) = &mut self.animator {
            if a.mode() == Mode::Paused { a.resume() } else { a.pause() }
        }
        cx.notify();
    }
}

impl Render for App {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let metrics = self.metrics.clone();
        let started = metrics.renders.start();
        let element = self.render_window(window, cx);
        metrics.renders.stop(started);
        element
    }
}

impl App {
    fn render_window(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        if !std::mem::replace(&mut self.checked, true) {
            self.check_program(cx);
        }
        self.tick(window, cx);
        self.update_overlay();
        self.flush_console(window, cx);
        self.update_title(window, cx);
        self.schedule_frame(window, cx);

        let theme = cx.theme().clone();
        let running = self.running();
        self.editor_pane.update(cx, |pane, cx| pane.set_readonly(running, cx));
        self.console_pane.update(cx, |pane, cx| pane.set_disabled(running, cx));
        let paused = self.animator.as_ref().is_some_and(|a| a.mode() == Mode::Paused && !a.finished());
        let canvas = self.canvas.clone();
        let overlay = match &self.animator {
            Some(a) => {
                // A fifth of a pixel, as the buffer is drawn.
                let tolerance = 0.2 / self.canvas.borrow().drawing.scale().max(1e-6);
                Overlay { partial: a.partial(tolerance), turtle: a.shown.then_some((a.pos, a.heading)) }
            }
            None => Overlay { partial: None, turtle: Some((pendown_logo::Point { x: 0.0, y: 0.0 }, 0.0)) },
        };

        let toolbar = ToolbarState { running, paused, speed: self.speed };
        self.toolbar_pane.update(cx, |pane, cx| pane.set(toolbar, cx));

        v_flex()
            .size_full()
            .on_action(cx.listener(|this, _: &NewFile, window, cx| this.then(After::NewFile, window, cx)))
            .on_action(cx.listener(|this, _: &OpenFile, window, cx| this.then(After::OpenFile, window, cx)))
            .on_action(cx.listener(|this, _: &SaveFile, window, cx| this.save(After::Nothing, window, cx)))
            .on_action(cx.listener(|this, _: &SaveFileAs, window, cx| this.save_as(After::Nothing, window, cx)))
            .on_action(cx.listener(|this, _: &ExportPicture, window, cx| this.export(window, cx)))
            .on_action(cx.listener(|this, _: &RunProgram, _, cx| this.start(DEFAULT_CHANNEL, cx)))
            .on_action(cx.listener(|this, _: &ToggleMetrics, window, cx| this.toggle_metrics(window, cx)))
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(
                h_flex()
                    .h(px(TOOLBAR_HEIGHT))
                    .px_2()
                    .gap_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(self.toolbar_pane.clone().cached(StyleRefinement::default().h_full().flex_1()))
                    // A fixed width, so the toolbar's size doesn't change
                    // with the status, which would redraw it.
                    .child(
                        div()
                            .flex_none()
                            .w(px(STATUS_WIDTH))
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(self.status()),
                    ),
            )
            .child(
                div().flex_1().min_h_0().child(
                    h_resizable("main")
                        .child(
                            resizable_panel()
                                .size(px(460.))
                                .child(self.editor_pane.clone().cached(StyleRefinement::default().size_full())),
                        )
                        .child(
                            resizable_panel().child(
                                div()
                                    .relative()
                                    .size_full()
                                    .child(
                                        gpui_kit::canvas(
                                            move |bounds, _, _| bounds,
                                            move |_, bounds, window, _| canvas.borrow_mut().paint(bounds, window, overlay.clone()),
                                        )
                                        .size_full(),
                                    )
                                    .when_some(self.overlay.as_ref(), |panel, metrics| {
                                        panel.child(
                                            v_flex()
                                                .absolute()
                                                .top_2()
                                                .left_2()
                                                .p_2()
                                                .rounded_md()
                                                .bg(hsla(0.0, 0.0, 0.0, 0.7))
                                                .text_color(hsla(0.0, 0.0, 1.0, 0.9))
                                                .font_family(theme.mono_font_family.clone())
                                                .text_xs()
                                                .children(metrics.lines.iter().map(|line| div().child(line.clone()))),
                                        )
                                    }),
                            ),
                        ),
                ),
            )
            .child(
                div().h(px(CONSOLE_HEIGHT)).border_t_1().border_color(theme.border).child(
                    h_resizable("console-area")
                        .child(resizable_panel().child(self.render_console(running, cx)))
                        .child(
                            resizable_panel()
                                .size(px(220.))
                                .child(self.counters_pane.clone().cached(StyleRefinement::default().size_full())),
                        )
                        .child(
                            resizable_panel()
                                .size(px(340.))
                                .child(self.variables_pane.clone().cached(StyleRefinement::default().size_full())),
                        ),
                ),
            )
    }

    /// The console's title, buttons, output and input.
    fn render_console(&mut self, running: bool, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let theme = cx.theme().clone();
        v_flex()
            .size_full()
            .child(
                h_flex()
                    .px_2()
                    .py_1()
                    .gap_2()
                    .child(div().text_sm().text_color(theme.muted_foreground).child("Console"))
                    .child(div().flex_1())
                    .when(!self.console_definitions.is_empty(), |row| {
                        row.child(
                            Button::new("copy-definitions")
                                .xsmall()
                                .ghost()
                                .label("Copy to editor")
                                .disabled(running)
                                .on_click(cx.listener(|this, _, window, cx| this.copy_definitions(window, cx))),
                        )
                    })
                    .child(Button::new("clear-console").xsmall().ghost().label("Clear").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.console.clear();
                            this.console_dirty = true;
                            cx.notify();
                        },
                    ))),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .capture_action(cx.listener(|this, _: &MoveUp, window, cx| {
                        this.recall(true, window, cx);
                        cx.stop_propagation();
                    }))
                    .capture_action(cx.listener(|this, _: &MoveDown, window, cx| {
                        this.recall(false, window, cx);
                        cx.stop_propagation();
                    }))
                    .child(self.console_pane.clone().cached(StyleRefinement::default().size_full())),
            )
    }
}
