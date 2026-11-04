# How many inputs were needed and given

Status: resolved
Type: task

## What

"Not enough inputs to X" becomes "X needs 2 inputs, but got 1", at its three sources: the two in `parse.rs` (about lines 673 and 713) and the one in `prims/templates.rs` (line 118).

- The parser already knows the count: `arity.default`, `arity.min`, and how many inputs it had read when it stopped.
- The wording is "1 input", "2 inputs", and "got none" for zero.
- A form that takes more than the usual number, in parentheses, says "at least": "SUM needs at least 2 inputs, but got 1".
- "Too many inputs to X" gets the same treatment: "X takes 2 inputs, but got 3".
- Template errors keep working for `THE TEMPLATE`. Check that the wording reads well there.

The messages stay in `Error::message`. `.about()` stays as it is.

## Done when

- `REPEAT 4`, `REPEAT [FD 50]`, `FD`, `PRINT` and a procedure's `SQ` read as in the spec, while typing and on Run.
- Tests with the old wording are updated: `spike03.rs`, `templates.rs`, `syntax.rs`, `cli.rs`, `autotest.rs`.
- `cargo test --workspace` passes.

## Comments

2026-11-04: Done. The wording comes from `not_enough_inputs` and `too_many_inputs` in `error.rs`, used by the parser and by templates.

- A plain call says the usual number: `SUM 1` is "SUM needs 2 inputs, but got 1". "At least" and "at most" only come with parentheses, and only for commands that take a range: `(IF "true)` is "IF needs at least 2 inputs, but got 1". This issue's example of `SUM` saying "at least" can't happen, as `(SUM 1)` isn't an error.
- `CS` with an input reads "CS takes no inputs, but got 1".
- A template reads "The template needs 2 inputs, but got 1", no longer "THE TEMPLATE".
- ADR 0001 still quotes the old wording, as a record of the decision. Left for issue 09 to note.

`cargo test --workspace` passes. The scripted session passes every check except "frame intervals" for the 131,070-move tree, on the private Xvfb display (p50 = 32 ms). The same section fails the same way without this change, so that's the display.
