# 9. The logo crate has no GUI code and emits a stream of turtle events

Status: Accepted
Date: 2026-10-17 (recorded later, from commit 8104814 and `plan-1.md`)

## Context

The project has a logo crate, a CLI and a GUI playground. The interpreter must be usable by both front ends, testable without a window, and able to drive an animated turtle.

## Decision

- **No GUI code in the logo crate.** The interpreter outputs a stream of turtle events in its own coordinate system: `Move`, `Turn`, `Arc`, `Ellipse`, `Rect`, `Label`, `Fill`, `Filled`, `Clean`, `SetBackground`, `ShowTurtle`/`HideTurtle`, `Print`, `Wait`, `SetSpeed`, `Instruction` and `Sync`. The full list and fields are in `plan-1.md`.
- **The stream covers everything the user should see happen,** not only lines that leave ink: pen-up moves and turns animate too. `Turn` carries the signed angle turned, so `RT 270` visibly turns 270° rather than taking the short way round.
- **Events carry resolved values,** such as RGBA colours, so front ends never need to know about palettes or colour names.
- **The interpreter does all the geometry,** including splitting a move that wraps at an edge and stopping at the edge in `FENCE` mode. It uses a fixed turtle area (default size in ADR 0003). The GUI only scales and flips the y-axis, and never sends the window size back.
- **Turtle queries** (`POS`, `HEADING`, `PENDOWNP`…) are answered from the interpreter's own turtle state, never by asking the GUI.
- **Two consumers:** the GUI animates the stream on the canvas (ADR 0011), and the CLI renders it to a file with no timing.

## Consequences

- Examples can be tested by rendering their event streams with the CLI (ADR 0018).
- Resizing the window never changes how a program behaves.
