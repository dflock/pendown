# Procedures, variables and control

Status: ready-for-agent
Type: task
Blocked by: 01

## What

Finish the language commands. Most of them already exist from spike 03.

- **`TO`/`END`:**
  - inputs, multi-line and one-line forms, and calls before definitions (ADRs 0001 and 0006);
  - the error messages for a bad name, a missing `END`, and a definition given twice.
- **Variables:** `MAKE`, `LOCAL`, `LOCALMAKE` and `THING`, with dynamic scope and shallow binding (ADR 0007).
  - `LOCAL` takes a word or a list of words, and `(LOCAL "a "b)`.
  - A `LOCAL` in a tail-called procedure adds to the current frame, like its inputs.
- **Control:** `REPEAT`, `REPCOUNT`, `FOREVER`, `IF`, `IFELSE`, `STOP` and `OUTPUT`, with the tail positions and the depth limit of 10,000 from ADR 0007.
- **Names:** case-insensitive, with the spelling from the first `TO` or `MAKE` kept for display (ADR 0004).

## Done when

- Spike 03's tests pass, plus new ones for:
  - `LOCAL`, `LOCALMAKE` and `THING`;
  - dynamic scope through several levels;
  - `REPCOUNT` in nested `REPEAT`s;
  - display spelling.
- The depth limit and the stack guard are tested in `crates/pendown-logo`, as in spike 03.

## Comments
