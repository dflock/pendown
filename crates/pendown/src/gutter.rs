//! The editor's line numbers, drawn by Pendown rather than gpui-kit's
//! editor so they can take breakpoints (ADR 0029): click a line number to
//! stop there, and it shows as a white number on a red pill. The rows
//! follow the editor's scroll offset and line height. Soft wrap and folding
//! are off, so each row is one line. The line the editor's highlight is on
//! has a green arrow behind its number; a breakpoint's pill there gets a
//! white ring, so the two shapes stay apart for red/green colour blind eyes.

use std::collections::BTreeSet;

use gpui_kit::component::{ActiveTheme as _, input::EditorState, tooltip::Tooltip};
use gpui_kit::{prelude::FluentBuilder as _, *};
use pendown::breakpoints;

/// The editor's text starts this far below its top: gpui-kit's padding for
/// a medium editor.
const TOP: Pixels = px(8.);

/// More rows than any window shows at once.
const MAX_ROWS: usize = 200;

/// The current line's arrow, and a breakpoint's pill.
const GREEN: u32 = 0x15803d;
const RED: u32 = 0xdc2626;

/// Room to the right of the numbers, where the arrow's point shows.
const RIGHT: Pixels = px(9.);

/// A breakpoint was set or taken away.
pub struct BreakpointsChanged;

pub struct Gutter {
    editor: Entity<EditorState>,
    /// Lines with a breakpoint, counting from 0.
    pub lines: BTreeSet<usize>,
    /// The text they're lines of, to follow them through edits.
    text: String,
    /// The line the editor's highlight is on, counting from 0.
    current: Option<usize>,
    _observe: Subscription,
}

impl EventEmitter<BreakpointsChanged> for Gutter {}

impl Gutter {
    pub fn new(editor: Entity<EditorState>, cx: &mut Context<Self>) -> Gutter {
        // Scrolling, typing and the cursor all move or change the numbers.
        let observe = cx.observe(&editor, |_, _, cx| cx.notify());
        let text = editor.read(cx).value().to_string();
        Gutter {
            editor,
            lines: BTreeSet::new(),
            text,
            current: None,
            _observe: observe,
        }
    }

    /// Moves the breakpoints with their lines, after the editor's text
    /// changed.
    pub fn follow(&mut self, cx: &mut Context<Self>) {
        let text = self.editor.read(cx).value().to_string();
        if text != self.text {
            let lines = breakpoints::follow(&self.text, &text, &self.lines);
            self.text = text;
            if lines != self.lines {
                self.lines = lines;
                cx.emit(BreakpointsChanged);
            }
            cx.notify();
        }
    }

    /// No breakpoints: for a new or opened program.
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.text = self.editor.read(cx).value().to_string();
        if !self.lines.is_empty() {
            self.lines.clear();
            cx.emit(BreakpointsChanged);
        }
        cx.notify();
    }

    /// Sets or takes away the breakpoint on `line`, if it has code.
    pub fn toggle(&mut self, line: usize, cx: &mut Context<Self>) {
        self.follow(cx);
        if !self.lines.remove(&line) {
            if !self
                .text
                .split('\n')
                .nth(line)
                .is_some_and(breakpoints::can_take)
            {
                return;
            }
            self.lines.insert(line);
        }
        cx.emit(BreakpointsChanged);
        cx.notify();
    }

    /// Marks the line the editor's highlight is on, or none.
    pub fn set_current(&mut self, line: Option<usize>, cx: &mut Context<Self>) {
        if self.current != line {
            self.current = line;
            cx.notify();
        }
    }

    /// For the scripted session: the line the arrow is on.
    pub fn current(&self) -> Option<usize> {
        self.current
    }

    /// Where the interpreter stops for them in the editor's text now.
    pub fn stops(&self, cx: &App) -> Vec<u32> {
        breakpoints::stops(self.editor.read(cx).value().as_ref(), &self.lines)
    }
}

impl Render for Gutter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let state = self.editor.read(cx);
        let text = state.value().to_string();
        let line_height = state.line_height().unwrap_or(px(20.));
        let scroll = state.scroll_offset().y;
        let rows = state.visible_row_range().unwrap_or(0..0);
        let cursor_line = state.cursor_position().line as usize;
        let count = text.split('\n').count();
        // As wide as gpui-kit's numbers were: three digits at least.
        let digits = count.to_string().len().max(3);
        let width = px(16. + 8.5 * digits as f32) + RIGHT - px(6.);
        let lines: Vec<&str> = text.split('\n').collect();
        // The editor's visible rows are from its last layout, which can be a
        // frame behind the text: so as many rows as could fit, from there.
        let first = rows.start.min(count);
        let last = (first + MAX_ROWS).min(count);
        let numbers = (first..last).map(|line| {
            let on = self.lines.contains(&line);
            let current = self.current == Some(line);
            let code = lines.get(line).is_some_and(|l| breakpoints::can_take(l));
            let colour = if current {
                gpui::white()
            } else if line == cursor_line {
                theme.foreground
            } else {
                theme.muted_foreground
            };
            let pill = div()
                .id(("line-number", line))
                .flex()
                .items_center()
                .justify_center()
                .min_w(px(22.))
                .h(line_height - px(4.))
                .px(px(4.))
                .rounded_full()
                .child((line + 1).to_string());
            let pill = if on {
                let pill = pill
                    .bg(rgb(RED))
                    .text_color(gpui::white())
                    .font_weight(FontWeight::MEDIUM)
                    .hover(|s| s.bg(rgb(0xb91c1c)));
                if current {
                    pill.border(px(1.5)).border_color(gpui::white())
                } else {
                    pill
                }
            } else if current {
                let pill = pill.text_color(colour).font_weight(FontWeight::SEMIBOLD);
                if code {
                    pill.hover(|s| s.bg(hsla(0., 0.72, 0.51, 0.45)))
                } else {
                    pill
                }
            } else if code {
                pill.text_color(colour)
                    .hover(|s| s.bg(hsla(0., 0.72, 0.51, 0.22)).text_color(rgb(0x7f1d1d)))
            } else {
                pill.text_color(colour)
            };
            let pill = if code || on {
                let tip = tooltip(lines.get(line).copied().unwrap_or(""), on);
                pill.cursor_pointer()
                    .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            cx.stop_propagation();
                            this.toggle(line, cx);
                        }),
                    )
            } else {
                pill
            };
            div()
                .relative()
                .h(line_height)
                .flex()
                .items_center()
                .justify_end()
                .pr(RIGHT)
                .when(current, |row| row.child(arrow()))
                .child(pill)
        });
        div()
            .id("gutter")
            .w(width)
            .h_full()
            .flex_none()
            // The wheel over the numbers scrolls the editor.
            .on_scroll(cx.listener(move |this, event: &ScrollEvent, _, cx| {
                let delta = event.delta.pixel_delta(line_height);
                this.editor.update(cx, |state, cx| {
                    let offset = state.scroll_offset();
                    state.set_scroll_offset(point(offset.x, offset.y + delta.y), cx);
                });
            }))
            .relative()
            .overflow_hidden()
            .bg(theme.background)
            .font_family(theme.mono_font_family.clone())
            .text_size(theme.mono_font_size)
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(TOP + scroll + line_height * first as f32)
                    .children(numbers),
            )
    }
}

/// What clicking a line's number does.
fn tooltip(line: &str, on: bool) -> SharedString {
    if on {
        return "Take the breakpoint away".into();
    }
    let mut words = line.split_whitespace();
    match (words.next(), words.next()) {
        (Some(to), Some(name)) if to.eq_ignore_ascii_case("to") => {
            format!("Stop each time {} is called", name.to_uppercase()).into()
        }
        _ => "Stop here: Kame waits before this line".into(),
    }
}

/// The current line's marker: green, rounded on the left and pointed on
/// the right, filling the row behind its number.
fn arrow() -> impl IntoElement {
    gpui_kit::canvas(
        |_, _, _| {},
        |bounds, _, window, _| {
            let Bounds { origin, size } = bounds;
            let (top, bottom) = (origin.y + px(1.), origin.y + size.height - px(1.));
            let middle = origin.y + size.height / 2.;
            let left = origin.x + px(2.);
            let tip = origin.x + size.width - px(1.);
            let shoulder = tip - (bottom - top) / 2.;
            let r = px(3.);
            let mut path = PathBuilder::fill();
            path.move_to(point(left + r, top));
            path.line_to(point(shoulder, top));
            path.line_to(point(tip, middle));
            path.line_to(point(shoulder, bottom));
            path.line_to(point(left + r, bottom));
            path.curve_to(point(left, bottom - r), point(left, bottom));
            path.line_to(point(left, top + r));
            path.curve_to(point(left + r, top), point(left, top));
            path.close();
            if let Ok(path) = path.build() {
                window.paint_path(path, rgb(GREEN));
            }
        },
    )
    .absolute()
    .inset_0()
}
