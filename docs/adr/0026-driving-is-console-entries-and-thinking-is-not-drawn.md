# 26. Driving is console entries, and Kame thinking is never part of the drawing

Status: Accepted
Date: 2026-10-29

## Context

Drive Kame moves and turns Kame from the arrow keys, the floor turtle's button box. Show Kame thinking draws a turn's arc and a move's ruler over the canvas, to give the numbers their meaning. Driving has to fit with what the console already does, Undo (ADR 0025) and Teach Kame, and the arcs and rulers have to fit with the drawing, Export and the animator (ADRs 0010, 0011).

## Decision

- **Each press is a console entry:** it's printed as `> FD 10`, recorded with the console's entries, and runs as a console job, so Undo and Teach Kame work on driving as on typing. There is no separate driving history.
- **Presses of the same key in a row join into one entry,** `FD 30` rather than three `FD 10`s, so taught procedures read as a person would write them. The first press is the entry; the rest carry it on without saving Kame for Undo (`define_console`), and the entry's text, its history and its console line change to match. Undo takes back the whole entry. Any other key, Space, Undo, or typing at the console ends it.
- **Driving plays at a speed of its own,** 100 steps a second, whatever the speed slider says, so it feels direct and the slider stays the program's. Presses made while one plays wait their turn.
- **Kame thinking is the animator's to know and the canvas's to draw** (`Animator::thinking`), drawn over the drawing and under Kame each frame, like the move in progress. It's never in the pixel buffer, the marks, or Export. Only at animated speeds.
- **While driving, the window joins thinking across an entry's presses,** since each press is a job with an animator of its own: the arc or ruler shows the whole entry from where it started, with a line out in front of Kame and what the entry means under it.

## Consequences

- An entry joined from ten presses is ten lines in the drawing, which look as one. Teach Kame and Undo see one entry.
- Space tracks the pen from when driving starts, assuming it's down; a program that left it up makes the first Space say `PU` again.
- Driving needs the canvas to have focus. The toolbar's New, Open, Save and Export became icons with tooltips, to make room for Drive and Thinking at the default window size.
