# Special variables, NAME and GLOBAL

Status: done
Type: task

## What

- **`CASEIGNOREDP`:** true by default; when false, `EQUALP`, `MEMBERP`, `BEFOREP` and the rest compare case.
- **`FULLPRINTP`:** when true, the empty word prints as `||`.
- **`PRINTDEPTHLIMIT` and `PRINTWIDTHLIMIT`:** limit how much of a list `PRINT`, `SHOW` and `TYPE` print, with `...` for the rest.
- **`LOGOVERSION`:** Pendown's version, as a number.
- **Run** sets them afresh, along with clearing the other variables (ADR 0006).
- **`NAME value varname`** is `MAKE` with its inputs swapped. **`GLOBAL`** accepts names, and makes sure they exist.

## Done when

Each command has tests of its behaviour and its errors in `pendown-logo`, or the GUI and CLI tests cover it.

## Comments

### 2026-10-21: done

The special variables are interned once (`Specials` in `interp.rs`) and set afresh on every Run. `EQUALP` and the other comparisons read `CASEIGNOREDP`, and `PRINT`, `TYPE` and `SHOW` read the print limits through `Value::print_styled`. Tests: `tests/specials.rs`.
