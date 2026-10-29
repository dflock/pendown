# Drive Kame

Status: needs-triage
Type: task
Blocked by: 02

## What

- A toolbar toggle, **Drive**, which focuses the canvas; Esc, Run, Pause and Step stop driving.
- The strip across the top of the canvas: what the keys do, the move step (1, 5, 10, 25, 50) and turn step (1, 5, 15, 30, 45, 90), and `Pen up` when it is.
- Keys: ↑ `FD`, ↓ `BK`, ← `LT`, → `RT`, Space `PU`/`PD`, Backspace and Ctrl+Z Undo.
- Each press is a console entry; presses of the same key in a row join into one, carried on without saving Kame for Undo, with the entry's text and console line updated.
- Its own speed, 100 steps and 150° a second; presses made while one plays wait their turn.
- Thinking always shown while driving, with the line out in front of Kame and `RT 45 · heading 135`.
- An ADR: driving is console entries; thinking is never part of the drawing.

## Done when

Driving works as in the spec and mockups, Undo and Teach Kame work on driven entries, and unit tests cover joining presses.
