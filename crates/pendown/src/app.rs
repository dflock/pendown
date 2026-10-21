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
    input::{Editor, EditorState, InputHighlighter, Position, RangeDecoration, RangeDecorationCollection, RangeDecorationStyle},
    resizable::{h_resizable, resizable_panel},
    slider::{Slider, SliderEvent, SliderState},
    v_flex,
};
use gpui_kit::*;
use pendown::{
    animator::{Animator, Mode, Speed},
    errors,
    run::{self, Channel, DEFAULT_CHANNEL, Run},
    speed::{INSTANT_LEVEL, describe, level_speed, speed_level},
};
use pendown_logo::RunError;

use crate::{
    autotest::Autotest,
    canvas::{Canvas, Overlay},
    highlighter::LogoHighlighter,
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
    pub(crate) console: Vec<SharedString>,
    last_frame: Option<Instant>,
    pub(crate) frame_intervals: Samples,
    pub(crate) events: usize,
    pub(crate) autotest: Option<Autotest>,
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
        let speed = level_speed(5);
        let speed_slider = cx.new(|_| {
            SliderState::new().min(1.0).max(INSTANT_LEVEL as f32).step(1.0).default_value(speed_level(speed) as f32)
        });
        let subscriptions = vec![cx.subscribe_in(&speed_slider, window, |this, _, event: &SliderEvent, window, cx| {
            if let SliderEvent::Change(value) = event {
                this.set_speed(level_speed(value.end().round() as u8), window, cx);
            }
        })];
        App {
            editor,
            highlight,
            speed_slider,
            speed,
            canvas: Rc::new(RefCell::new(Canvas::new())),
            animator: None,
            run: None,
            run_text: String::new(),
            finished_at: None,
            stop_at: None,
            console: Vec::new(),
            last_frame: None,
            frame_intervals: Samples::default(),
            events: 0,
            autotest: Autotest::from_env(),
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
        self.set_diagnostics(Vec::new(), cx);
        let (animator, run) = run::start(text, self.speed, channel);
        self.animator = Some(animator);
        self.run = Some(run);
        self.finished_at = None;
        self.stop_at = None;
        self.last_frame = None;
        self.frame_intervals = Samples::default();
        self.events = 0;
        cx.notify();
    }

    pub(crate) fn stop(&mut self, cx: &mut Context<Self>) {
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

    /// Shows errors as diagnostics on the code, and in the console with
    /// their call chains (ADR 0005).
    fn show_errors(&mut self, errors: Vec<pendown_logo::Error>, cx: &mut Context<Self>) {
        let mut diagnostics = Vec::new();
        for e in &errors {
            let line = e.span.map_or(String::new(), |s| format!("Line {}: ", s.line + 1));
            self.console.push(format!("{line}{}", e.message).into());
            for step in errors::chain(e) {
                self.console.push(format!("    {step}").into());
            }
            if let Some(span) = e.span {
                let range = self.position(span.start as usize)..self.position(span.end as usize);
                diagnostics.push(Diagnostic::new(range, e.message.clone()).with_severity(DiagnosticSeverity::Error));
            }
        }
        self.set_diagnostics(diagnostics, cx);
    }

    /// Moves the highlight to the running instruction, scrolling the editor
    /// to keep it in view (spike 01).
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
                self.console.extend(text.lines().map(|line| SharedString::from(line.to_string())));
            }
            if let Some(speed) = frame.speed {
                // The animator has changed already: this moves the slider.
                self.set_speed(speed, window, cx);
            }
            if frame.highlight_changed {
                self.on_highlight();
                self.show_highlight(cx);
            }
        }
        let animator = self.animator.as_ref().expect("checked above");
        if animator.finished() && self.finished_at.is_none() && !animator.stopped {
            self.finished_at = Some(now);
            match self.run.as_ref().and_then(|r| r.result()) {
                Some(Err(RunError::Parse(errors))) => self.show_errors(errors, cx),
                Some(Err(RunError::Runtime(e))) => self.show_errors(vec![e], cx),
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

    fn toggle_pause(&mut self, cx: &mut Context<Self>) {
        if let Some(a) = &mut self.animator {
            if a.mode() == Mode::Paused { a.resume() } else { a.pause() }
        }
        cx.notify();
    }
}

impl Render for App {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.tick(window, cx);
        self.drive_autotest(window, cx);
        if self.animator.as_ref().is_some_and(|a| a.wants_frames()) || self.autotest.is_some() {
            window.request_animation_frame();
        }

        let theme = cx.theme().clone();
        let running = self.running();
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

        let toolbar = h_flex()
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
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_pause(cx))),
            )
            .child(Button::new("step").small().label("Step").disabled(!paused).on_click(cx.listener(|this, _, _, cx| {
                if let Some(a) = &mut this.animator {
                    a.step_line();
                }
                cx.notify();
            })))
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
            .child(div().w(px(16.)))
            .child(div().w(px(160.)).child(Slider::new(&self.speed_slider)))
            .child(div().w(px(10.)))
            .child(div().text_sm().w(px(180.)).child(describe(self.speed)))
            .child(div().text_sm().text_color(theme.muted_foreground).child(self.status()));

        v_flex()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(toolbar)
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
                            gpui_kit::canvas(
                                move |bounds, _, _| bounds,
                                move |_, bounds, window, _| canvas.borrow_mut().paint(bounds, window, overlay.clone()),
                            )
                            .size_full(),
                        )),
                ),
            )
            .child(
                div()
                    .id("console")
                    .h(px(120.))
                    .p_2()
                    .border_t_1()
                    .border_color(theme.border)
                    .overflow_y_scroll()
                    .font_family(theme.mono_font_family.clone())
                    .text_xs()
                    .children(self.console.iter().skip(self.console.len().saturating_sub(200)).cloned().map(|l| div().child(l))),
            )
    }
}
