//! Teach Kame: a dialog that turns entries typed at the console into a
//! procedure, puts it in the editor and defines it at the console.

use std::sync::Arc;

use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::{
    ActiveTheme, Disableable as _, Sizable as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};
use gpui_kit::{assets::IconName, prelude::FluentBuilder as _, *};
use pendown::{
    console::{Ending, Entry, defines},
    help::picture_options,
    teach::{check_name, picture_program, procedure, uses},
};

use crate::{
    app::App as PendownWindow,
    help_panel::{Picture, Rendered, code_block, kame, svg_image},
};

/// The dialog's width, and its picture's size, in logical pixels.
const WIDTH: f32 = 760.;
const PICTURE: f32 = 170.;

/// An entry in the list.
struct Row {
    text: String,
    /// Why it can't be ticked, if it can't.
    fixed: Option<&'static str>,
    /// Shown under it: its error, or that it was stopped.
    note: Option<String>,
    /// Procedures it calls that are defined only at the console.
    uses: Vec<String>,
    ticked: bool,
}

pub struct TeachKame {
    app: WeakEntity<PendownWindow>,
    rows: Vec<Row>,
    /// The row clicked last, for Shift-click.
    anchor: Option<usize>,
    pub(crate) name: Entity<InputState>,
    in_program: Vec<String>,
    at_console: Vec<String>,
    /// The procedures the picture can use.
    definitions: String,
    picture: Option<Picture>,
    /// What the picture is of, or is being drawn of: the newest wins.
    picture_of: String,
    _subscriptions: Vec<Subscription>,
}

impl TeachKame {
    /// `in_program` and `at_console` are the procedures defined there, and
    /// `definitions` their text, which the picture uses.
    pub fn new(
        app: WeakEntity<PendownWindow>,
        entries: Vec<Entry>,
        in_program: Vec<String>,
        at_console: Vec<String>,
        definitions: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> TeachKame {
        let console_only: Vec<String> = at_console
            .iter()
            .filter(|n| !in_program.contains(n))
            .cloned()
            .collect();
        let rows = entries
            .into_iter()
            .map(|e| {
                let fixed = defines(&e.text)
                    .then_some("A definition can't go inside another. Use Copy to editor for it.");
                let note = match &e.ending {
                    Some(Ending::Error(message)) => Some(message.clone()),
                    Some(Ending::Stopped) => Some("Stopped".to_string()),
                    _ => None,
                };
                // A definition doesn't use the procedure it defines.
                let uses = match fixed {
                    Some(_) => Vec::new(),
                    None => uses(&e.text, &console_only)
                        .into_iter()
                        .map(String::from)
                        .collect(),
                };
                Row {
                    uses,
                    ticked: fixed.is_none() && e.ending == Some(Ending::Ran),
                    text: e.text,
                    fixed,
                    note,
                }
            })
            .collect();
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("box, star, house…"));
        let subscriptions = vec![cx.subscribe_in(&name, window, |_, _, event, _, cx| {
            if let InputEvent::Change = event {
                cx.notify();
            }
        })];
        let mut dialog = TeachKame {
            app,
            rows,
            anchor: None,
            name,
            in_program,
            at_console,
            definitions,
            picture: None,
            picture_of: String::new(),
            _subscriptions: subscriptions,
        };
        dialog.draw_picture(cx);
        dialog
    }

    fn ticked(&self) -> Vec<&str> {
        self.rows
            .iter()
            .filter(|r| r.ticked)
            .map(|r| r.text.as_str())
            .collect()
    }

    /// Ticks or unticks a row, or with Shift, every row from the one clicked
    /// last to this one.
    pub(crate) fn toggle(&mut self, i: usize, shift: bool, cx: &mut Context<Self>) {
        let ticked = !self.rows[i].ticked;
        let range = match self.anchor {
            Some(a) if shift => a.min(i)..=a.max(i),
            _ => i..=i,
        };
        for row in &mut self.rows[range] {
            if row.fixed.is_none() {
                row.ticked = ticked;
            }
        }
        self.anchor = Some(i);
        self.draw_picture(cx);
        cx.notify();
    }

    pub(crate) fn tick_all(&mut self, ticked: bool, cx: &mut Context<Self>) {
        for row in &mut self.rows {
            if row.fixed.is_none() {
                row.ticked = ticked;
            }
        }
        self.draw_picture(cx);
        cx.notify();
    }

    /// Draws what the ticked entries draw, on a background thread, as Help
    /// draws its examples.
    fn draw_picture(&mut self, cx: &mut Context<Self>) {
        let program = picture_program(&self.definitions, &self.ticked());
        if program == self.picture_of {
            return;
        }
        self.picture_of = program.clone();
        self.picture = None;
        let task = cx.background_spawn({
            let program = program.clone();
            async move { pendown_cli::render(&program, picture_options()) }
        });
        cx.spawn(async move |this, cx| {
            let output = task.await;
            this.update(cx, |this, cx| {
                if this.picture_of != program {
                    return;
                }
                this.picture = Some(match output.svg {
                    Some(svg) if output.drawn > 0 => {
                        Picture::Drawing(Arc::from(svg.into_bytes()), Rendered::default())
                    }
                    _ => Picture::Printed(output.printed),
                });
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// For the scripted session: each row as `x` for ticked, `-` for not,
    /// or `#` for one that can't be, then its text and any notes in
    /// brackets, separated by ` | `.
    pub(crate) fn describe(&self) -> String {
        self.rows
            .iter()
            .map(|r| {
                let mark = match (r.fixed, r.ticked) {
                    (Some(_), _) => '#',
                    (None, true) => 'x',
                    (None, false) => '-',
                };
                let notes: Vec<String> = r
                    .note
                    .iter()
                    .cloned()
                    .chain(r.uses.iter().map(|n| format!("uses {n}")))
                    .collect();
                let notes = if notes.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", notes.join(", "))
                };
                format!("{mark} {}{notes}", r.text.replace('\n', " "))
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }

    /// For the scripted session: the picture has been drawn.
    pub(crate) fn has_drawing(&self) -> bool {
        matches!(self.picture, Some(Picture::Drawing(..)))
    }

    /// For the scripted session: why the name can't be used, if it can't.
    pub(crate) fn name_problem(&self, cx: &App) -> Option<String> {
        self.checked_name(cx).err().flatten()
    }

    /// The name, in capitals, or why it can't be used.
    fn checked_name(&self, cx: &App) -> Result<String, Option<String>> {
        check_name(
            &self.name.read(cx).value(),
            &self.in_program,
            &self.at_console,
        )
    }

    /// Teach Kame: the procedure goes in the editor and is defined at the
    /// console, and the dialog closes.
    pub(crate) fn teach(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Ok(name) = self.checked_name(cx) else {
            return;
        };
        let ticked = self.ticked();
        if ticked.is_empty() {
            return;
        }
        let text = procedure(&name, &ticked);
        self.app
            .update(cx, |app, cx| app.teach(&name, text, window, cx))
            .ok();
        window.close_dialog(cx);
    }

    fn render_row(&self, i: usize, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let theme = cx.theme();
        let row = &self.rows[i];
        let mut notes = Vec::new();
        if let Some(why) = row.fixed {
            notes.push((why.to_string(), theme.muted_foreground));
        }
        if let Some(note) = &row.note {
            notes.push((note.clone(), theme.danger));
        }
        for name in &row.uses {
            notes.push((
                format!("Uses {name}, so {name} has to be in your program too"),
                theme.muted_foreground,
            ));
        }
        let error = row.note.is_some() && row.fixed.is_none();
        h_flex()
            .items_start()
            .gap_2()
            .px_2()
            .py_1()
            .rounded_md()
            .when(error, |el| el.bg(theme.danger.opacity(0.08)))
            .child(
                Checkbox::new(("teach-row", i))
                    .checked(row.ticked)
                    .disabled(row.fixed.is_some())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.toggle(i, window.modifiers().shift, cx)
                    })),
            )
            .child(
                v_flex()
                    .min_w_0()
                    .child(
                        div()
                            .font_family(theme.mono_font_family.clone())
                            .text_xs()
                            .when(row.fixed.is_some(), |el| {
                                el.text_color(theme.muted_foreground)
                            })
                            .child(row.text.replace('\n', " ")),
                    )
                    .children(
                        notes
                            .into_iter()
                            .map(|(text, color)| div().text_xs().text_color(color).child(text)),
                    ),
            )
    }

    fn render_picture(&self, cx: &App) -> AnyElement {
        let size = px(PICTURE);
        match &self.picture {
            Some(Picture::Drawing(svg, rendered)) => {
                svg_image(svg.clone(), rendered.clone(), PICTURE, PICTURE)
                    .rounded_md()
                    .into_any_element()
            }
            _ => div()
                .size(size)
                .rounded_md()
                .bg(black())
                .when(self.ticked().is_empty(), |el| {
                    el.flex()
                        .items_center()
                        .justify_center()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Nothing ticked")
                })
                .into_any_element(),
        }
    }
}

/// A small heading, as Help's are.
fn heading(text: &'static str, cx: &App) -> Div {
    div()
        .mb_1()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(cx.theme().muted_foreground)
        .child(text.to_uppercase())
}

impl Render for TeachKame {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let name = self.checked_name(cx);
        let shown_name = match &name {
            Ok(name) => name.clone(),
            Err(_) => self.name.read(cx).value().trim().to_uppercase(),
        };
        let ticked = self.ticked();
        let ticked_count = ticked.len();
        let ready = name.is_ok() && ticked_count > 0;
        let code = procedure(
            if shown_name.is_empty() {
                "…"
            } else {
                &shown_name
            },
            &ticked,
        );
        let rows: Vec<_> = (0..self.rows.len())
            .map(|i| self.render_row(i, cx))
            .collect();
        let tickable = self.rows.iter().filter(|r| r.fixed.is_none()).count();

        v_flex()
            .w_full()
            .gap_3()
            // The header: Kame, and what to do.
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(kame(72., 48.))
                    .child(
                        v_flex()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Teach Kame a new word"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child("Tick what Kame should remember, and give it a name."),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .h(px(400.))
                    .gap_4()
                    .items_start()
                    // The entries since the last Run.
                    .child(
                        v_flex()
                            .w(px(300.))
                            .h_full()
                            .flex_none()
                            .border_1()
                            .border_color(theme.border)
                            .rounded_lg()
                            .overflow_hidden()
                            .child(
                                h_flex()
                                    .gap_1()
                                    .px_2()
                                    .py_1()
                                    .border_b_1()
                                    .border_color(theme.border)
                                    .bg(theme.muted)
                                    .child(
                                        div()
                                            .flex_1()
                                            .text_xs()
                                            .text_color(theme.muted_foreground)
                                            .child(format!(
                                                "Since the last Run · {ticked_count} of {tickable} ticked"
                                            )),
                                    )
                                    .child(
                                        Button::new("teach-all")
                                            .xsmall()
                                            .outline()
                                            .label("All")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.tick_all(true, cx)
                                            })),
                                    )
                                    .child(
                                        Button::new("teach-none")
                                            .xsmall()
                                            .outline()
                                            .label("None")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.tick_all(false, cx)
                                            })),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h_0()
                                    .p_1()
                                    .children(rows)
                                    .overflow_y_scrollbar()
                                    .id("teach-rows"),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .border_t_1()
                                    .border_color(theme.border)
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child("Shift-click to tick a range. Undone entries aren't here."),
                            ),
                    )
                    // The name, what goes in the program, and what it draws.
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .gap_3()
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .child("Name for the new word"),
                                    )
                                    .child(
                                        Input::new(&self.name)
                                            .font_family(theme.mono_font_family.clone()),
                                    )
                                    .when_some(name.as_ref().err().cloned().flatten(), |el, why| {
                                        el.child(div().text_xs().text_color(theme.danger).child(why))
                                    }),
                            )
                            .child(
                                h_flex()
                                    .flex_1()
                                    .min_h_0()
                                    .gap_3()
                                    .items_start()
                                    .child(
                                        v_flex()
                                            .flex_1()
                                            .min_w_0()
                                            .child(heading("Goes in your program", cx))
                                            .child(
                                                div()
                                                    .max_h(px(250.))
                                                    .child(code_block(&code, None, &theme))
                                                    .overflow_y_scrollbar()
                                                    .id("teach-code"),
                                            ),
                                    )
                                    .child(
                                        v_flex()
                                            .flex_none()
                                            .w(px(PICTURE))
                                            .child(heading("Kame draws", cx))
                                            .child(self.render_picture(cx))
                                            .child(
                                                div()
                                                    .mt_1()
                                                    .text_xs()
                                                    .text_color(theme.muted_foreground)
                                                    .child("From the centre, facing up"),
                                            ),
                                    ),
                            ),
                    ),
            )
            // What happens, and the buttons.
            .child(
                h_flex()
                    .gap_2()
                    .pt_3()
                    .border_t_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("It goes at the end of your program, and works at the console straight away."),
                    )
                    .child(
                        Button::new("teach-cancel")
                            .outline()
                            .label("Cancel")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        Button::new("teach")
                            .primary()
                            .icon(IconName::MessageSquarePlus)
                            .label(if shown_name.is_empty() || name.is_err() {
                                "Teach Kame".to_string()
                            } else {
                                format!("Teach Kame {shown_name}")
                            })
                            .disabled(!ready)
                            .on_click(cx.listener(|this, _, window, cx| this.teach(window, cx))),
                    ),
            )
    }
}

/// The dialog's width, for opening it.
pub fn width() -> Pixels {
    px(WIDTH)
}
