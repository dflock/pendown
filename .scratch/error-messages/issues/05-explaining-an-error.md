# Explaining an error, in the console and the hover

Status: ready-for-agent
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
