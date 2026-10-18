# 12. Highlight the running instruction; Step runs one line

Status: Accepted
Date: 2026-10-18 (recorded later, from commits 8104814 and 1ac2446)

## Context

A headline feature is a direct visual link between what the turtle is doing and the code doing it. The plan first highlighted the running line, but `REPEAT 4 [FD 50 RT 90]` is all one line, so a line highlight can't show which `FD` or `RT` is running. Separately, the user wanted Step to go one line of code at a time. UCBLogo's own `STEP` and `TRACE` commands are obsolete here, replaced by GUI controls.

## Decision

- **Before running each instruction, the interpreter sends `Instruction { span, starts_line }`.** `span` covers just that instruction, so the highlight moves `REPEAT` → `FD 50` → `RT 90` and round again. `starts_line` marks the first instruction run from each source line.
- **Running normally,** `Instruction` is an ordinary event that only moves the highlight.
- **Step** runs one source line while paused. Everything that line draws plays at the current speed, with each instruction highlighted, then the program pauses at the start of the next line.
- **While paused or stepping,** the GUI sets a shared `stepping` flag, and the interpreter treats each `Instruction` with `starts_line` like `Sync`, waiting for the animator before running the line. Otherwise the interpreter could already have run lines ahead of the highlight, including effects that aren't events, such as `MAKE`.
- **The Pause button only stops the animator.** It isn't Logo's `PAUSE` command, which is obsolete, along with `CONTINUE`. A later "Pause and inspect" could replace it in the GUI.
- **Every parsed instruction carries a source span** from the start, recording which source it came from (ADRs 0001 and 0006).

## Consequences

- Open question: whether a second "Step instruction" button, which stops inside brackets, would also be useful.
