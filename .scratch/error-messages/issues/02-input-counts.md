# How many inputs were needed and given

Status: ready-for-agent
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
