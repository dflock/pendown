# Turtle modes: `WRAP`, `WINDOW`, `FENCE`

Status: ready-for-agent
Type: task
Blocked by: 05

## What

`WRAP`, `WINDOW`, `FENCE` and `TURTLEMODE`, with the interpreter doing all the geometry (ADR 0009). The GUI only scales.

- **`WRAP`** is the default. A move that crosses an edge is split into several `Move` events, and the turtle reappears on the opposite edge. Moves that wrap more than once, and moves along an edge or through a corner, need care.
- **`FENCE`** stops the turtle at the edge with an error, as the manual says. Decide whether the move is drawn up to the edge before the error, and record it in the kdl's `differs` if it differs from UCBLogo.
- **`WINDOW`** lets the turtle move off the area, which then shows nothing of it.
- Switching to `WRAP` or `FENCE` with the turtle outside the area: follow UCBLogo (check `refs/ucblogo-code`).
- `ARC` (issue 08) must wrap and fence like any other move. Design the splitting so arcs can use it too.
- The GUI shows the space outside the turtle area in a slightly different shade (ADR 0003), so the edges are visible.

## Done when

- `pendown-logo` has tests for:
  - a move that wraps once and more than once;
  - moves through corners;
  - `FENCE` errors;
  - `WINDOW` moves off the area;
  - `TURTLEMODE`'s output.
- A `pendown-cli` picture test shows a wrapping spiral.

## Comments
