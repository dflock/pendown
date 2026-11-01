# Shortcuts: a tab of every key, from the keymap

## Why

Pendown has keyboard shortcuts, but nothing tells you about them. Run has had Ctrl+Enter (Cmd+Enter on a Mac) since phase 1, and no button or page says so. The drive keys are shown only while driving, and the editor's keys (Undo, Find, Indent) not at all.

## What done looks like

- **A Shortcuts tab** in the side panel, after Gallery: Help, Tutorial, Gallery, Shortcuts. It lists the keys in groups, each row a description and its keys, drawn as key caps:
  - **Your program:** Run (Ctrl+Enter, F5), and help for the word under the cursor (F1).
  - **Files:** New, Open, Save, Save As, Export picture.
  - **The editor:** Undo, Redo, Cut, Copy, Paste, Select all, Find, Replace, Indent and Outdent (Tab, Shift+Tab), and moving by word and to the start and end.
  - **The console:** Enter runs what you typed; Up and Down go back through what you've typed before.
  - **Driving Kame:** the arrows, Space for the pen, Backspace or Ctrl+Z to undo a move, Escape to stop driving.
  - **For the curious:** F12, the metrics overlay.
- **Keys are shown as this computer writes them:** Ctrl+Enter on Linux and Windows, ⌘↩ on a Mac, from gpui-component's `Kbd`.
- **F5 also runs the program,** as in many editors. Ctrl+Enter stays.
- **Toolbar tooltips show their key:** "Run · Ctrl+Enter", from the same keymap. New, Open, Save and Export too, on the file menu buttons.

## How it stays in sync

**The keys come from the keymap, not from text.** gpui keeps every binding, and `Window::highest_precedence_binding_for_action` (and `_in_context`) say which key does an action. The tab asks for each action's key as it draws, so changing a binding changes the tab.

- **`shortcuts.rs`: one table** of groups and rows. A row is an action and its description, like `(RunProgram, "Run your program")`. `bind_keys` binds the app's own rows from the same table, so a binding and its row are one entry: you can't add a key without saying what it does.
- **The editor's rows point at the editor's actions** (`Undo`, `Search`, `Indent`...), from gpui-base, looked up in its `"Input"` context. If gpui-base changes a key, the tab follows; if it drops or renames an action, the build breaks.
- **The drive keys become a table in `driving.rs`** that both `drive_key` and the tab read. They're handled by hand, not through the keymap, so this table is their one list. The drive strip's labels come from it too.
- **The console's keys are written in the table as text.** Enter and Up and Down are how a text line behaves, not bindings that could change.
- **Tests:** every action row has a binding (no row shows a blank), every binding in `bind_keys` has a row, and the drive table has no key twice. The scripted session opens the tab and checks a few rows and that Run's tooltip names its key.

## Decisions

- **A tab after Gallery,** not a page in Help. Help's pages are about the language; shortcuts are about the app.
- **Ctrl+Enter and F5 both run.** One action, two bindings; the row shows both.
- **Not every binding.** The editor binds dozens of keys, some only for Emacs habits (Ctrl+A to the line's start on a Mac). The table chooses the ones worth knowing; the keymap only supplies their keys.
- **Several keys for one action are all shown,** in the keymap's order, as `Kbd`s with "or" between them.

## Out of scope

- Changing keys (a keymap editor or settings file).
- ~~New shortcuts for Pause, Single Step or Reset.~~ Added after all, in issue 05.
- A menu bar.

## Issues

| # | Issue | Blocked by |
|---|---|---|
| 01 | The table, and binding keys from it | |
| 02 | The Shortcuts tab | 01 |
| 03 | Drive keys from one table | 01 |
| 04 | Tooltips with keys, README and session | 02, 03 |
| 05 | Keys and tooltips for Pause/Resume, Single Step and Reset | 04 |
