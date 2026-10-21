# CATCH, THROW, ERROR and ERRACT

Status: done
Type: task
Blocked by: 04

## What

- **`CATCH tag list`** and **`THROW tag`** / **`(THROW tag value)`**, with case-insensitive tags. An uncaught `THROW` is an error, "Can't find a CATCH for TAG".
- **`CATCH "ERROR`** catches errors, never Stop. **`ERROR`** outputs `[0 message procedure line]`, or `[]`, then forgets the error.
- **`THROW "ERROR`**, with an optional message, which points at the call to the procedure containing it.
- **`THROW "TOPLEVEL`** ends the program without an error.
- **`ERRACT`:** run where an uncaught error happened, then the error stops the program.

## Done when

Each command has tests of its behaviour and its errors in `pendown-logo`, or the GUI and CLI tests cover it.

## Comments

### 2026-10-21: done

`Flow::Throw` is the new exit. `ERROR`'s code is always 0, and its line is a line number (ADR 0021). `ERRACT` runs at the innermost procedure, or at the top level, before the error carries on. Tests: `tests/catch.rs`.
