# What kind of input was wanted, and clearer wording

Status: ready-for-agent
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
