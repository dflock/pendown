//! The editor, the console and the toolbar as views of their own, which the
//! window draws cached (phase 3 issue 08). A frame only rebuilds what changed: gpui reuses
//! a cached view until it, or an entity it read, is notified. Without this,
//! every animation frame laid out and painted the whole window, about 4 ms
//! of the UI thread's time at 60 frames a second.

use std::rc::Rc;

use gpui_kit::component::{
    ActiveTheme, Disableable as _, Selectable as _, Sizable,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Editor, EditorState, Input, InputState, Textarea, TextareaState},
    menu::{DropdownMenu as _, PopupMenuItem},
    slider::{Slider, SliderState},
};
use gpui_kit::{assets::IconName, *};
use pendown::{animator::Speed, metrics::WindowMetrics, run::DEFAULT_CHANNEL, speed::describe};

use crate::{
    app::App,
    files::{After, Picture},
};

pub struct EditorPane {
    pub editor: Entity<EditorState>,
    readonly: bool,
    metrics: Rc<WindowMetrics>,
}

impl EditorPane {
    pub fn new(editor: Entity<EditorState>, metrics: Rc<WindowMetrics>) -> EditorPane {
        EditorPane {
            editor,
            readonly: false,
            metrics,
        }
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
    pub fn new(
        output: Entity<TextareaState>,
        input: Entity<InputState>,
        metrics: Rc<WindowMetrics>,
    ) -> ConsolePane {
        ConsolePane {
            output,
            input,
            disabled: false,
            metrics,
        }
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
                div().p_1().child(
                    Input::new(&self.input)
                        .small()
                        .disabled(self.disabled)
                        .font_family(theme.mono_font_family.clone()),
                ),
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
    /// The side panel is open.
    pub help: bool,
    /// Show Kame thinking is on.
    pub thinking: bool,
    /// Drive Kame is on.
    pub driving: bool,
}

/// The toolbar's buttons, slider and speed. The run's status, which changes
/// as it draws, is beside it in the window instead.
pub struct ToolbarPane {
    app: WeakEntity<App>,
    slider: Entity<SliderState>,
    state: ToolbarState,
}

impl ToolbarPane {
    pub fn new(
        app: WeakEntity<App>,
        slider: Entity<SliderState>,
        state: ToolbarState,
    ) -> ToolbarPane {
        ToolbarPane { app, slider, state }
    }

    pub fn set(&mut self, state: ToolbarState, cx: &mut Context<Self>) {
        if self.state != state {
            self.state = state;
            cx.notify();
        }
    }

    /// A button with only an icon, named by its tooltip, that calls `f` on
    /// the window's `App`.
    fn icon_button(
        &self,
        id: &'static str,
        icon: IconName,
        tooltip: &'static str,
        f: impl Fn(&mut App, &mut Window, &mut Context<App>) + 'static,
    ) -> Button {
        let app = self.app.clone();
        Button::new(id)
            .flex_none()
            .small()
            .icon(icon)
            .tooltip(tooltip)
            .on_click(move |_, window, cx| {
                app.update(cx, |this, cx| f(this, window, cx)).ok();
            })
    }

    /// A button that calls `f` on the window's `App`.
    fn button(
        &self,
        id: &'static str,
        icon: IconName,
        label: &'static str,
        f: impl Fn(&mut App, &mut Window, &mut Context<App>) + 'static,
    ) -> Button {
        let app = self.app.clone();
        Button::new(id)
            .flex_none()
            .small()
            .icon(icon)
            .label(label)
            .on_click(move |_, window, cx| {
                app.update(cx, |this, cx| f(this, window, cx)).ok();
            })
    }
}

impl ToolbarPane {
    /// Export, with a menu of the formats.
    fn export_menu(&self) -> impl IntoElement {
        let app = self.app.clone();
        let item = move |label: &'static str, picture: Picture| {
            let app = app.clone();
            PopupMenuItem::new(label).on_click(move |_, window, cx| {
                app.update(cx, |this, cx| this.export(picture, window, cx))
                    .ok();
            })
        };
        Button::new("export")
            .flex_none()
            .small()
            .ghost()
            .icon(IconName::ImageDown)
            .tooltip("Export")
            .dropdown_caret(true)
            .dropdown_menu(move |menu, _, _| {
                menu.item(item("PNG picture", Picture::Png))
                    .item(item("SVG drawing", Picture::Svg))
            })
    }
}

impl Render for ToolbarPane {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let ToolbarState {
            running,
            paused,
            speed,
            help,
            thinking,
            driving,
        } = self.state;
        h_flex()
            .gap_2()
            .size_full()
            .overflow_hidden()
            .child(
                self.icon_button("new", IconName::FilePlus, "New", |this, window, cx| {
                    this.then(After::NewFile, window, cx)
                })
                .ghost(),
            )
            .child(
                self.icon_button("open", IconName::FolderOpen, "Open", |this, window, cx| {
                    this.then(After::OpenFile, window, cx)
                })
                .ghost(),
            )
            .child(
                self.icon_button("save", IconName::Save, "Save", |this, window, cx| {
                    this.save(After::Nothing, window, cx)
                })
                .ghost(),
            )
            .child(self.export_menu())
            .child(div().flex_none().w(px(12.)))
            .child(
                self.button("run", IconName::Play, "Run", |this, _, cx| {
                    this.start(DEFAULT_CHANNEL, cx)
                })
                .primary(),
            )
            .child(
                self.button(
                    "pause",
                    if paused {
                        IconName::Play
                    } else {
                        IconName::Pause
                    },
                    if paused { "Resume" } else { "Pause" },
                    |this, _, cx| this.toggle_pause(cx),
                )
                // As wide as Resume, so the swap doesn't move what's after it.
                .w(px(88.))
                .disabled(!running),
            )
            .child(
                self.button("step", IconName::StepForward, "Step line", |this, _, cx| {
                    this.step(false, cx)
                })
                .disabled(!paused),
            )
            .child(
                self.button(
                    "step-instruction",
                    IconName::Footprints,
                    "Step instruction",
                    |this, _, cx| this.step(true, cx),
                )
                .disabled(!paused),
            )
            .child(
                self.button("stop", IconName::Square, "Stop", |this, _, cx| {
                    this.stop(cx)
                })
                .danger()
                .disabled(!running),
            )
            // Nothing shrinks, so the slider keeps its place and size as
            // the speed's description changes length.
            .child(div().flex_none().w(px(2.)))
            .child(
                div()
                    .flex_none()
                    .w(px(160.))
                    .child(Slider::new(&self.slider)),
            )
            // Room for the knob, which overhangs the end of the track.
            .child(div().flex_none().w(px(6.)))
            .child(
                div()
                    .flex_none()
                    .w(px(150.))
                    .text_sm()
                    .whitespace_nowrap()
                    .child(describe(speed)),
            )
            .child(div().flex_none().w(px(6.)))
            // The arrow keys move and turn Kame, while nothing else runs.
            .child(
                self.button("drive", IconName::Gamepad2, "Drive", |this, window, cx| {
                    this.toggle_driving(window, cx)
                })
                .selected(driving)
                .disabled(running && !driving),
            )
            // Arcs on turns and rulers on moves, over the canvas.
            .child(
                self.button(
                    "thinking",
                    IconName::Lightbulb,
                    "Thinking",
                    |this, _, cx| this.toggle_thinking(cx),
                )
                .selected(thinking),
            )
            .child(div().flex_1())
            // Shows and hides the side panel; F1 opens it on the word under
            // the cursor (phase 5).
            .child(
                self.button("help", IconName::BookOpen, "Help", |this, _, cx| {
                    this.toggle_help(cx)
                })
                .selected(help),
            )
    }
}
