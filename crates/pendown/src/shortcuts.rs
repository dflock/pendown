//! The keyboard shortcuts (`.scratch/shortcuts/`, ADR 0031): one table of
//! what each key does. `bind_keys` binds the app's keys from it, and the
//! Shortcuts tab shows it, asking the keymap for each action's keys as it
//! draws, so the tab shows what the keys really are.

use gpui_kit::{base::input, component::kbd::Kbd, *};

use crate::{driving::DRIVE_KEYS, files::*};

/// The editor binds its keys in this key context.
const EDITOR: &str = "Input";

/// Where a row's keys come from.
pub enum Keys {
    /// One of the app's actions, and the keys `bind_keys` binds to it, in
    /// gpui's notation. The tab shows them all, from the keymap.
    App(fn() -> Box<dyn Action>, &'static [&'static str]),
    /// The editor's actions, whose keys the editor binds itself. The tab
    /// shows each one's key, from the keymap, side by side.
    Editor(Vec<Box<dyn Action>>),
    /// Keys that aren't bindings in the keymap, written out: the console's,
    /// and the drive keys. Side by side, or `or` between them if `either`.
    Written {
        keys: &'static [&'static str],
        either: bool,
    },
}

pub struct Row {
    pub what: &'static str,
    pub keys: Keys,
}

pub struct Group {
    pub title: &'static str,
    /// When the keys work, if not always.
    pub when: Option<&'static str>,
    pub rows: Vec<Row>,
}

fn app(what: &'static str, action: fn() -> Box<dyn Action>, keys: &'static [&'static str]) -> Row {
    Row {
        what,
        keys: Keys::App(action, keys),
    }
}

fn editor(what: &'static str, actions: Vec<Box<dyn Action>>) -> Row {
    Row {
        what,
        keys: Keys::Editor(actions),
    }
}

fn written(what: &'static str, keys: &'static [&'static str]) -> Row {
    Row {
        what,
        keys: Keys::Written {
            keys,
            either: false,
        },
    }
}

/// Every shortcut, in the groups the tab shows. The editor binds dozens of
/// keys; these are the ones worth knowing.
pub fn groups() -> Vec<Group> {
    vec![
        Group {
            title: "Your program",
            when: None,
            rows: vec![
                app(
                    "Run your program",
                    || Box::new(RunProgram),
                    &["secondary-enter", "f5"],
                ),
                app(
                    "Pause, or carry on when paused",
                    || Box::new(PauseOrResume),
                    &["f6"],
                ),
                app(
                    "Single step: one instruction, then pause",
                    || Box::new(SingleStep),
                    &["f10"],
                ),
                app(
                    "Reset: stop, and put everything back",
                    || Box::new(ResetProgram),
                    &["shift-f5"],
                ),
                app(
                    "Help for the word under the cursor",
                    || Box::new(ShowHelp),
                    &["f1"],
                ),
            ],
        },
        Group {
            title: "Files",
            when: None,
            rows: vec![
                app("New program", || Box::new(NewFile), &["secondary-n"]),
                app("Open a program", || Box::new(OpenFile), &["secondary-o"]),
                app("Save", || Box::new(SaveFile), &["secondary-s"]),
                app(
                    "Save as a new file",
                    || Box::new(SaveFileAs),
                    &["secondary-shift-s"],
                ),
                app(
                    "Export the picture",
                    || Box::new(ExportPicture),
                    &["secondary-e"],
                ),
            ],
        },
        Group {
            title: "The editor",
            when: None,
            rows: vec![
                editor("Undo", vec![Box::new(input::Undo)]),
                editor("Redo", vec![Box::new(input::Redo)]),
                editor(
                    "Cut, copy and paste",
                    vec![
                        Box::new(input::Cut),
                        Box::new(input::Copy),
                        Box::new(input::Paste),
                    ],
                ),
                editor("Select everything", vec![Box::new(input::SelectAll)]),
                editor(
                    "Indent and outdent",
                    vec![
                        Box::new(input::IndentInline),
                        Box::new(input::OutdentInline),
                    ],
                ),
                editor(
                    "Move a word at a time",
                    vec![
                        Box::new(input::MoveToPreviousWord),
                        Box::new(input::MoveToNextWord),
                    ],
                ),
                editor(
                    "Go to the start or the end",
                    vec![Box::new(input::MoveToStart), Box::new(input::MoveToEnd)],
                ),
            ],
        },
        Group {
            title: "The console",
            when: None,
            rows: vec![
                written("Run what you typed", &["enter"]),
                written("Bring back what you typed before", &["up", "down"]),
            ],
        },
        Group {
            title: "Driving Kame",
            when: Some("while driving"),
            rows: DRIVE_KEYS
                .iter()
                .map(|(_, keys, what)| Row {
                    what,
                    keys: Keys::Written { keys, either: true },
                })
                .collect(),
        },
        Group {
            title: "For the curious",
            when: None,
            rows: vec![app(
                "How fast Pendown is drawing",
                || Box::new(ToggleMetrics),
                &["f12"],
            )],
        },
    ]
}

/// Binds the app's keys from the table: Ctrl on Linux and Windows, Cmd on
/// macOS, for `secondary-`.
pub fn bind_keys(cx: &mut gpui_kit::App) {
    let mut bindings = Vec::new();
    for row in groups().into_iter().flat_map(|g| g.rows) {
        if let Keys::App(action, keys) = row.keys {
            // `secondary-` needs the host OS, so these go through `load`.
            // The first key last, so it has the highest precedence, and
            // tooltips show it.
            for key in keys.iter().rev() {
                bindings.push(
                    KeyBinding::load(key, action(), None, false, None, cx).expect("a key binding"),
                );
            }
        }
    }
    cx.bind_keys(bindings);
}

/// A row's keys, with whether to put `or` between them (several keys for
/// one thing) rather than show them side by side (one key each for
/// several things). Missing any the keymap has no key for.
pub fn keystrokes(row: &Row, window: &Window, cx: &gpui_kit::App) -> (Vec<Keystroke>, bool) {
    let first = |binding: &KeyBinding| {
        binding
            .keystrokes()
            .first()
            .map(|k| k.as_keystroke().clone())
    };
    match &row.keys {
        Keys::App(action, _) => {
            // The keymap's keys, in the table's order: the first is the
            // one people know.
            let mut bound = window.bindings_for_action(action().as_ref());
            bound.reverse();
            (bound.iter().filter_map(first).collect(), true)
        }
        Keys::Editor(actions) => {
            let context = KeyContext::parse(EDITOR).expect("a key context");
            let keys = actions
                .iter()
                .filter_map(|action| {
                    window
                        .highest_precedence_binding_for_action_in_context(
                            action.as_ref(),
                            context.clone(),
                        )
                        .as_ref()
                        .and_then(first)
                })
                .collect();
            (keys, false)
        }
        Keys::Written { keys, either } => (
            keys.iter()
                .map(|k| crate::driving::keystroke(k, cx))
                .collect(),
            *either,
        ),
    }
}

/// A keystroke as it's shown: the arrow keys as arrows, which Mac
/// keyboards use and are shorter than "Left" and "Right" elsewhere.
pub fn shown(keystroke: &Keystroke) -> Keystroke {
    let arrow = match keystroke.key.as_str() {
        "up" => "↑",
        "down" => "↓",
        "left" => "←",
        "right" => "→",
        _ => return keystroke.clone(),
    };
    Keystroke {
        key: arrow.into(),
        ..keystroke.clone()
    }
}

/// A keystroke's label, as this computer writes it: `Ctrl+Enter`, or `⌘↩`
/// on a Mac.
pub fn label(keystroke: &Keystroke) -> String {
    Kbd::format(&shown(keystroke))
}

/// A row's keys' labels.
pub fn labels(row: &Row, window: &Window, cx: &gpui_kit::App) -> Vec<String> {
    keystrokes(row, window, cx).0.iter().map(label).collect()
}

/// How many keys a row should show: one per action for the editor, at
/// least one otherwise. For the check that every row has its keys.
pub fn expected(row: &Row) -> usize {
    match &row.keys {
        Keys::App(_, keys) => keys.len(),
        Keys::Editor(actions) => actions.len(),
        Keys::Written { keys, .. } => keys.len(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    // Not `super::*`, which brings gpui's `test` over the standard one.
    use super::{DRIVE_KEYS, Keys, expected, groups};

    #[test]
    fn no_key_is_in_twice() {
        let mut seen = HashSet::new();
        for row in groups().iter().flat_map(|g| &g.rows) {
            if let Keys::App(_, keys) = row.keys {
                for key in keys {
                    assert!(seen.insert(*key), "{key} is bound twice");
                }
            }
        }
        let mut seen = HashSet::new();
        for (_, keys, what) in DRIVE_KEYS {
            for key in keys {
                assert!(
                    seen.insert(key),
                    "{key} drives twice, the second for {what}"
                );
            }
        }
    }

    #[test]
    fn every_row_says_what_it_does_and_has_keys() {
        for group in groups() {
            assert!(!group.rows.is_empty(), "{}", group.title);
            for row in &group.rows {
                assert!(!row.what.is_empty());
                assert!(expected(row) > 0, "{}", row.what);
            }
        }
    }
}
