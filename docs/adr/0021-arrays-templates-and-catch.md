# 21. Arrays, templates, and CATCH and THROW

Status: Accepted
Date: 2026-10-21

## Context

Phase 2 completes UCBLogo's core language. Three parts of it needed decisions about the interpreter rather than about single commands:
- arrays, which are mutable, unlike every other Logo value;
- templates, the instruction lists that `MAP`, `FOREACH` and the other higher-order commands run;
- `CATCH` and `THROW`, which add a new way to leave an instruction.

The rest of the control commands needed a decision too: whether they're built into the evaluator, as `IF` and `REPEAT` are, or are ordinary commands.

## Decision

- **Arrays are a fourth kind of value,** `Value::Array`, with their members in a `RefCell` and an origin, 1 unless given.
  - They're shared: `SETITEM` changes the array wherever it's used, as in UCBLogo. That includes an array typed into a procedure, which is created once, when the program is read.
  - Two arrays are `EQUALP` only if they're the same array.
  - `SETITEM` and `MDSETITEM` refuse to put an array inside itself, so printing and comparing can't loop forever.
  - The reader reads `{a b c}` and `{a b c}@0`. Arrays print in braces, even from `PRINT`.
  - `ARRAY`, `MDARRAY`, `ISEQ` and `RSEQ` refuse to make more than 10 million members, so a typo such as `ISEQ 1 1e12` is an error instead of running out of memory.
- **Templates come in UCBLogo's four forms:**
  - a procedure's name;
  - an explicit-slot list, using `?`, `?1` (read as `(? 1)`), `?REST` and `#`;
  - a lambda, `[[x y] instructions]`, whose inputs are local variables in a frame of their own;
  - procedure text, `[[x y] [line] [line]]`, made into a procedure.
  
  A template is prepared, and its lists parsed, once per call of `MAP`, `FOREACH` or the like, not once per member. `?REST` is only worked out when it's used, so `MAP` stays linear in the length of its data.
- **`#` and `REPCOUNT`** give the position in the data inside the template commands, as well as the count inside `REPEAT` and `FOREVER`.
- **`THROW` is a new kind of exit from an instruction,** alongside `STOP`, `OUTPUT` and errors.
  - `CATCH` compares tags without regard to case.
  - `CATCH "ERROR` catches errors and `THROW "ERROR`. It never catches Stop (ADR 0011).
  - `(THROW "ERROR message)` is an error at the call of the procedure containing it, as if that procedure were a primitive.
  - `THROW "TOPLEVEL` ends the program without an error. Any other uncaught `THROW` is an error.
- **`ERROR`** outputs UCBLogo's four-member list, `[code message procedure line]`.
  - The code is always 0, because Pendown's errors have messages, not numbers.
  - The line is a line number, since the editor shows line numbers.
- **`ERRACT`:** there's no `PAUSE`, so when an error isn't caught and `ERRACT` is a list, the list runs where the error happened, where it can see the local variables. Then the error stops the program as usual. It doesn't run inside `CATCH "ERROR`.
- **The loops, `RUN`, `CASE`, `COND`, `CATCH` and the template commands are ordinary built-in commands.** They run lists through the evaluator (ADR 0007).
  - They don't take part in tail calls; only `IF`, `IFELSE` and `OUTPUT` do, as in phase 1.
  - `STOP` and `OUTPUT` inside them leave the procedure they're in, as in UCBLogo.
- **Every loop checks Stop and the instruction limit on every pass,** so a loop with an empty body, such as `FOREVER []`, can still be stopped, and the CLI's `--limit` still ends it.
- **The special variables** (`CASEIGNOREDP`, `ERRACT`, `FULLPRINTP`, `PRINTDEPTHLIMIT`, `PRINTWIDTHLIMIT` and `LOGOVERSION`) are ordinary variables, which the interpreter reads when it needs them. Run sets them afresh with the others (ADR 0006).

## Consequences

- A recursion through `RUN`, `FOREACH` or `MAP` isn't a tail call, so it's limited to the usual depth of 10,000 calls.
- Arrays are the only values that can change, so they're the only ones `EQUALP` compares by identity. Code that copies values, such as the watch panels in phase 4, must remember that.
- The phase 4 watch panels can read the counters of `REPEAT`, `FOREVER` and the template commands from the same stack.
- The `differs` notes in `pendown-logo.kdl` record where these choices differ from the manual.
