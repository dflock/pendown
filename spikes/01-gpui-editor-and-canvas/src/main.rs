//! Spike 01: gpui-kit editor and canvas in one window.
//!
//! See `.scratch/phase-1-spikes/issues/01-gpui-editor-and-canvas.md`.
//!
//! Run with `cargo run --release`. Set `SPIKE_AUTOTEST=1` to run a timed,
//! scripted session that logs measurements to stdout and then quits.

use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui_kit::component::{
    ActiveTheme, Sizable,
    button::{Button, ButtonVariants as _},
    h_flex,
    highlighter::{Diagnostic, DiagnosticSeverity},
    input::{
        Editor, EditorState, FoldRange, HighlightStyleResolver, InputEdit, InputEvent,
        InputHighlighter, Position, RangeDecoration, RangeDecorationCollection,
        RangeDecorationStyle, Rope,
    },
    resizable::{h_resizable, resizable_panel},
    v_flex,
};
use gpui_kit::{prelude::FluentBuilder as _, *};

const SAMPLE: &str = r#"; A small Logo program
TO SQUARE :size
  REPEAT 4 [FD :size RT 90]
END

TO FLOWER :size
  REPEAT 12 [SQUARE :size RT 30]
END

SETPENCOLOR "gold
FLOWER 80
FOOBAR 3          ; not a known command: should get a diagnostic
"#;

const COMMANDS: &[&str] = &[
    "fd", "forward", "bk", "back", "rt", "right", "lt", "left", "repeat", "to", "end", "make",
    "if", "ifelse", "stop", "output", "print", "pu", "pd", "penup", "pendown", "setpencolor",
    "setpc", "home", "cs", "clearscreen", "arc", "circle", "forever",
];

// ---------------------------------------------------------------- a tiny Logo lexer

#[derive(Clone, Copy, PartialEq, Debug)]
enum Tok {
    Comment,
    Number,
    Variable,
    Quoted,
    Bracket,
    Operator,
    Command,
    Word,
}

fn lex(text: &str) -> Vec<(Range<usize>, Tok)> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        let start = i;
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let tok = match c {
            b';' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                Tok::Comment
            }
            b'[' | b']' | b'(' | b')' => {
                i += 1;
                Tok::Bracket
            }
            b'+' | b'-' | b'*' | b'/' | b'=' | b'<' | b'>' => {
                i += 1;
                Tok::Operator
            }
            _ => {
                while i < bytes.len() && !b" \t\r\n[]();".contains(&bytes[i]) {
                    i += 1;
                }
                let word = &text[start..i];
                if word.starts_with(':') {
                    Tok::Variable
                } else if word.starts_with('"') {
                    Tok::Quoted
                } else if word.parse::<f64>().is_ok() {
                    Tok::Number
                } else if COMMANDS.contains(&word.to_lowercase().as_str()) {
                    Tok::Command
                } else {
                    Tok::Word
                }
            }
        };
        out.push((start..i, tok));
    }
    out
}

/// Approximate instruction spans: a command or procedure name plus the
/// arguments after it, up to the next name or bracket.
fn instructions(tokens: &[(Range<usize>, Tok)]) -> Vec<Range<usize>> {
    let mut out: Vec<Range<usize>> = Vec::new();
    let mut current: Option<Range<usize>> = None;
    for (range, tok) in tokens {
        match tok {
            Tok::Command | Tok::Word => {
                if let Some(r) = current.take() {
                    out.push(r);
                }
                current = Some(range.clone());
            }
            Tok::Number | Tok::Variable | Tok::Quoted | Tok::Operator => {
                if let Some(r) = current.as_mut() {
                    r.end = range.end;
                }
            }
            Tok::Bracket | Tok::Comment => {
                if let Some(r) = current.take() {
                    out.push(r);
                }
            }
        }
    }
    out.extend(current);
    out
}

fn line_col(text: &str, offset: usize) -> (u32, u32) {
    let before = &text[..offset];
    let line = before.matches('\n').count();
    let col = before.len() - before.rfind('\n').map_or(0, |i| i + 1);
    (line as u32, col as u32)
}

fn big_program(lines: usize) -> String {
    let mut s = String::from("; A generated program for timing\n");
    let mut n = 0;
    while s.lines().count() < lines {
        n += 1;
        s.push_str(&format!(
            "TO SHAPE{n} :size\n  REPEAT {} [FD :size RT {}] ; side {n}\n  SETPENCOLOR \"gold\nEND\n",
            3 + n % 5,
            360 / (3 + n % 5)
        ));
    }
    s
}

// ---------------------------------------------------------------- an editor highlighter for Logo

thread_local! {
    /// Time spent re-lexing in `LogoHighlighter::update`, read by the app.
    static HIGHLIGHTER_COST: RefCell<Samples> = RefCell::new(Samples::default());
}

/// Plugs our lexer into the editor as its syntax highlighter. Re-lexes the
/// whole buffer on every edit, which is the simplest thing that could work.
#[derive(Default)]
struct LogoHighlighter {
    runs: Vec<(Range<usize>, &'static str)>,
}

impl InputHighlighter for LogoHighlighter {
    fn language(&self) -> SharedString {
        "logo".into()
    }

    fn update(
        &mut self,
        _edit: Option<InputEdit>,
        text: &Rope,
        _folding: bool,
        _window: &mut Window,
        _cx: &mut Context<EditorState>,
    ) {
        let started = Instant::now();
        let text = text.to_string();
        self.runs = lex(&text)
            .into_iter()
            .filter_map(|(range, tok)| {
                let name = match tok {
                    Tok::Comment => "comment",
                    Tok::Number => "number",
                    Tok::Variable => "variable",
                    Tok::Quoted => "string",
                    Tok::Command => "keyword",
                    Tok::Bracket => "punctuation.bracket",
                    Tok::Operator => "operator",
                    Tok::Word => "function",
                };
                Some((range, name))
            })
            .collect();
        HIGHLIGHTER_COST.with(|c| c.borrow_mut().push(started.elapsed()));
    }

    fn styles(
        &self,
        range: &Range<usize>,
        resolver: &dyn HighlightStyleResolver,
    ) -> Vec<(Range<usize>, HighlightStyle)> {
        let mut out = Vec::new();
        let mut at = range.start;
        let first = self.runs.partition_point(|(r, _)| r.end <= range.start);
        for (r, name) in &self.runs[first..] {
            if r.start >= range.end {
                break;
            }
            let start = r.start.max(range.start);
            let end = r.end.min(range.end);
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

    fn fold_ranges(&self, _text: &Rope) -> Vec<FoldRange> {
        Vec::new()
    }
}

// ---------------------------------------------------------------- timing helpers

#[derive(Default)]
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
        let mean = v.iter().map(|d| ms(*d)).sum::<f64>() / v.len() as f64;
        format!(
            "n={} mean={:.2}ms p50={:.2}ms p95={:.2}ms max={:.2}ms",
            v.len(),
            mean,
            ms(v[v.len() / 2]),
            ms(v[v.len() * 95 / 100]),
            ms(v[v.len() - 1])
        )
    }
}

// ---------------------------------------------------------------- the app

/// One phase of the scripted session: what changes each frame.
struct Phase {
    name: &'static str,
    secs: u64,
    canvas: bool,
    highlight: bool,
    follow: bool,
    typing: bool,
    recolour: bool,
    screenshot: bool,
}

const PHASES: &[Phase] = &[
    Phase { name: "sample program, nothing changing", secs: 2, canvas: false, highlight: false, follow: false, typing: false, recolour: true, screenshot: true },
    Phase { name: "idle: request a frame, nothing changes", secs: 3, canvas: false, highlight: false, follow: false, typing: false, recolour: true, screenshot: false },
    Phase { name: "canvas only: turtle moves each frame", secs: 3, canvas: true, highlight: false, follow: false, typing: false, recolour: true, screenshot: false },
    Phase { name: "canvas + highlight, no scrolling", secs: 4, canvas: true, highlight: true, follow: false, typing: false, recolour: true, screenshot: false },
    Phase { name: "canvas + highlight, following it", secs: 6, canvas: true, highlight: true, follow: true, typing: false, recolour: true, screenshot: true },
    Phase { name: "typing a character per frame, no recolour", secs: 4, canvas: false, highlight: false, follow: false, typing: true, recolour: false, screenshot: false },
    Phase { name: "typing a character per frame, recolouring", secs: 4, canvas: false, highlight: false, follow: false, typing: true, recolour: true, screenshot: true },
    Phase { name: "finished", secs: 1, canvas: false, highlight: false, follow: false, typing: false, recolour: true, screenshot: false },
];

struct Spike {
    editor: Entity<EditorState>,
    highlight: RangeDecorationCollection,
    instructions: Vec<Range<usize>>,
    step: usize,
    canvas_step: usize,
    animating: bool,
    recolour_enabled: bool,
    typed: usize,
    follow: bool,
    log: Vec<SharedString>,
    last_frame: Option<Instant>,
    frame_intervals: Samples,
    highlight_cost: Samples,
    recolour_cost: Samples,
    /// The current phase of the scripted session, if running one.
    auto: Option<usize>,
    phase_started: Instant,
    screenshots: usize,
    _subscriptions: Vec<Subscription>,
}

impl Spike {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("logo")
                .line_number(true)
                .soft_wrap(false)
                .default_value(SAMPLE)
        });
        let highlight = editor.update(cx, |state, cx| {
            state.set_highlighter_factory(
                Rc::new(|language| {
                    (language == "logo")
                        .then(|| Box::new(LogoHighlighter::default()) as Box<dyn InputHighlighter>)
                }),
                cx,
            );
            state.create_range_decorations_collection(vec![], cx)
        });
        let subscriptions = vec![cx.subscribe(&editor, |this, _, event: &InputEvent, cx| {
            if let InputEvent::Change = event
                && this.recolour_enabled
            {
                this.recolour(cx);
            }
        })];
        let auto = std::env::var_os("SPIKE_AUTOTEST").map(|_| 0);
        let mut this = Self {
            editor,
            highlight,
            instructions: vec![],
            step: 0,
            canvas_step: 0,
            animating: false,
            recolour_enabled: true,
            typed: 0,
            follow: true,
            log: vec![],
            last_frame: None,
            frame_intervals: Samples::default(),
            highlight_cost: Samples::default(),
            recolour_cost: Samples::default(),
            auto,
            phase_started: Instant::now(),
            screenshots: 0,
            _subscriptions: subscriptions,
        };
        this.recolour(cx);
        this
    }

    fn say(&mut self, msg: impl Into<SharedString>) {
        let msg = msg.into();
        println!("{msg}");
        self.log.push(msg);
    }

    /// Re-lex the whole buffer for diagnostics and instruction spans.
    /// (Syntax colours come from `LogoHighlighter`.)
    fn recolour(&mut self, cx: &mut Context<Self>) {
        let started = Instant::now();
        let text = self.editor.read(cx).value().to_string();
        let tokens = lex(&text);

        // Unknown words that aren't defined with TO get a diagnostic.
        let defined: Vec<String> = tokens
            .windows(2)
            .filter(|w| text[w[0].0.clone()].eq_ignore_ascii_case("to"))
            .map(|w| text[w[1].0.clone()].to_lowercase())
            .collect();
        let diagnostics: Vec<Diagnostic> = tokens
            .iter()
            .filter(|(r, t)| *t == Tok::Word && !defined.contains(&text[r.clone()].to_lowercase()))
            .map(|(r, _)| {
                let (l0, c0) = line_col(&text, r.start);
                let (l1, c1) = line_col(&text, r.end);
                Diagnostic::new(
                    Position::new(l0, c0)..Position::new(l1, c1),
                    format!("I don't know how to {}", &text[r.clone()]),
                )
                .with_severity(DiagnosticSeverity::Error)
            })
            .collect();
        self.editor.update(cx, |state, cx| {
            // The set converts positions with its own copy of the text, so it
            // must be reset to the current text first.
            let text = state.text().clone();
            if let Some(set) = state.diagnostics_mut() {
                set.reset(&text);
                set.extend(diagnostics);
                if std::env::var_os("SPIKE_DEBUG").is_some() {
                    for e in set.iter() {
                        println!("diagnostic {:?}: {}", e.range, e.diagnostic.message);
                    }
                }
            }
            cx.notify();
        });

        self.instructions = instructions(&tokens);
        self.recolour_cost.push(started.elapsed());
    }

    fn load_big(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let program = big_program(500);
        self.editor
            .update(cx, |state, cx| state.set_value(program, window, cx));
        self.recolour(cx);
        self.recolour_cost = Samples::default();
        self.say(format!(
            "Loaded 500-line program: {} instructions",
            self.instructions.len()
        ));
    }

    /// Move the highlight to the next instruction, scrolling to keep it visible.
    fn advance_highlight(&mut self, cx: &mut Context<Self>) {
        if self.instructions.is_empty() {
            return;
        }
        let started = Instant::now();
        self.step = (self.step + 1) % self.instructions.len();
        let range = self.instructions[self.step].clone();
        self.highlight.set(
            vec![
                RangeDecoration::new(range.clone())
                    .with_style(RangeDecorationStyle::Fill)
                    .with_color(cx.theme().warning.opacity(0.35)),
            ],
            cx,
        );
        if self.follow {
            let text = self.editor.read(cx).value().to_string();
            let (line, _) = line_col(&text, range.start);
            let debug = self.step % 120 == 0;
            let msg = self.editor.update(cx, |state, cx| {
                let (Some(rows), Some(line_height)) = (state.visible_row_range(), state.line_height())
                else {
                    return Some("follow: no layout yet".to_string());
                };
                let line = line as usize;
                let offset = state.scroll_offset();
                // Keep a margin of 3 lines above and below the highlight.
                let scroll = line < rows.start + 3 || line + 3 >= rows.end;
                if scroll {
                    let top = line.saturating_sub(3) as f32;
                    state.set_scroll_offset(point(offset.x, -line_height * top), cx);
                }
                debug.then(|| {
                    format!(
                        "follow: line {line}, visible rows {rows:?}, line height {line_height:?}, offset y {:?}, scroll {scroll}",
                        offset.y
                    )
                })
            });
            if let Some(msg) = msg {
                self.say(msg);
            }
        }
        self.highlight_cost.push(started.elapsed());
    }

    fn reset_samples(&mut self) {
        self.frame_intervals = Samples::default();
        self.highlight_cost = Samples::default();
        self.recolour_cost = Samples::default();
        HIGHLIGHTER_COST.with(|c| *c.borrow_mut() = Samples::default());
        self.last_frame = None;
    }

    fn report(&mut self) {
        let frames = self.frame_intervals.summary();
        let highlight = self.highlight_cost.summary();
        let recolour = self.recolour_cost.summary();
        let highlighter = HIGHLIGHTER_COST.with(|c| c.borrow().summary());
        self.say(format!("  Frame intervals: {frames}"));
        self.say(format!("  Highlight update: {highlight}"));
        self.say(format!("  Highlighter re-lex: {highlighter}"));
        self.say(format!("  Diagnostics + instruction spans: {recolour}"));
    }

    fn screenshot(&mut self, name: &str) {
        self.screenshots += 1;
        let path = format!("screenshot-{}-{name}.png", self.screenshots);
        // xwd + ImageMagick: capture just this window by its title.
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "xwd -silent -name 'Pendown spike 01' | convert xwd:- '{path}'"
            ))
            .status();
        self.say(format!("Screenshot {path}: {:?}", status.map(|s| s.success())));
    }

    /// The scripted session: run each phase in `PHASES` for its time, then report.
    fn drive_autotest(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(ix) = self.auto else { return };
        let phase = &PHASES[ix];
        if self.phase_started.elapsed() < Duration::from_secs(phase.secs) {
            return;
        }
        self.say(format!("--- {}:", phase.name));
        self.report();
        if phase.screenshot {
            self.screenshot(&phase.name.split([' ', ',', ':']).next().unwrap_or("phase").to_string());
        }
        if ix == 0 {
            self.load_big(window, cx);
        }
        self.reset_samples();
        self.phase_started = Instant::now();
        if ix + 1 == PHASES.len() {
            self.auto = None;
            cx.quit();
            return;
        }
        self.auto = Some(ix + 1);
        let next = &PHASES[ix + 1];
        self.animating = next.highlight;
        self.follow = next.follow;
        self.recolour_enabled = next.recolour;
        if next.typing {
            // Type inside the procedure body on line 3, where it stays visible.
            self.editor.update(cx, |state, cx| {
                state.set_scroll_offset(point(px(0.), px(0.)), cx);
                state.set_cursor_position(Position::new(2, 2), window, cx)
            });
        }
    }

    fn type_a_character(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        const TEXT: &str = "FD 10 RT 5 ";
        self.typed += 1;
        let ch = if self.typed % 40 == 0 {
            "\n  ".to_string()
        } else {
            TEXT[self.typed % TEXT.len()..][..1].to_string()
        };
        self.editor
            .update(cx, |state, cx| state.insert(ch, window, cx));
    }
}

impl Render for Spike {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let now = Instant::now();
        let phase = self.auto.map(|ix| &PHASES[ix]);
        let busy = self.animating || phase.is_some();
        if busy {
            if let Some(last) = self.last_frame {
                self.frame_intervals.push(now - last);
            }
            self.last_frame = Some(now);
        } else {
            self.last_frame = None;
        }
        if phase.is_some_and(|p| p.canvas) || (phase.is_none() && self.animating) {
            self.canvas_step += 1;
        }
        if self.animating {
            self.advance_highlight(cx);
        }
        if phase.is_some_and(|p| p.typing) {
            self.type_a_character(window, cx);
        }
        if phase.is_some() {
            self.drive_autotest(window, cx);
        }
        if busy {
            window.request_animation_frame();
        }

        let theme = cx.theme().clone();
        let step = self.canvas_step;

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
                        Button::new("load")
                            .small()
                            .label("Load 500 lines")
                            .on_click(cx.listener(|this, _, window, cx| this.load_big(window, cx))),
                    )
                    .child(
                        Button::new("animate")
                            .small()
                            .when(self.animating, |b| b.primary())
                            .label(if self.animating { "Stop highlight" } else { "Animate highlight" })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.animating = !this.animating;
                                if !this.animating {
                                    this.report();
                                    this.frame_intervals = Samples::default();
                                    this.highlight_cost = Samples::default();
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("follow")
                            .small()
                            .when(self.follow, |b| b.primary())
                            .label("Follow highlight")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.follow = !this.follow;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("report")
                            .small()
                            .label("Report timings")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.report();
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div().flex_1().min_h_0().child(
                    h_resizable("main")
                        .child(
                            resizable_panel().size(px(520.)).child(
                                Editor::new(&self.editor)
                                    .bordered(false)
                                    .h_full()
                                    .font_family(theme.mono_font_family.clone())
                                    .text_size(theme.mono_font_size),
                            ),
                        )
                        .child(resizable_panel().child(turtle_canvas(step, &theme))),
                ),
            )
            .child(
                div()
                    .id("console")
                    .h(px(140.))
                    .p_2()
                    .border_t_1()
                    .border_color(theme.border)
                    .overflow_y_scroll()
                    .font_family(theme.mono_font_family.clone())
                    .text_xs()
                    .children(self.log.iter().rev().take(50).cloned().map(|l| div().child(l))),
            )
    }
}

/// A turtle drawing on a raw gpui canvas: a square spiral, with the turtle
/// partway along it so the canvas repaints with the highlight.
fn turtle_canvas(step: usize, theme: &gpui_kit::component::Theme) -> impl IntoElement {
    let bg = theme.background;
    let ink = theme.primary;
    let turtle = theme.success;
    canvas(
        move |bounds, _, _| bounds,
        move |_, bounds, window, _| {
            window.paint_quad(fill(bounds, bg));
            let centre = bounds.center();
            let mut pos = centre;
            let mut heading: f32 = 0.0;
            let mut path = PathBuilder::stroke(px(1.5));
            path.move_to(pos);
            let segments = 400;
            let visible = step % segments + 1;
            for i in 0..visible {
                let len = 2.0 + i as f32 * 0.9;
                let (s, c) = heading.to_radians().sin_cos();
                pos = point(pos.x + px(len * s), pos.y - px(len * c));
                path.line_to(pos);
                heading += 89.0;
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, ink);
            }
            // The turtle: a triangle pointing along its heading.
            let (s, c) = heading.to_radians().sin_cos();
            let tip = point(pos.x + px(12.0 * s), pos.y - px(12.0 * c));
            let left = point(pos.x - px(6.0 * c), pos.y - px(6.0 * s));
            let right = point(pos.x + px(6.0 * c), pos.y + px(6.0 * s));
            let mut tri = PathBuilder::fill();
            tri.move_to(tip);
            tri.line_to(left);
            tri.line_to(right);
            tri.close();
            if let Ok(tri) = tri.build() {
                window.paint_path(tri, turtle);
            }
        },
    )
    .size_full()
}

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("Pendown spike 01".into()),
                ..Default::default()
            }),
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1200.), px(800.)),
                cx,
            ))),
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, |window, cx| cx.new(|cx| Spike::new(window, cx)))
            .expect("failed to open window");
        cx.activate(true);
    });
}
