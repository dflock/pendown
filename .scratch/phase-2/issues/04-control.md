# RUN, the loops, CASE and COND

Status: ready-for-agent
Type: task

## What

- **`RUN`** runs a list (or a word), and outputs if its last instruction outputs. **`RUNRESULT`** outputs `[]` or `[value]`. **`IGNORE`** does nothing.
- **`FOR`**, with a local control variable, an optional step, and its limits evaluated as by `RUN`.
- **`WHILE`, `UNTIL`, `DO.WHILE` and `DO.UNTIL`.**
- **`CASE` and `COND`,** as commands or operations.
- `STOP` and `OUTPUT` inside them leave the procedure, as in UCBLogo, and errors point at the code inside the list.

## Done when

Each command has tests of its behaviour and its errors in `pendown-logo`, or the GUI and CLI tests cover it.

## Comments
