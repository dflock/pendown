# The editor: colours and checks for the new syntax

Status: done
Type: task
Blocked by: 01, 04, 06

## What

- `{` and `}` colour as brackets, `?`, `?1` and `?REST` as commands.
- The instruction lists of `RUN`, `FOR`, `WHILE`, `UNTIL`, `DO.WHILE`, `DO.UNTIL`, `CATCH` and `IGNORE` are checked while typing, as `REPEAT`'s are.

## Done when

Each command has tests of its behaviour and its errors in `pendown-logo`, or the GUI and CLI tests cover it.

## Comments

### 2026-10-21: done

`syntax.rs` colours `{`, `}@0` and the slots, and checks the lists and templates of `RUN`, `FOR`, the `WHILE` loops, `CATCH` and the template commands.
