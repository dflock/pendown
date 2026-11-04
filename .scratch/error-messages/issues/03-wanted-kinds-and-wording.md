# What kind of input was wanted, and clearer wording

Status: resolved
Type: task

## What

**"Doesn't like".** `Args::doesnt_like` (`prims/mod.rs`, about line 926) can't say what was wanted, but the accessors that call it can: `num` and `int` want a number, `word` a word, and so on. They pass it along, giving "FD needs a number, but got the word x" and "RT needs a number, but got the list [90]".

Other code calls `doesnt_like` directly. Where that code knows what it wanted, it says so. The rest keep today's wording, which is still true. Don't try to convert every call in this issue.

**REPEAT with no brackets.** `REPEAT 4 FD 50` gives "FD didn't output to REPEAT" from `didnt_output` (`interp.rs`, about line 346). When the input being filled wants a list of instructions, it should say "REPEAT needs a list of instructions here. Put them in [ ]". `eval` only gets the name of what the value is for, so work out how it can tell which input this is, perhaps from the `eg` line's input names. Other cases keep "FD didn't output to PRINT".

**One-line rewordings:**

- ":size has no value" becomes ":size has no value yet. Give it one with MAKE "size 100". Check that it reads well for every variable, including a name that's an input of a procedure.
- "SQ has no END" becomes "SQ has no END. Add END after its last line".
- "Unexpected ]" becomes "This ] has no matching [". Check the same error for other brackets.

## Done when

- Each of those programs reads as in the spec.
- A test covers each new wording, and the old tests are updated. Help examples with `fails=#true` still pass (`tests/examples.rs`), with their saved `.txt` files looked at before updating.
- `cargo test --workspace` passes.

## Comments

2026-11-04: Done. `Args::wanted(i, "a number")` says what an input should have been, and `Value::described` says what it was. New tests are in `crates/pendown-logo/tests/messages.rs`.

- **Says what it wanted:** `num`, `int` ("a whole number"), `point` ("a list of two numbers"), `word`, `list`, `runnable`, the conditions of `IF` and `IFELSE`, `AND`, `OR` and `NOT` ("TRUE or FALSE"), and `members`.
- **Keeps "doesn't like":** a value that's the right kind but not allowed, such as `SQRT -1`, `REPEAT -1 [FD 1]` and `FIRST []`, and the other direct `doesnt_like` calls, about 55 places. It's still true, and these could be improved one by one.
- **Brackets:** `eval_list` turned out to know the position and that it wants a list, so no `eg` lookup was needed. It covers `REPEAT`, `IF`, `IFELSE`, `FOREVER` and the others that use it. It also says "REPEAT needs a list of instructions, but got 5" when the input is a value.
- **Not covered:** primitives that evaluate their inputs first, including `FOR` and `WHILE`, still say "FD didn't output to FOR". That needs a different approach, as they don't use `eval_list`. A follow-up, if it's worth it.
- `THING "nothing` and `PUSH "nothing 1` keep "nothing has no value", as only `:name` was in scope.
- `This ) has no matching (` and `This } has no matching {` changed along with `]`.

`cargo test --workspace` passes, 415 tests. The saved `error-1.txt` and `phase-2-control.txt` changed only in that message. In the scripted session on Xvfb, the only failure is "frame intervals" on the 131,070-move tree, which issue 02's comment shows is the display.
