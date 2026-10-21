# `ARC` and the shapes

Status: done
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

### 2026-10-21: done

- **`ARC`** (`prims/shapes.rs`, ADR 0016):
  - `geometry::arc_center` and `arc_point` give the arc's centre, on the turtle's right for a right turn, and where the turtle is part-way along.
  - An arc inside the area, or any arc in `WINDOW` mode, is one `Arc` event.
  - Otherwise it goes as one-degree chords, each a `Move` and a `Turn`, which wrap or stop at the fence like `FD`.
- **The shapes:** `CIRCLE`, `ELLIPSE`, `SQUARE` and `RECTANGLE`, with optional rounded corners, send `Ellipse` and `Rect` events. `fill` is always `None` until phase 2.
  - **Decided:** shapes at the edges draw as if in `WINDOW` mode, never wrapped. ADR 0017 records it, and also the sweep and the corner limit (at most half the shorter side).
- **`pendown-render`** has a `Mark` type (line, arc, ellipse, rectangle), and the `Drawing` now takes marks. Each mark's outline is flattened to within a fifth of a pixel, which gives:
  - the strokes, with round joins;
  - the length;
  - the partial outline while sweeping.
- **The animator** plays arcs and shapes at the turtle's speed along their outline. An arc carries the turtle and turns it as it goes; a pen-up shape takes no time. `partial()` now returns a polyline, which the canvas strokes.
- **`SQUARE` is now a built-in,** so tests and examples that defined `TO SQUARE` now define `TO BOX`, including the welcome program.
- **Tests:**
  - `tests/shapes.rs` (8 tests), and the geometry `arcs` unit test;
  - 6 `Mark` unit tests, and a render test of shape outlines;
  - 2 animator tests;
  - an `arcs-and-shapes` picture.
