# The table, and binding keys from it

Status: resolved
Type: task

## What

A `shortcuts` module in `pendown` with the groups and rows of the spec: each row an action and a description, or (for the console) keys as text and a description.

- `files::bind_keys` binds the app's rows from the table, with their keys (`secondary-enter`, `f5`, `secondary-s`...). The keys live in the table beside the action, so they're in one place.
- F5 runs the program, as Ctrl+Enter does.
- A function that gives a row's keys from the keymap, for the tab and the tooltips, in the editor's `"Input"` context for its rows.

## Done when

- Unit tests: every action row has at least one binding; every binding made by `bind_keys` is in the table.
- F5 runs the program (a scripted-session check, in issue 04).

## Comments

2026-11-01: Done: `shortcuts.rs`, with `groups()` and `bind_keys`, which replaces `files::bind_keys`. F5 runs. The first key is bound last, so it has the highest precedence and tooltips show it; the tab reverses the keymap's order to match the table. Unit tests: no key bound twice, every row says what it does and has keys. The check that every row's keys are in the keymap needs a window, so it's in the scripted session (`CheckShortcutsBound`). Find and Replace are left out: the editor only handles them when searchable, and ours isn't.
