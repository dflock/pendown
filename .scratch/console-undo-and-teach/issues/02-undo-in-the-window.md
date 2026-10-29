# Undo in the window

Status: done
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

## Comments

### 2026-10-29: done

`Drawing::truncate` in `pendown-render` cuts the marks back and draws the rest again. The window keeps a `Before` for each console entry not undone yet (`console_before`, beside `console_entry`'s record): the number of marks, the background, the animator's Kame, and the marks its first `CLEAN` threw away. `undo_console` brings back the user's drawing if an example is showing, puts the drawing back, starts `Job::Undo` for Kame's pen and colours, puts the new animator's Kame where the entry found it, and prints `Undid: <entry>`. Kame jumps back; it doesn't walk.

The window records how each console entry ended (`Ran`, `Error` with the first error's message, `Stopped`), for Teach Kame. Run clears the entries; the console's Clear doesn't.

The button is `Undo <label>`, where `console::label` gives the first line cut to 24 characters with `…`. It's always shown, disabled while anything runs or with nothing to undo. The tooltip is `Undo (Ctrl+Z)`, or `Undo (Cmd+Z)` on macOS. Ctrl+Z is gpui-kit's `Undo` action, caught on the way down to the console's input line only when the line is empty and there's something to undo; otherwise it undoes typing.

The scripted session has a "Console undo" section: an oops taken back, Ctrl+Z past a `CLEAN` and a colour change, an entry that stopped with an error, Ctrl+Z with something typed, and an entry that was stopped. Screenshots `undo-before` and `undo-after`. It needed two new steps: `ConsoleKeys`, which presses keys in the input line after the current update, and `CheckConsoleTyped`. The whole session passes.
