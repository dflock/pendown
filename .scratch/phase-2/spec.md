# Phase 2: the complete core language and filled shapes

The rest of UCBLogo's core language, the palette, and filled shapes: 76 commands in all, plus the special variables. See "Phase 2" in `implementation-plan.md` for the scope, and `docs/adr/` for the decisions. Filled shapes are our extension (ADR 0017).

Pixel drawing (`FILL`, `FILLED`, `LABEL`, the pen modes) is phase 7. Performance is phase 3, and the watch panels are phase 4.

## What "done" looks like

- Every phase 2 command works in `pendown-logo`, with tests of its behaviour and its errors.
- The special variables `CASEIGNOREDP`, `ERRACT`, `FULLPRINTP`, `PRINTDEPTHLIMIT`, `PRINTWIDTHLIMIT` and `LOGOVERSION` work as variables.
- Arrays can be typed in as `{a b c}` and `{a b c}@0`, and the editor colours and checks them.
- Shapes are filled with `SETFILLCOLOR`, in the GUI, in the CLI and in PNG export.
- The kdl test checks that the interpreter implements exactly the phase 1 and phase 2 commands.
- `examples/phase-2/` has programs that use every phase 2 command, and they're picture tests like phase 1's.

## Decisions

These are recorded in ADR 0021, and as `differs` notes in `pendown-logo.kdl` where we differ from UCBLogo.

- **Arrays are a fourth kind of value,** mutable and shared, so `SETITEM` changes the array wherever it's used. Two arrays are `EQUALP` only if they're the same array, as in UCBLogo.
- **Templates** come in UCBLogo's four forms:
  - a procedure's name;
  - an explicit-slot list using `?`, `?1`, `?2`, `?REST` and `#`;
  - a lambda, `[[x y] instructions]`;
  - procedure text, `[[x y] [line] [line]]`.
  
  A template is prepared once per `MAP`, `FOREACH` or similar call, not once per item.
- **`CATCH`, `THROW` and errors:**
  - `THROW` is a new way for the evaluator to leave an instruction, alongside `STOP`, `OUTPUT` and errors.
  - `CATCH "ERROR` catches errors, but never Stop (ADR 0011).
  - `ERROR` outputs UCBLogo's four-member list. Its error code is always 0, because Pendown's errors have messages, not numbers, and the line is a line number.
  - `THROW "TOPLEVEL` ends the program without an error.
- **`ERRACT`:** when an error isn't caught and `ERRACT` is a list, the list runs where the error happened, so it can see the local variables. Then the error stops the program as usual. There's no `PAUSE`, so this is for printing what went wrong.
- **Loops, `RUN`, `CASE` and the other control commands** are ordinary built-in commands that run lists through the evaluator (ADR 0007). They don't take part in tail calls; only `IF`, `IFELSE` and `OUTPUT` do, as in phase 1.

## How the work is split

| Issue | What | Blocked by |
|---|---|---|
| 01 | Arrays: the value, the reader, printing, and the array commands | |
| 02 | The rest of words and lists | 01 |
| 03 | Special variables, `NAME` and `GLOBAL` | |
| 04 | `RUN`, `RUNRESULT`, `IGNORE`, the loops, `CASE` and `COND` | |
| 05 | `CATCH`, `THROW`, `ERROR` and `ERRACT` | 04 |
| 06 | Templates and the higher-order commands | 04 |
| 07 | Arithmetic | |
| 08 | The palette and filled shapes, through to the GUI and CLI | |
| 09 | The editor: colours and checks for the new syntax | 01, 04, 06 |
| 10 | Phase 2 check: examples using every command, and kdl agreement | all others |

## Testing

As in phase 1:
- **`pendown-logo`:** every command has tests of its output and its errors.
- **Pictures:** filled shapes and the examples are tested with saved PNGs.
- **The GUI:** a scripted session (`PENDOWN_AUTOTEST=1`).

## Out of scope

- Pixel drawing (phase 7), performance (phase 3) and the watch panels (phase 4).
- Backslashes and vertical bars in words (phase 7's tokenizer details). So `FULLPRINTP` only affects how the empty word prints.
- `FORM`'s printf formats.
