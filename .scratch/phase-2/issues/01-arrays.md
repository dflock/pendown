# Arrays

Status: ready-for-agent
Type: task

## What

- **The value:** `Value::Array`, mutable and shared, with an origin (1 unless given).
- **The reader:** `{a b c}` and `{a b c}@0`, nested in lists and arrays, with errors for an unmatched `{` or `}`.
- **Printing:** arrays print in braces, with `@origin` when the origin isn't 1, from `PRINT` as well as `SHOW`.
- **Commands:** `ARRAY`, `MDARRAY`, `LISTTOARRAY`, `ARRAYTOLIST`, `SETITEM`, `MDITEM`, `MDSETITEM` and `ARRAYP`/`ARRAY?`.
- **The phase 1 commands on arrays:** `ITEM`, `COUNT`, `EMPTYP`, and `FIRST`, which outputs the origin, as the manual says. `EQUALP` compares arrays by identity.
- **`SETITEM` refuses** a value that contains the array, so arrays can't be circular.

## Done when

Each command has tests of its behaviour and its errors in `pendown-logo`, or the GUI and CLI tests cover it.

## Comments
