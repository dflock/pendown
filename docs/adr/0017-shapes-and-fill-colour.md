# 17. Shapes centred on the turtle, with a fill colour

Status: Accepted
Date: 2026-10-18 (recorded later, from commit 5bc4b58)

## Context

UCBLogo has no built-in shapes. Circles have to be drawn with `REPEAT` or `ARC`, and the only fills are `FILL` (a flood fill) and `FILLED` (a polygon traced by the turtle). LibreLogo and FMSLogo have shape commands and a separate fill colour, which make drawing much friendlier.

## Decision

New commands, all drawn **centred on the turtle**, turned with its heading. The turtle doesn't move.

- **`CIRCLE radius`** and **`ELLIPSE across along`** (phase 1). `across` is the radius from side to side relative to the turtle, and `along` is the radius in the direction it's heading.
- **`SQUARE side`** and **`RECTANGLE across along`** (phase 1), with full side lengths. An optional last input in parentheses rounds the corners: `(SQUARE side corner)`, `(RECTANGLE across along corner)`.
- Circles and ellipses take radii, and squares and rectangles take side lengths, because that's how people usually measure each.
- **Outline:** drawn with the current pen when the pen is down.
- **Fill (phase 2):** `SETFILLCOLOR colour` (short form `SETFC`) sets a fill colour, using any colour form (ADR 0015). `SETFILLCOLOR "none` turns filling off, and `"none` is the default. When a fill colour is set, shapes are filled even with the pen up, so pen up gives a filled shape with no outline. `FILLCOLOR` (`FC`) reports it, or `NONE`.
- **Events:** `Ellipse { center, radii, heading, pen, fill }` for `CIRCLE` and `ELLIPSE`, and `Rect { center, size, corner, heading, pen, fill }` for `SQUARE` and `RECTANGLE`.
- **At the edges,** shapes are drawn as if in `WINDOW` mode: they're never wrapped or fenced, and anything outside the canvas is cut off (phase 1 issue 08). They don't move the turtle, and drawing a shape in pieces round the edges would look broken.
- **Animated,** the outline sweeps round clockwise from the point straight ahead of the turtle, at the turtle's speed along the outline. A shape with the pen up and no fill can't be seen, so it takes no time.
- **Rounded corners** can't be rounder than half the shorter side; a larger corner radius is reduced to that. Negative sizes are errors.

## Consequences

- UCBLogo's `FILLED` and `FILL` (phase 5) still cover filling shapes drawn by the turtle.
- `SETFC` and `FC` are our own short names; no other Logo uses them.
- All six commands are in `pendown-logo.kdl` with `extension #true` and a `source`.
