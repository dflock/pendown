# CATCH, THROW, ERROR and ERRACT

Status: ready-for-agent
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
