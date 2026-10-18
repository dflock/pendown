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
- The `Arc` event carries the pen, and the animator moves the turtle sprite along the arc.

## Consequences

- UCBLogo programs that use `ARC` draw differently. `CIRCLE` (ADR 0017) covers drawing around the turtle.
- `pendown-logo.kdl` keeps the manual's description of `ARC` and adds a `differs` note.
