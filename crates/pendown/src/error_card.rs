//! An error over the editor: its message at the end of the line, and a card
//! under the line saying how the command is written. It sits over the editor
//! rather than opening a gap in the code, so it's placed from the editor's
//! scroll offset and line height, as the gutter is, and measured text. Soft
//! wrap and folding are off, so each row is one line. `range_to_bounds` won't
//! do: it lags a frame, and puts lines scrolled above the view at the top.
//!
//! The editor's own hover popup is never drawn. The editor has no way to turn
//! it off, so it serves as the signal for which error the pointer's on: after
//! each mouse move, it's read and cleared, before the next frame.

use std::{cell::Cell, rc::Rc};

use gpui_kit::component::{
    ActiveTheme as _, IconName, Sizable as _,
    button::{Button, ButtonVariants as _},
    input::EditorState,
};
use gpui_kit::{prelude::FluentBuilder as _, *};
use pendown::error_display::Shown;
use pendown_logo::{Span, explain::Explanation, reference::reference};

/// The editor's text starts this far from its top and left: gpui-kit's
/// padding for a medium editor.
const TOP: f32 = 8.;
const LEFT: f32 = 12.;

/// The gap between the end of a line and its message.
const MESSAGE_GAP: f32 = 12.;
const MESSAGE_HEIGHT: f32 = 18.;
const MESSAGE_PADDING: f32 = 7.;
/// How close to the pane's edge either one goes.
const MARGIN: f32 = 4.;

const CARD_WIDTH: f32 = 350.;
const CARD_PADDING_X: f32 = 11.;
const CARD_PADDING_Y: f32 = 9.;
const CARD_GAP: f32 = 7.;
/// The gap between the line and the card.
const LINE_GAP: f32 = 3.;
const TITLE_LINE: f32 = 17.;
const LABEL_LINE: f32 = 16.;
const CODE_LINE: f32 = 19.;
const FOOT: f32 = 24.;
/// Room at the card's top right for its close button, which the message
/// mustn't run under when it's in the card.
const CLOSE_ROOM: f32 = 22.;

/// The mockup's colours: dark red on pale red has good contrast, and it's
/// never the only signal, as the line is underlined and the words say it.
const ERROR_BG: u32 = 0xfef2f2;
const ERROR_LINE: u32 = 0xfecaca;
const ERROR_FG: u32 = 0x991b1b;

/// The dot at the end of a line with an error.
const DOT: u32 = 0xdc2626;
const DOT_SIZE: f32 = 7.;
const DOT_GAP: f32 = 10.;

/// What `place` needs to know, in pixels from the top left of the pane.
#[derive(Clone, Copy, Debug)]
struct Layout {
    pane: (f32, f32),
    line_top: f32,
    line_height: f32,
    /// Where the line's text ends, and where the error starts.
    line_end: f32,
    error_start: f32,
    message_width: f32,
    /// The height of the message as the card's first lines.
    message_height: f32,
    /// The height of the syntax and example in the card, with their gaps.
    rows_height: f32,
    /// There's something for a card to say beyond the message.
    has_body: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Placement {
    /// Where the message goes at the end of the line, if it fits there.
    message: Option<(f32, f32)>,
    card: Option<CardPlace>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CardPlace {
    at: (f32, f32),
    width: f32,
    height: f32,
    /// The message didn't fit at the end of the line.
    has_message: bool,
}

/// How wide the card is in a pane this wide.
fn card_width(pane_width: f32) -> f32 {
    CARD_WIDTH.min(pane_width - 2. * MARGIN).max(0.)
}

/// Where an error's message and card go, or nothing if its line is out of
/// view. The card goes under the line if it fits, or above if that fits, or
/// else on whichever side has more room. It starts at the error's column,
/// and stays in the pane.
fn place(layout: Layout) -> Option<Placement> {
    let (pane_width, pane_height) = layout.pane;
    let line_bottom = layout.line_top + layout.line_height;
    if line_bottom <= 0. || layout.line_top >= pane_height {
        return None;
    }
    let message_x = layout.line_end + MESSAGE_GAP;
    let fits = message_x + layout.message_width <= pane_width - MARGIN;
    let message = fits.then_some((
        message_x,
        layout.line_top + (layout.line_height - MESSAGE_HEIGHT) / 2.,
    ));
    if fits && !layout.has_body {
        return Some(Placement {
            message,
            card: None,
        });
    }
    let moved = if fits {
        0.
    } else {
        layout.message_height + CARD_GAP
    };
    let body = if layout.rows_height > 0. {
        layout.rows_height + CARD_GAP
    } else {
        0.
    };
    let height = 2. + 2. * CARD_PADDING_Y + moved + body + FOOT;
    let width = card_width(pane_width);
    let below = line_bottom + LINE_GAP;
    let above = layout.line_top - LINE_GAP - height;
    let room_below = pane_height - MARGIN - below;
    let room_above = layout.line_top - LINE_GAP - MARGIN;
    let y = if room_below >= height || room_below >= room_above {
        below
    } else {
        above
    };
    let x = layout
        .error_start
        .min(pane_width - MARGIN - width)
        .max(MARGIN);
    Some(Placement {
        message,
        card: Some(CardPlace {
            at: (x, y),
            width,
            height,
            has_message: !fits,
        }),
    })
}

/// How tall the syntax and example are in the card.
fn rows_height(explanation: &Explanation) -> f32 {
    let row = LABEL_LINE + CODE_LINE;
    match (explanation.usage.is_some(), explanation.example.is_some()) {
        (true, true) => 2. * row + CARD_GAP,
        (false, false) => 0.,
        _ => row,
    }
}

/// An error on the editor's code.
pub struct LineError {
    /// Counting from 0.
    line: usize,
    /// Bytes into the line.
    column: usize,
    explanation: Explanation,
}

impl LineError {
    /// `span` is in `text`.
    pub fn new(text: &str, span: Span, explanation: Explanation) -> LineError {
        let before = &text[..(span.start as usize).min(text.len())];
        LineError {
            line: before.matches('\n').count(),
            column: before.len() - before.rfind('\n').map_or(0, |i| i + 1),
            explanation,
        }
    }
}

/// The card's Help button was pressed, for this page.
pub struct OpenHelp(pub usize);

/// The card's close button was pressed.
pub struct HideCard;

/// The mouse moved.
pub struct PointerMoved {
    /// Where the error the pointer's on starts, in bytes into the text.
    pub error: Option<usize>,
    /// It's on the card showing or that card's line, so the card should
    /// stay, for reaching its button.
    pub near_card: bool,
}

pub struct ErrorCard {
    editor: Entity<EditorState>,
    errors: Vec<LineError>,
    /// The error showing, as an index into `errors`.
    showing: Option<usize>,
    /// Lines with an error the card isn't on, for a red dot.
    dots: Vec<usize>,
    /// The pane, from its last layout.
    pane: Rc<Cell<Bounds<Pixels>>>,
    /// The line and card showing, in the pane.
    near_card: Vec<Bounds<Pixels>>,
    _observe: Subscription,
}

impl EventEmitter<OpenHelp> for ErrorCard {}
impl EventEmitter<PointerMoved> for ErrorCard {}
impl EventEmitter<HideCard> for ErrorCard {}

impl ErrorCard {
    pub fn new(editor: Entity<EditorState>, cx: &mut Context<Self>) -> ErrorCard {
        // Scrolling, typing and the cursor all move it.
        let observe = cx.observe(&editor, |_, _, cx| cx.notify());
        ErrorCard {
            editor,
            errors: Vec::new(),
            showing: None,
            dots: Vec::new(),
            pane: Rc::default(),
            near_card: Vec::new(),
            _observe: observe,
        }
    }

    /// The errors on the code, and what to show of them.
    pub fn set(&mut self, errors: Vec<LineError>, shown: &Shown, cx: &mut Context<Self>) {
        self.errors = errors;
        self.show(shown, cx);
    }

    pub fn show(&mut self, shown: &Shown, cx: &mut Context<Self>) {
        self.showing = shown.card;
        self.dots.clone_from(&shown.dots);
        cx.notify();
    }

    /// For the scripted session: the message, syntax and example on show.
    pub fn on_show(&self) -> Option<&Explanation> {
        Some(&self.errors.get(self.showing?)?.explanation)
    }

    /// After a mouse move: every listener has run, so the editor has set its
    /// hover popup if the pointer's on an underline. Clearing it means the
    /// popup isn't drawn, and that the editor sets it again on the next move.
    fn pointer_moved(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        if self.errors.is_empty() {
            return;
        }
        let popup = self.editor.read(cx).diagnostic_popover();
        if popup.is_some() {
            self.editor
                .update(cx, |state, cx| state.clear_diagnostic_popover(cx));
        }
        let origin = self.pane.get().origin;
        let inside = point(position.x - origin.x, position.y - origin.y);
        cx.emit(PointerMoved {
            error: popup.map(|entry| entry.range.start),
            near_card: self.near_card.iter().any(|b| b.contains(&inside)),
        });
    }
}

/// How wide `text` is in this font.
fn measure(text: &str, font: Font, size: Pixels, window: &Window) -> f32 {
    let run = TextRun {
        len: text.len(),
        font,
        color: black(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let shaped = window
        .text_system()
        .shape_line(text.to_string().into(), size, &[run], None);
    f32::from(shaped.width())
}

/// How many lines `text` takes up, wrapped to this width.
fn lines_when_wrapped(text: &str, font: Font, size: Pixels, width: f32, window: &Window) -> usize {
    let run = TextRun {
        len: text.len(),
        font,
        color: black(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    window
        .text_system()
        .shape_text(text.to_string().into(), size, &[run], Some(px(width)), None)
        .map_or(1, |lines| {
            lines.iter().map(|l| l.wrap_boundaries.len() + 1).sum()
        })
}

impl ErrorCard {
    /// A row of the card: a label, and a line of code.
    fn row(label: &'static str, code: StyledText, cx: &App) -> Div {
        let theme = cx.theme();
        div()
            .child(
                div()
                    .h(px(LABEL_LINE))
                    .text_size(px(11.))
                    .line_height(px(LABEL_LINE))
                    .text_color(theme.muted_foreground)
                    .child(label),
            )
            .child(
                div()
                    .h(px(CODE_LINE))
                    .font_family(theme.mono_font_family.clone())
                    .text_size(theme.mono_font_size)
                    .line_height(px(CODE_LINE))
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(code),
            )
    }
}

impl Render for ErrorCard {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The whole pane, with no handlers of its own: clicks go through.
        let root = div().absolute().inset_0().child({
            let pane = self.pane.clone();
            let view = cx.entity_id();
            let card = cx.weak_entity();
            canvas(
                move |bounds, _, cx| {
                    if pane.replace(bounds).size != bounds.size {
                        cx.notify(view);
                    }
                },
                move |_, _, window, _| {
                    let card = card.clone();
                    window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                        if phase != DispatchPhase::Capture {
                            return;
                        }
                        let (card, position) = (card.clone(), event.position);
                        // Once every listener has run, including the editor's.
                        cx.defer(move |cx| {
                            card.update(cx, |card, cx| card.pointer_moved(position, cx))
                                .ok();
                        });
                    });
                },
            )
            .absolute()
            .size_full()
        });
        self.near_card.clear();
        if self.errors.is_empty() {
            return root;
        }
        let theme = cx.theme().clone();
        let state = self.editor.read(cx);
        let line_height = f32::from(state.line_height().unwrap_or(px(20.)));
        let scroll = state.scroll_offset();
        let text = state.value().to_string();
        let lines: Vec<&str> = text
            .split('\n')
            .map(|l| l.strip_suffix('\r').unwrap_or(l))
            .collect();
        let mono = font(theme.mono_font_family.clone());
        let left = LEFT + f32::from(scroll.x);
        let code_size = theme.mono_font_size;
        let top_of = |line: usize| TOP + f32::from(scroll.y) + line_height * line as f32;

        let pane_height = f32::from(self.pane.get().size.height);
        let dots = self.dots.iter().filter_map(|&line| {
            let top = top_of(line);
            if top + line_height <= 0. || top >= pane_height {
                return None;
            }
            let end = measure(lines.get(line)?, mono.clone(), code_size, window);
            Some(
                div()
                    .absolute()
                    .left(px(left + end + DOT_GAP))
                    .top(px(top + (line_height - DOT_SIZE) / 2.))
                    .size(px(DOT_SIZE))
                    .rounded_full()
                    .bg(rgb(DOT)),
            )
        });
        let root = root.children(dots.collect::<Vec<_>>());
        let Some(error) = self.showing.and_then(|i| self.errors.get(i)) else {
            return root;
        };
        let line = lines.get(error.line).copied().unwrap_or_default();
        let before = line.get(..error.column).unwrap_or(line);

        let mut bold = font(theme.font_family.clone());
        bold.weight = FontWeight::BOLD;
        let message = &error.explanation.message;
        let pane = self.pane.get().size;
        let pane = (f32::from(pane.width), f32::from(pane.height));
        let inner = (card_width(pane.0) - 2. - 2. * CARD_PADDING_X - CLOSE_ROOM).max(1.);
        let wrapped = lines_when_wrapped(message, bold.clone(), px(12.), inner, window);
        let placement = place(Layout {
            pane,
            line_top: top_of(error.line),
            line_height,
            line_end: left + measure(line, mono.clone(), code_size, window),
            error_start: left + measure(before, mono.clone(), code_size, window),
            message_width: measure(message, bold.clone(), px(12.), window) + 2. * MESSAGE_PADDING,
            message_height: TITLE_LINE * wrapped as f32,
            rows_height: rows_height(&error.explanation),
            has_body: error.explanation.usage.is_some() || error.explanation.help.is_some(),
        });
        let Some(placement) = placement else {
            return root;
        };
        // On the way from an underline to the card's button, the pointer
        // crosses the gap between them, so the line's row takes that in.
        let mut near_card = vec![Bounds::new(
            point(px(0.), px(top_of(error.line) - LINE_GAP)),
            size(px(pane.0), px(line_height + 2. * LINE_GAP)),
        )];
        near_card.extend(placement.card.map(|place| {
            Bounds::new(
                point(px(place.at.0), px(place.at.1)),
                size(px(place.width), px(place.height)),
            )
        }));

        let message_on_line = placement.message.map(|(x, y)| {
            div()
                .absolute()
                .left(px(x))
                .top(px(y))
                .h(px(MESSAGE_HEIGHT))
                .px(px(MESSAGE_PADDING))
                .rounded(px(4.))
                .bg(rgb(ERROR_BG))
                .text_color(rgb(ERROR_FG))
                .font_family(theme.font_family.clone())
                .font_weight(FontWeight::BOLD)
                .text_size(px(12.))
                .line_height(px(MESSAGE_HEIGHT))
                .whitespace_nowrap()
                .child(message.clone())
        });

        let card = placement.card.map(|place| {
            let resolver = &*theme.highlight_theme;
            let explanation = &error.explanation;
            let usage = explanation.usage.as_ref().map(|usage| {
                let styles = crate::highlighter::usage_styles(usage, resolver);
                StyledText::new(usage.clone()).with_highlights(styles)
            });
            let example = explanation.example.as_ref().map(|example| {
                let styles = crate::highlighter::program_styles(example, resolver, None);
                StyledText::new(example.clone()).with_highlights(styles)
            });
            let editor = self.editor.clone();
            div()
                .id("error-card")
                .absolute()
                .left(px(place.at.0))
                .top(px(place.at.1))
                .w(px(place.width))
                .h(px(place.height))
                .flex()
                .flex_col()
                .gap(px(CARD_GAP))
                .px(px(CARD_PADDING_X))
                .py(px(CARD_PADDING_Y))
                .overflow_hidden()
                .rounded(px(6.))
                .border_1()
                .border_color(rgb(ERROR_LINE))
                .bg(theme.background)
                .shadow_md()
                .font_family(theme.font_family.clone())
                .text_size(px(12.))
                .text_color(theme.foreground)
                // Clicks stop here, so they don't reach the code it hides,
                // but the wheel still scrolls the editor.
                .occlude()
                .on_scroll(move |event: &ScrollEvent, _, cx| {
                    let delta = event.delta.pixel_delta(px(line_height));
                    editor.update(cx, |state, cx| {
                        let offset = state.scroll_offset();
                        state.set_scroll_offset(offset + delta, cx);
                    });
                })
                .when(place.has_message, |card| {
                    card.child(
                        div()
                            .text_color(rgb(ERROR_FG))
                            .font_weight(FontWeight::BOLD)
                            .line_height(px(TITLE_LINE))
                            .pr(px(CLOSE_ROOM))
                            .child(message.clone()),
                    )
                })
                .when_some(usage, |card, usage| {
                    card.child(Self::row("Written like this", usage, cx))
                })
                .when_some(example, |card, example| {
                    card.child(Self::row("Example", example, cx))
                })
                .child(
                    div().absolute().top(px(4.)).right(px(4.)).child(
                        Button::new("error-card-close")
                            .xsmall()
                            .ghost()
                            .icon(IconName::Close)
                            .tooltip("Hide this (Esc)")
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(HideCard))),
                    ),
                )
                .child(
                    div()
                        .h(px(FOOT))
                        .flex()
                        .items_center()
                        .justify_between()
                        .text_size(px(11.))
                        .text_color(theme.muted_foreground)
                        .child("Esc hides this")
                        .when_some(explanation.defined_on, |foot, line| {
                            foot.child(format!("Defined on line {line}"))
                        })
                        .when_some(explanation.help, |foot, i| {
                            let title = &reference().commands[i].title;
                            foot.child(
                                Button::new("error-card-help")
                                    .xsmall()
                                    .outline()
                                    .label(format!("Help on {title}"))
                                    .on_click(cx.listener(move |_, _, _, cx| {
                                        cx.emit(OpenHelp(i));
                                    })),
                            )
                        }),
                )
        });
        let root = root.children(message_on_line).children(card);
        self.near_card = near_card;
        root
    }
}

#[cfg(test)]
mod tests {
    use pendown_logo::{SourceId, Span, explain::Explanation};

    use super::{CardPlace, Layout, LineError, place, rows_height};

    /// A line in the middle of a roomy pane, with a syntax and an example.
    fn layout() -> Layout {
        Layout {
            pane: (460., 640.),
            line_top: 200.,
            line_height: 20.,
            line_end: 100.,
            error_start: 40.,
            message_width: 220.,
            message_height: 17.,
            rows_height: 77.,
            has_body: true,
        }
    }

    fn card(layout: Layout) -> CardPlace {
        place(layout).unwrap().card.unwrap()
    }

    #[test]
    fn the_message_ends_the_line_and_the_card_goes_under_it() {
        let placement = place(layout()).unwrap();
        assert_eq!(placement.message, Some((112., 201.)));
        let card = placement.card.unwrap();
        assert_eq!(card.at, (40., 223.));
        assert_eq!(card.width, 350.);
        assert!(!card.has_message);
    }

    #[test]
    fn a_line_out_of_view_shows_nothing() {
        for line_top in [-20., -500., 640., 900.] {
            assert_eq!(
                place(Layout {
                    line_top,
                    ..layout()
                }),
                None,
                "{line_top}"
            );
        }
    }

    #[test]
    fn a_line_partly_in_view_still_shows() {
        for line_top in [-19., 639.] {
            assert!(
                place(Layout {
                    line_top,
                    ..layout()
                })
                .is_some(),
                "{line_top}"
            );
        }
    }

    #[test]
    fn the_first_line_has_its_card_under_it() {
        let card = card(Layout {
            line_top: 8.,
            ..layout()
        });
        assert_eq!(card.at.1, 31.);
    }

    #[test]
    fn the_last_line_has_its_card_above_it() {
        let card = card(Layout {
            line_top: 616.,
            ..layout()
        });
        assert_eq!(card.at.1, 616. - 3. - card.height);
    }

    #[test]
    fn with_no_room_either_way_it_takes_the_side_with_more() {
        let short = Layout {
            pane: (460., 120.),
            ..layout()
        };
        let lower = card(Layout {
            line_top: 80.,
            ..short
        });
        assert_eq!(lower.at.1, 80. - 3. - lower.height);
        let higher = card(Layout {
            line_top: 10.,
            ..short
        });
        assert_eq!(higher.at.1, 33.);
    }

    #[test]
    fn the_card_stays_in_the_pane_sideways() {
        // Near the right edge, and scrolled off to the left.
        assert_eq!(
            card(Layout {
                error_start: 300.,
                ..layout()
            })
            .at
            .0,
            460. - 4. - 350.
        );
        assert_eq!(
            card(Layout {
                error_start: -90.,
                ..layout()
            })
            .at
            .0,
            4.
        );
    }

    #[test]
    fn a_narrow_pane_narrows_the_card_and_moves_the_message_into_it() {
        let wide = card(layout());
        let placement = place(Layout {
            pane: (200., 640.),
            ..layout()
        })
        .unwrap();
        assert_eq!(placement.message, None);
        let narrow = placement.card.unwrap();
        assert_eq!(narrow.width, 192.);
        assert_eq!(narrow.at.0, 4.);
        assert!(narrow.has_message);
        assert_eq!(narrow.height, wide.height + 17. + 7.);
    }

    #[test]
    fn a_long_line_moves_the_message_into_the_card() {
        let placement = place(Layout {
            line_end: 300.,
            ..layout()
        })
        .unwrap();
        assert_eq!(placement.message, None);
        assert!(placement.card.unwrap().has_message);
    }

    #[test]
    fn a_message_that_just_fits_stays_on_the_line() {
        // 4 from the edge.
        let fits = place(Layout {
            line_end: 460. - 4. - 220. - 12.,
            ..layout()
        })
        .unwrap();
        assert!(fits.message.is_some());
        let over = place(Layout {
            line_end: 460. - 4. - 220. - 12. + 1.,
            ..layout()
        })
        .unwrap();
        assert_eq!(over.message, None);
    }

    #[test]
    fn a_bare_message_has_no_card_unless_it_needs_one() {
        let bare = Layout {
            rows_height: 0.,
            has_body: false,
            ..layout()
        };
        assert_eq!(place(bare).unwrap().card, None);
        let moved = place(Layout {
            line_end: 300.,
            ..bare
        })
        .unwrap();
        assert!(moved.card.unwrap().has_message);
    }

    #[test]
    fn rows_are_as_tall_as_what_they_hold() {
        let usage = Some("FD distance".to_string());
        let both = Explanation {
            usage: usage.clone(),
            example: Some("FD 100".into()),
            ..Explanation::default()
        };
        assert_eq!(rows_height(&both), 77.);
        let one = Explanation {
            usage,
            ..Explanation::default()
        };
        assert_eq!(rows_height(&one), 35.);
        assert_eq!(rows_height(&Explanation::default()), 0.);
    }

    #[test]
    fn a_line_error_finds_its_line_and_column() {
        let text = "FD 1\nüü FOO 2";
        let start = text.find("FOO").unwrap() as u32;
        let span = Span {
            source: SourceId::Editor,
            start,
            end: start + 3,
            line: 1,
        };
        let error = LineError::new(text, span, Explanation::default());
        assert_eq!((error.line, error.column), (1, 5));
    }
}
