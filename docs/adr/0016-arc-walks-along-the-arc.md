# 16. `ARC` moves the turtle along the arc

Status: Accepted
Date: 2026-10-18 (recorded later, from commit 5bc4b58)

## Context

In UCBLogo, `ARC angle radius` draws an arc *around* the turtle, with the turtle at the centre, and the turtle doesn't move. That surprises nearly everyone, and doesn't match what you'd draw by hand.

## Decision

- **`ARC angle radius`** moves the turtle along an arc of a circle with that radius, turning through `angle` degrees as it goes. A positive angle curves right (clockwise, like `RT`), and a negative angle curves left.
- It finishes at the end of the arc with its heading changed by `angle`. So `ARC 360 50` draws a full circle and returns the turtle to where it started, and so does `REPEAT 4 [ARC 90 50]`.
- It draws only when the pen is down, like `FD`, and wraps or stops at the edges like any other move.
- For a positive angle it's the same as `REPEAT angle [FD (2 * PI * radius / 360) RT 1]`, but drawn as a smooth curve.
- The `Arc` event carries the pen, and the animator moves the turtle sprite along the arc, at the same speed in steps as a move.
- **At the edges** (phase 1 issue 06): an arc that stays inside the turtle area, or any arc in `WINDOW` mode, is one `Arc` event. An arc that would cross an edge in `WRAP` or `FENCE` mode goes as one-degree chords instead, each a `Move` and a `Turn`. So it wraps or stops at the fence exactly like `FD`, as UCBLogo's `ARC` does.
- `ARC angle 0` just turns, `ARC 0 radius` does nothing, and a negative radius is an error.

## Consequences

- UCBLogo programs that use `ARC` draw differently. `CIRCLE` (ADR 0017) covers drawing around the turtle.
- `pendown-logo.kdl` keeps the manual's description of `ARC` and adds a `differs` note.
