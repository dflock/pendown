# Explaining an error, in the console and the hover

Status: resolved
Type: task
Blocked by: 02, 04

## What

One function that turns an error into everything shown about it, in `pendown-logo` with no GUI code, so it can be tested without a window:

```rust
pub struct Explanation {
    pub message: String,
    /// `REPEAT number [instructions]`
    pub usage: Option<String>,
    /// `REPEAT 4 [FD 150 RT 90]`
    pub example: Option<String>,
    /// For a learner's own procedure: the line of its TO.
    pub defined_on: Option<u32>,
    /// Index into `Reference::commands`, for "Help on".
    pub help: Option<usize>,
}

pub fn explain(error: &Error, source: &str) -> Explanation
```

- **Which command:** `Error::command`, which `.about()` sets.
- **Syntax from `eg`:** pick the line starting with the name as typed, and fall back to the first line. Turn the manual's input names into plain ones with a small table, starting with the words most used in `eg`: `num`, `instructionlist`, `dist`, `thing`, `list`, `tf`, `tfexpression`. Words not in the table stay as they are.
- **A learner's procedure:** when the reference has no such command, find its `TO` line in `source`.
- **Errors with no command,** such as "This [ has no matching ]": message only.

**Using it:**

- `show_errors` (`app.rs`, about line 1262) prints the syntax and example under the error, before the call chain. `error_help` comes from the explanation.
- Both places that build a `Diagnostic` (about lines 1251 and 1287) use a Markdown message with the syntax and example in code blocks. The editor's popup already renders Markdown.
- `pendown-cli` prints the message only, so its output stays one line per error.

`CheckDiagnostics` in the scripted session compares whole messages, so it should compare the first line.

## Done when

- Unit tests cover: `FD` and `FORWARD` choosing their own `eg` line, a command with no `try`, a procedure with and without inputs, and an error with no command.
- Hovering an underline in the real app shows the syntax and example, checked on the private display.
- `cargo test --workspace` and the scripted session pass.

## Comments

2026-11-04: Done. `explain` is in `crates/pendown-logo/src/explain.rs`, with tests in `tests/explain.rs`. `errors.rs` in the app turns an `Explanation` into console lines and the hover's Markdown.

- **Input names:** beyond the issue's list, the table covers `varname` (`MAKE "name value`), `pos` (`SETPOS [x y]`), `xcor`, `ycor`, colours, `wordorlist` and a few more. `thing` reads as "value" and `tf` as "condition". Others stay as the manual has them, such as `RT degrees`.
- **Not in the issue:**
  - An operator gets its own line, so `+` shows `number + number`.
  - An obsolete command offers Help, as it did, but no syntax, since it can't be used. A learner's procedure with that name is treated as theirs.
  - The hover escapes Markdown in the message, or "This ] has no matching [" could be read as markup.
  - `usage_lines` moved from `help_panel.rs` to `Command`, as both need it.
  - Replaying a paused program prints the syntax and example too, like `show_errors`, but still doesn't set `error_help`, as before.
- **Which text procedures are looked for in:** `run_text` for a run's errors, so a help example finds its own, and the editor's text while typing. A procedure defined only in a console entry has no `TO` in either, so its error shows the message alone.
- **Finding `TO`** looks at the first two words of each line, after any `;`. A line continued with `~` isn't followed.

Checked on the private display: hovering `REPEAT 4`, a procedure's `SQ` and a stray `]` shows the right content, and the console shows it after Run, before the call chain. `cargo test --workspace` passes, 434 tests. In the scripted session the only failure is "frame intervals" on the 131,070-move tree, as in issues 02 and 03.
