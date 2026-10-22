//! The editor, the console and the toolbar as views of their own, which the
//! window draws cached (phase 3 issue 08). A frame only rebuilds what changed: gpui reuses
//! a cached view until it, or an entity it read, is notified. Without this,
//! every animation frame laid out and painted the whole window, about 4 ms
//! of the UI thread's time at 60 frames a second.

use std::rc::Rc;

use gpui_kit::component::{
    ActiveTheme, Disableable as _, Sizable,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Editor, EditorState, Input, InputState, Textarea, TextareaState},
    slider::{Slider, SliderState},
};
use gpui_kit::*;
use pendown::{animator::Speed, metrics::WindowMetrics, run::DEFAULT_CHANNEL, speed::describe};

use crate::{app::App, files::After};

pub struct EditorPane {
    pub editor: Entity<EditorState>,
    readonly: bool,
    metrics: Rc<WindowMetrics>,
}

impl EditorPane {
    pub fn new(editor: Entity<EditorState>, metrics: Rc<WindowMetrics>) -> EditorPane {
        EditorPane { editor, readonly: false, metrics }
    }

    /// The editor is read-only while a program runs.
    pub fn set_readonly(&mut self, readonly: bool, cx: &mut Context<Self>) {
        if self.readonly != readonly {
            self.readonly = readonly;
            cx.notify();
        }
    }
}

impl Render for EditorPane {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.metrics.editor_renders.add(1);
        let theme = cx.theme();
        Editor::new(&self.editor)
            .bordered(false)
            .readonly(self.readonly)
            .h_full()
            .font_family(theme.mono_font_family.clone())
            .text_size(theme.mono_font_size)
    }
}

/// The console's output and its input line.
pub struct ConsolePane {
    pub output: Entity<TextareaState>,
    pub input: Entity<InputState>,
    disabled: bool,
    metrics: Rc<WindowMetrics>,
}

impl ConsolePane {
    pub fn new(output: Entity<TextareaState>, input: Entity<InputState>, metrics: Rc<WindowMetrics>) -> ConsolePane {
        ConsolePane { output, input, disabled: false, metrics }
    }

    /// The input line is disabled while a program runs.
    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        if self.disabled != disabled {
            self.disabled = disabled;
            cx.notify();
        }
    }
}

impl Render for ConsolePane {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.metrics.console_renders.add(1);
        let theme = cx.theme();
        gpui_kit::component::v_flex()
            .size_full()
            .child(
                div().flex_1().min_h_0().px_1().child(
                    Textarea::new(&self.output)
                        .readonly(true)
                        .bordered(false)
                        .h_full()
                        .font_family(theme.mono_font_family.clone())
                        .text_xs(),
                ),
            )
            .child(
                div()
                    .p_1()
                    .child(Input::new(&self.input).small().disabled(self.disabled).font_family(theme.mono_font_family.clone())),
            )
    }
}

/// What the toolbar shows, which changes only when a run starts, pauses or
/// ends, or the speed changes.
#[derive(Clone, Copy, PartialEq)]
pub struct ToolbarState {
    pub running: bool,
    pub paused: bool,
    pub speed: Speed,
}

/// The toolbar's buttons, slider and speed. The run's status, which changes
/// as it draws, is beside it in the window instead.
pub struct ToolbarPane {
    app: WeakEntity<App>,
    slider: Entity<SliderState>,
    state: ToolbarState,
}

impl ToolbarPane {
    pub fn new(app: WeakEntity<App>, slider: Entity<SliderState>, state: ToolbarState) -> ToolbarPane {
        ToolbarPane { app, slider, state }
    }

    pub fn set(&mut self, state: ToolbarState, cx: &mut Context<Self>) {
        if self.state != state {
            self.state = state;
            cx.notify();
        }
    }

    /// A button that calls `f` on the window's `App`.
    fn button(
        &self,
        id: &'static str,
        label: &'static str,
        f: impl Fn(&mut App, &mut Window, &mut Context<App>) + 'static,
    ) -> Button {
        let app = self.app.clone();
        Button::new(id).small().label(label).on_click(move |_, window, cx| {
            app.update(cx, |this, cx| f(this, window, cx)).ok();
        })
    }
}

impl Render for ToolbarPane {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let ToolbarState { running, paused, speed } = self.state;
        h_flex()
            .gap_2()
            .size_full()
            .child(self.button("new", "New", |this, window, cx| this.then(After::NewFile, window, cx)).ghost())
            .child(self.button("open", "Open", |this, window, cx| this.then(After::OpenFile, window, cx)).ghost())
            .child(self.button("save", "Save", |this, window, cx| this.save(After::Nothing, window, cx)).ghost())
            .child(self.button("export", "Export PNG", |this, window, cx| this.export(window, cx)).ghost())
            .child(div().w(px(12.)))
            .child(self.button("run", "Run", |this, _, cx| this.start(DEFAULT_CHANNEL, cx)).primary())
            .child(
                self.button("pause", if paused { "Resume" } else { "Pause" }, |this, _, cx| this.toggle_pause(cx))
                    .disabled(!running),
            )
            .child(self.button("step", "Step", |this, _, cx| this.step(false, cx)).disabled(!paused))
            .child(
                self.button("step-instruction", "Step instruction", |this, _, cx| this.step(true, cx)).disabled(!paused),
            )
            .child(self.button("stop", "Stop", |this, _, cx| this.stop(cx)).danger().disabled(!running))
            .child(div().w(px(16.)))
            .child(div().w(px(160.)).child(Slider::new(&self.slider)))
            .child(div().w(px(10.)))
            .child(div().text_sm().w(px(180.)).child(describe(speed)))
    }
}
