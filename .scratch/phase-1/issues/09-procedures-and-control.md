# Procedures, variables and control

Status: done
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

### 2026-10-21: done

- **New commands:**
  - `LOCAL`: a word, a list of words, or `(LOCAL "a "b)`;
  - `LOCALMAKE`;
  - `#` as UCBLogo's short form of `REPCOUNT`.
- **How locals work:** `Interpreter::make_local` saves the variable's value in the running procedure's frame, with no new value, so it's restored when the procedure ends (shallow binding, ADR 0007). A tail-called procedure's locals go in the shared frame.
- **Decisions, with `differs` notes in the kdl:**
  - `LOCAL` and `LOCALMAKE` outside a procedure are errors.
  - `REPCOUNT` outside `REPEAT` and `FOREVER` is an error, where UCBLogo outputs -1.
  - `REPEAT` with a negative count is an error; spike 03 silently did nothing.
- **Unchanged from spike 03:** `TO`/`END` (both forms, calls before definitions), `MAKE`, `THING`, `REPEAT`, `FOREVER`, `IF`, `IFELSE`, `STOP` and `OUTPUT`, with the tail calls, the depth limit of 10,000 and the stack guard. Display spelling was fixed in issue 01.
- **Tests:** `tests/variables.rs` (11 tests):
  - `LOCAL` hiding a global, and having no value until made;
  - words and lists, and `LOCALMAKE`;
  - dynamic scope through three levels;
  - locals in tail calls;
  - `MAKE` setting the nearest variable;
  - `THING` with a computed name;
  - nested `REPCOUNT` and `#`;
  - the errors, and `IF`/`IFELSE`/`OUTPUT`.
  
  Spike 03's 41 tests still cover the rest.
