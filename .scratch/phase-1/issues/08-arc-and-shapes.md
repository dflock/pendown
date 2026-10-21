# `ARC` and the shapes

Status: ready-for-agent
Type: task
Blocked by: 05, 07

## What

- **`ARC angle radius`,** our version (ADR 0016): the turtle walks along the arc and ends turned by `angle`.
  - An `Arc` event carries the pen.
  - The animator moves the sprite along the arc at the current speed, measured along the arc.
  - `pendown-render` draws it as a smooth curve.
  - It wraps and fences like any other move (issue 06's splitting).
- **`CIRCLE`, `ELLIPSE`, `SQUARE` and `RECTANGLE`** (ADR 0017):
  - centred on the turtle and turned with its heading;
  - the turtle doesn't move;
  - an outline only, drawn with the current pen when it's down (fill is phase 2);
  - `(SQUARE side corner)` and `(RECTANGLE across along corner)` round the corners.
- **Events:** `Ellipse { center, radii, heading, pen, fill }` and `Rect { center, size, corner, heading, pen, fill }`, with `fill` always none in phase 1.
- **At animated speeds,** the animator draws the outline sweeping round, as the plan says.
- **The highlight** stays on the shape's instruction while it's drawn.
- Decide how shapes behave at the edges in `WRAP` mode (clip, or draw as if `WINDOW`), and record it in ADR 0017.

## Done when

- `pendown-logo` has tests of the events and the turtle's final state. `ARC 360 50` and `REPEAT 4 [ARC 90 50]` both return the turtle to where it started.
- `pendown-render` has pixel tests, and `pendown-cli` has a picture test of all four shapes, with rounded corners and an arc.
- In the GUI, arcs and shapes animate at slow speeds.

## Comments
