# 10. Keep the finished drawing in a pixel buffer

Status: Accepted
Date: 2026-10-17 (recorded later, from commit 8104814 and `plan-1.md`)

## Context

gpui's canvas keeps nothing between frames, so every visible line must be painted again every frame. Drawing only with vector paths would get slow for drawings with 100k+ segments. The Berkeley Logo manual also adds commands that need pixels:

- `FILL` is a flood fill bounded by lines drawn earlier, so it has to read pixels.
- `PENREVERSE` inverts each point along the turtle's path, and gpui has no XOR or invert blend mode.
- `PENERASE` can't be faked by drawing in the background colour, because the background can change later.

## Decision

- **The finished drawing lives in a pixel buffer** from the start of phase 1, even though `FILL` and the pen modes arrive in phase 2. `tiny-skia` is the candidate renderer.
- **Vector paths are only used for what's changing:** the move in progress and the turtle sprite. Frame cost then depends on what's changing, not on how much has been drawn.
- Completed moves are drawn into the buffer, and the buffer is shown with `window.paint_image`.

## Consequences

- PNG export comes straight from the buffer, and includes fills and pen modes. An SVG export from the event stream couldn't show those.
- How the buffer's resolution follows the canvas is decided in ADR 0003.
- Spike 02 checks that uploading the buffer every frame is fast enough and doesn't leak memory.
