# Undo in the window

Status: needs-triage
Type: task
Blocked by: 01

## What

- Before each console entry, save the number of marks, the background, and the animator's position, heading and shown flag. On the entry's first `CLEAN`, save the marks it throws away.
- `Drawing::truncate(n)` in `pendown-render`, which cuts the marks back and draws the rest again, as `set_background` does.
- The **Undo** button in the console's title bar, labelled `Undo <entry>` (shortened with `…` past about 24 characters, and on one line), disabled while running or with nothing to undo, with the tooltip `Undo (Ctrl+Z)`, and Lucide's `undo-2` icon (`IconName::Undo2`: add it to `ExtraIcons` in `icons.rs` if gpui-kit's default set doesn't embed it).
- Ctrl+Z / Cmd+Z in the console's input line when it's empty.
- Undo starts `Job::Undo`, puts the drawing and Kame back, and prints `Undid: <entry>`.
- Run clears the entries.

## Done when

Undo works as in the spec and the mockups, including after an error, after Stop, and after `CLEAN`, and screenshots show it.
