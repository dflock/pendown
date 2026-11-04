# Smarter hints

Status: ready-for-agent
Type: task
Blocked by: 05

## What

Hints that need their own logic.

**Did you mean.** In `Error::unknown` (`error.rs`), and with the learner's own procedures where they're known:

- `FORWRD` gets "Did you mean FORWARD?", from an edit distance of 1 against implemented commands and defined procedures, or 2 for longer names. Only suggest when there's exactly one closest match. Obsolete commands are left out. Use a small function rather than a new dependency.
- `FD100` gets "Did you mean FD 100?", when the word is a known command followed by digits.
- A suggestion's name goes through `.about()`, so the card shows that command's syntax.

Add a test of real-looking typos, and of words that must not get a suggestion, such as short ones. `syntax::check` has `no_procedures` for its `known` lookup, so check how procedures defined in the text can be offered there.

**Leftover input.** `FORWARD 10 20` gives "You don't say what to do with 20" from `interp.rs` (about line 340). It should say "Nothing uses 20. FORWARD takes 1 input" when the value comes straight after a call on the same line, and keep the current wording otherwise. Work out how to tell, perhaps from the instruction before it in the block. `prims/templates.rs` line 261 stays as it is.

**Brackets for `FOR` and `WHILE`.** `FOR [i 1 3] FD 5` and `WHILE "true FD 5` say "FD didn't output to FOR", as these primitives evaluate their inputs before running, so they miss issue 03's message in `eval_list`. They should say "FOR needs a list of instructions here. Put them in [ ]", and likewise for other primitives that take instructions this way. Work out which those are, and how to know which of their inputs want a list. Other cases keep "FD didn't output to PRINT".

## Done when

- The spec's three programs read as written, in the console and the card.
- `FOR` and `WHILE` without brackets read as above.
- A wrong suggestion hasn't been seen in the test list.
- `cargo test --workspace` passes.

## Comments
