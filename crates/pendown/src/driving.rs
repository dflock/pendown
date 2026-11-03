//! Drive Kame (ADR 0026): the arrow keys move and turn Kame by steps, each
//! press run as a console entry, presses of the same key in a row joining
//! into one, so Undo and Teach Kame work on them. Kame thinking is always
//! shown while driving, for the whole entry, with a line out in front of
//! Kame and what the entry means.

use std::time::Instant;

use gpui_kit::component::{ActiveTheme as _, h_flex};
use gpui_kit::{prelude::FluentBuilder as _, *};
use pendown::{
    animator::{FADE, Thinking, Thought},
    drive::{
        Input, Joined, Key, MOVE_STEPS, Press, Presses, TURN_STEPS, caption, number, pen, press,
    },
    run::Job,
};
use pendown_logo::Point;

use crate::app::App;

/// What a drive key does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DriveKey {
    Forward,
    Back,
    Left,
    Right,
    Pen,
    Undo,
    Stop,
}

/// The keys while driving, in gpui's notation, and what they do: the one
/// list, which `drive_key`, the drive strip and the Shortcuts tab all read
/// (`.scratch/shortcuts/`). They're handled here, not bound in the keymap,
/// as they only mean this while driving.
pub(crate) const DRIVE_KEYS: [(DriveKey, &[&str], &str); 7] = [
    (DriveKey::Forward, &["up"], "Forward"),
    (DriveKey::Back, &["down"], "Back"),
    (DriveKey::Left, &["left"], "Turn left"),
    (DriveKey::Right, &["right"], "Turn right"),
    (DriveKey::Pen, &["space"], "Pen up or down"),
    (
        DriveKey::Undo,
        &["backspace", "secondary-z"],
        "Undo the last move",
    ),
    (DriveKey::Stop, &["escape"], "Stop driving"),
];

/// The keystroke `keys` names, in gpui's notation, on this computer:
/// `secondary-z` is Ctrl+Z, or Cmd+Z on a Mac.
pub(crate) fn keystroke(keys: &str, cx: &gpui_kit::App) -> Keystroke {
    Keystroke::parse_with_os(keys, cx.platform_info().os).expect("a keystroke")
}

/// What `pressed` asks of driving, if it's a drive key. A key with no
/// modifiers in the table matches with any, so Shift+Up still drives.
fn drive_key_for(pressed: &Keystroke, cx: &gpui_kit::App) -> Option<DriveKey> {
    DRIVE_KEYS.iter().find_map(|(what, keys, _)| {
        keys.iter()
            .any(|k| {
                let k = keystroke(k, cx);
                k.key == pressed.key
                    && (k.modifiers == Modifiers::default() || k.modifiers == pressed.modifiers)
            })
            .then_some(*what)
    })
}

/// The label of a drive key's first key, as the Shortcuts tab shows it.
fn key_label(what: DriveKey, cx: &gpui_kit::App) -> String {
    let (_, keys, _) = DRIVE_KEYS
        .iter()
        .find(|(w, ..)| *w == what)
        .expect("every drive key is in the table");
    crate::shortcuts::label(&keystroke(keys[0], cx))
}

/// The entry presses are joining into.
pub(crate) struct Open {
    pub joined: Joined,
    /// Its line in the console, which grows with it.
    line: usize,
    /// Where Kame was, and its heading, when the entry started.
    start: Point,
    heading: f64,
    /// How far the presses before the one playing went.
    before: f64,
    /// What it last showed, to fade when it ends.
    last: Option<Thought>,
}

#[derive(Default)]
pub(crate) struct Driving {
    /// Whether Space has lifted the pen. Driving starts with it down.
    pub pen_up: bool,
    pub open: Option<Open>,
    /// The keys down, and the presses made while one plays.
    pub presses: Presses,
    /// The last entry's thought, fading, and when it started to.
    fading: Option<(Thought, Instant)>,
}

impl App {
    /// The toolbar's Drive, which gives the canvas the keys.
    pub(crate) fn toggle_driving(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.driving.is_some() {
            self.stop_driving(cx);
        } else if !self.running() {
            self.driving = Some(Driving::default());
            window.focus(&self.canvas_focus, cx);
        }
        cx.notify();
    }

    /// Esc, Run, Pause and Step stop driving.
    pub(crate) fn stop_driving(&mut self, cx: &mut Context<Self>) {
        if self.driving.take().is_some() {
            cx.notify();
        }
    }

    /// Where Kame is and which way it faces.
    fn kame(&self) -> (Point, f64) {
        self.animator
            .as_ref()
            .map_or((Point { x: 0.0, y: 0.0 }, 0.0), |a| (a.pos, a.heading))
    }

    /// Ends the entry presses are joining into: the next press starts a
    /// new one, and its thought fades.
    pub(crate) fn end_driven_entry(&mut self) {
        if let Some(driving) = &mut self.driving
            && let Some(open) = driving.open.take()
            && let Some(thought) = open.last
        {
            driving.fading = Some((thought, Instant::now()));
        }
    }

    /// Undo has taken back `text`: a driven entry ends, and taking back
    /// `PU` or `PD` puts the pen back as it was.
    pub(crate) fn driven_undo(&mut self, text: &str) {
        self.end_driven_entry();
        if let Some(driving) = &mut self.driving {
            match text {
                "PU" => driving.pen_up = false,
                "PD" => driving.pen_up = true,
                _ => {}
            }
        }
    }

    /// Plays a press, with none playing.
    fn drive(&mut self, input: Input, cx: &mut Context<Self>) {
        let Some(driving) = &mut self.driving else {
            return;
        };
        match input {
            Input::Undo => self.undo_console(cx),
            Input::Pen => {
                self.end_driven_entry();
                let driving = self.driving.as_mut().expect("driving");
                let text = pen(driving.pen_up);
                driving.pen_up = !driving.pen_up;
                self.enter(text, cx);
            }
            Input::Key(key) => {
                let open = driving.open.as_ref().map(|o| o.joined);
                match press(open, key, self.drive_steps) {
                    Press::New { joined, run } => {
                        self.end_driven_entry();
                        let (start, heading) = self.kame();
                        self.enter(&run, cx);
                        let line = self.console.len().saturating_sub(1);
                        if let Some(driving) = &mut self.driving {
                            driving.open = Some(Open {
                                joined,
                                line,
                                start,
                                heading,
                                before: 0.0,
                                last: None,
                            });
                        }
                    }
                    Press::More { joined, run } => {
                        let open = driving.open.as_mut().expect("joining");
                        let old = format!("> {}", open.joined.text());
                        open.before = open.joined.total;
                        open.joined = joined;
                        let line = open.line;
                        let text = joined.text();
                        self.console_entry.amend_last(&text);
                        if self.console.get(line).is_some_and(|l| l.as_ref() == old) {
                            self.console[line] = format!("> {text}").into();
                            self.console_dirty = true;
                        }
                        // Carries on the entry: not an entry of its own, so
                        // it saves no Kame for Undo (ADR 0025).
                        let left = self.animator.as_ref().map(|a| (a.pos, a.heading, a.shown));
                        self.start_job(Job::Define(run), pendown::run::DEFAULT_CHANNEL, cx);
                        self.put_kame(left);
                    }
                }
            }
        }
        cx.notify();
    }

    /// Runs `text` as a console entry, as if typed and entered.
    fn enter(&mut self, text: &str, cx: &mut Context<Self>) {
        self.print(format!("> {text}"));
        self.console_entry.submit(text);
        self.run_console(text.to_string(), cx);
    }

    /// The next press waiting, once the last has played.
    pub(crate) fn drive_next(&mut self, cx: &mut Context<Self>) {
        if self.running() {
            return;
        }
        if let Some(input) = self.driving.as_mut().and_then(|d| d.presses.next()) {
            self.drive(input, cx);
        }
    }

    /// What Kame is thinking about while driving: the whole entry so far,
    /// from where it started, and the last entry fading.
    pub(crate) fn drive_thinking(&mut self, now: Thinking) -> Thinking {
        let (pos, _) = self.kame();
        let Some(driving) = &mut self.driving else {
            return now;
        };
        let current = match &mut driving.open {
            Some(open) => {
                let sign = if open.joined.key == Key::Left {
                    -1.0
                } else {
                    1.0
                };
                let thought = match (now.current, open.joined.key.turns()) {
                    (Some(Thought::Turn { by, done, .. }), true) => Some(Thought::Turn {
                        at: open.start,
                        from: open.heading,
                        by: sign * open.before + by,
                        done,
                    }),
                    (
                        Some(Thought::Move {
                            to,
                            length,
                            heading,
                            done,
                            ..
                        }),
                        false,
                    ) => Some(Thought::Move {
                        from: open.start,
                        to,
                        length: open.before + length,
                        heading,
                        done,
                    }),
                    // Between presses: what the entry has done so far.
                    (_, true) if open.before > 0.0 => Some(Thought::Turn {
                        at: open.start,
                        from: open.heading,
                        by: sign * open.before,
                        done: true,
                    }),
                    (_, false) if open.before > 0.0 => Some(Thought::Move {
                        from: open.start,
                        to: pos,
                        length: open.before,
                        heading: open.heading,
                        done: true,
                    }),
                    _ => open.last,
                };
                open.last = thought.or(open.last);
                open.last
            }
            None => now.current,
        };
        let fading = driving.fading.and_then(|(thought, since)| {
            let left = 1.0 - since.elapsed().as_secs_f64() / FADE;
            (left > 0.0).then_some((thought, left))
        });
        if fading.is_none() {
            driving.fading = None;
        }
        Thinking { current, fading }
    }

    /// For the scripted session: what Kame is thinking about while
    /// driving, in words: `turn 15`, or just `turn from 90` while it's still
    /// starting, with `, fading move 10` for the last entry fading.
    pub(crate) fn thinking_described(&mut self) -> String {
        let now = self
            .animator
            .as_ref()
            .map(|a| a.thinking())
            .unwrap_or_default();
        let thinking = self.drive_thinking(now);
        let words = |thought: Thought| match thought {
            Thought::Turn { by, done: true, .. } => format!("turn {}", number(by.abs().round())),
            Thought::Turn { from, .. } => format!("turn from {}", number(from.round())),
            Thought::Move {
                length, done: true, ..
            } => format!("move {}", number(length.round())),
            Thought::Move { .. } => "move".to_string(),
        };
        let mut text = thinking.current.map_or("nothing".to_string(), words);
        if let Some((fading, _)) = thinking.fading {
            text.push_str(&format!(", fading {}", words(fading)));
        }
        text
    }

    /// Whether the last entry's thought is still fading, which needs frames.
    pub(crate) fn drive_fading(&self) -> bool {
        self.driving.as_ref().is_some_and(|d| d.fading.is_some())
    }

    /// What driving says under Kame: the entry, and Kame's heading.
    pub(crate) fn drive_caption(&self) -> Option<String> {
        let open = self.driving.as_ref()?.open.as_ref()?;
        Some(caption(open.joined, self.kame().1))
    }

    /// A key on the canvas. Only while driving: true if it was a driving key.
    pub(crate) fn drive_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.drive_keystroke(&event.keystroke, window, cx)
    }

    /// `drive_key` for a keystroke, which the scripted session presses.
    pub(crate) fn drive_keystroke(
        &mut self,
        keystroke: &Keystroke,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.driving.is_none() {
            return false;
        }
        let input = match drive_key_for(keystroke, cx) {
            Some(DriveKey::Forward) => Input::Key(Key::Forward),
            Some(DriveKey::Back) => Input::Key(Key::Back),
            Some(DriveKey::Left) => Input::Key(Key::Left),
            Some(DriveKey::Right) => Input::Key(Key::Right),
            Some(DriveKey::Pen) => Input::Pen,
            Some(DriveKey::Undo) => Input::Undo,
            Some(DriveKey::Stop) => {
                self.stop_driving(cx);
                return true;
            }
            None => return false,
        };
        let running = self.running();
        let play = self
            .driving
            .as_mut()
            .and_then(|d| d.presses.down(&keystroke.key, input, running));
        if let Some(input) = play {
            self.drive(input, cx);
        }
        window.focus(&self.canvas_focus, cx);
        true
    }

    /// A key coming up on the canvas, which stops it repeating.
    pub(crate) fn drive_key_up(&mut self, keystroke: &Keystroke) {
        if let Some(driving) = &mut self.driving {
            driving.presses.up(&keystroke.key);
        }
    }

    /// The strip across the top of the canvas while driving: the keys, the
    /// steps, and whether the pen is up.
    pub(crate) fn drive_strip(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let driving = self.driving.as_ref()?;
        let theme = cx.theme().clone();
        let accent: Hsla = rgb(0xfde68a).into();
        let key = |label: String| {
            div()
                .px_1()
                .min_w(px(20.))
                .h(px(20.))
                .flex()
                .items_center()
                .justify_center()
                .rounded_sm()
                .border_1()
                .border_b_2()
                .border_color(white().opacity(0.35))
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .child(label)
        };
        let pair = |a: String, b: String, what: (&'static str, &'static str)| {
            h_flex()
                .gap_1()
                .child(key(a))
                .child(what.0)
                .child(key(b))
                .child(what.1)
        };
        let steps = |id: &'static str, choices: &[f64], chosen: f64, turns: bool| {
            h_flex()
                .rounded_md()
                .border_1()
                .border_color(white().opacity(0.3))
                .overflow_hidden()
                .children(choices.iter().map(|&n| {
                    let selected = n == chosen;
                    div()
                        .id(SharedString::from(format!("{id}-{n}")))
                        .px_1p5()
                        .h(px(22.))
                        .flex()
                        .items_center()
                        .text_xs()
                        .font_family(theme.mono_font_family.clone())
                        .cursor_pointer()
                        .when(selected, |d| d.bg(accent).text_color(black()))
                        .child(number(n))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            if turns {
                                this.drive_steps.turns = n;
                            } else {
                                this.drive_steps.moves = n;
                            }
                            window.focus(&this.canvas_focus, cx);
                            cx.notify();
                        }))
                }))
        };
        Some(
            h_flex()
                .absolute()
                .top_3()
                .left_3()
                .right_3()
                .flex_wrap()
                .gap_x_3()
                .gap_y_2()
                .px_3()
                .py_2()
                .rounded_lg()
                .bg(rgb(0x18181b).opacity(0.88))
                .text_color(rgb(0xe4e4e7))
                .text_xs()
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(accent)
                        .child("Driving"),
                )
                .child(pair(
                    key_label(DriveKey::Forward, cx),
                    key_label(DriveKey::Back, cx),
                    ("FD", "BK"),
                ))
                .child(steps(
                    "move-step",
                    &MOVE_STEPS,
                    self.drive_steps.moves,
                    false,
                ))
                .child(pair(
                    key_label(DriveKey::Left, cx),
                    key_label(DriveKey::Right, cx),
                    ("LT", "RT"),
                ))
                .child(steps(
                    "turn-step",
                    &TURN_STEPS,
                    self.drive_steps.turns,
                    true,
                ))
                .child(
                    h_flex()
                        .gap_1()
                        .child(key(key_label(DriveKey::Pen, cx)))
                        .child("Pen"),
                )
                .when(driving.pen_up, |d| {
                    d.child(
                        div()
                            .px_2()
                            .rounded_full()
                            .bg(accent)
                            .text_color(black())
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Pen up"),
                    )
                })
                .child(
                    h_flex()
                        .gap_1()
                        .child(key(key_label(DriveKey::Undo, cx)))
                        .child("Undo"),
                )
                .child(
                    h_flex()
                        .gap_1()
                        .child(key(key_label(DriveKey::Stop, cx)))
                        .child("Stop driving"),
                )
                .into_any_element(),
        )
    }
}
