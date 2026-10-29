# 25. Console undo takes back the drawing and Kame, not variables or procedures

Status: Accepted
Date: 2026-10-29

## Context

Mindstorms' `ERASE 1` takes back the last thing drawn, so trial and error at the keyboard costs nothing. Pendown's console runs against whatever the last run left (ADR 0006), and entries pile up on it: moves, turns, pen and colour changes, `MAKE` and `TO`. An Undo for the console has to say which of these it takes back.

Kame's state lives in the interpreter's `Turtle`, which holds `Value`s. They hold `Rc`s, so they can't leave the interpreter's thread (ADR 0022). The drawing lives in the window.

## Decision

- **Undo takes back the newest console entry not yet undone,** as far back as the last Run. Run starts again with nothing to undo.
- **It puts back the drawing and Kame as the entry found them:** the marks, the background, and Kame's position, heading, pen (up or down, colour, size), fill colour, shown or hidden, and turtle mode.
- **It leaves variables and procedures as they are.** An undone `MAKE` or `TO` still counts. `ERASE 1` is about the picture, and taking back a variable or a procedure that later entries use would surprise more than it helps.
- **An entry that failed, or was stopped, can be undone too,** and so can one that didn't parse. Each entry has a saved state, so Undo stays in step with the entries.
- **The interpreter saves Kame:** `run_console` pushes a copy of the turtle and the background before each entry, `undo_console` pops one and puts it back, and `run_program` clears them. `Job::Undo` runs it on the interpreter's thread, and ends as other jobs do, with a `Watch`, so the colours panel follows.
- **The window saves the drawing:** the number of marks, the background and the animator's Kame before each entry, and the marks a `CLEAN` threw away.
- **The console keeps a record of the entries** beside its Up/Down history, which leaves out repeats: each entry's text, how it ended, and whether it's been undone. Undone entries stay in the history, so Up and Enter run one again.

## Consequences

- No new Logo command: Undo is a console button and Ctrl+Z, not `ERASE 1`, which ADR 0006 already leaves out.
- After undoing a `MAKE`, the variables panel still shows the variable. The console's message for Undo says what was taken back, not what wasn't.
- The saved turtles grow by one per console entry until the next Run. A turtle is small, so this doesn't matter.
- Anything that runs a console job must keep the window's entries and the interpreter's saved turtles in step: Teach Kame's definition, which isn't an entry, needs a way to run without saving one.
