# 31. Shortcuts are shown from the keymap, with one table of what they do

Status: Accepted
Date: 2026-11-01

## Context

Pendown had keyboard shortcuts that nothing told you about: Run has had Ctrl+Enter since phase 1. A page listing them would be easy to write and quick to go out of date: change a binding, or the editor component's, and the page is wrong.

## Decision

- **A Shortcuts tab** in the side panel, after Gallery, in groups: your program, files, the editor, the console, driving Kame, and F12.
- **One table, `shortcuts.rs`,** of rows: what a key does, and where its keys come from. `bind_keys` binds the app's keys from it, so a binding and its description are one entry.
- **The tab asks the keymap** for each action's keys as it draws (`Window::bindings_for_action`), not the table's text. The editor's rows name the editor's own actions, and look them up in its `"Input"` key context, so if the editor component changes a key the tab follows, and if it drops an action the build breaks.
- **The drive keys are one table in `driving.rs`,** which `drive_key`, the drive strip and the tab read. They aren't in the keymap, as they only mean this while driving, so that table is their one list. The console's Enter and Up and Down are written in the table: they're how a text line behaves, not bindings.
- **Keys as this computer writes them,** with gpui-component's `Kbd`: Ctrl+Enter, or ⌘↩ on a Mac. Arrow keys are shown as arrows everywhere.
- **Tooltips show their keys,** from the same keymap: Run, New, Open, Save and Export.
- **F5 runs too,** as in many editors. Ctrl+Enter, the first key, is the one tooltips show.
- **The debugger's keys, as in VS Code:** F6 pauses and carries on, F10 single steps, Shift+F5 resets. Each does nothing when its button is greyed out.
- **Not every binding.** The editor binds dozens, some for Emacs habits. The table chooses the ones worth knowing.

## Consequences

- Changing a key's binding changes the tab and the tooltips. Adding a shortcut is a row in the table.
- The scripted session checks every row shows all its keys, so a row can't silently show none.
- Find and Replace aren't listed: the editor component has them, but only for an editor set up as searchable, which Pendown's isn't.
- Like every app shortcut, F5 and Ctrl+Enter work when the focus is somewhere in the window's panes, not when nothing has focus.
