# 3. Render the pixel buffer at device resolution, from a display list

Status: Accepted
Date: 2026-10-18. Updated 2026-10-21 after spike 02.

## Context

The finished drawing lives in a pixel buffer, because `FILL`, `PENREVERSE` and `PENERASE` need pixels (ADR 0010). The interpreter uses a fixed turtle area, and the GUI scales it to fit the canvas. But the canvas can be resized, and it can be on a HiDPI display. A pixel buffer at a fixed resolution would look blurry when scaled up and waste memory when scaled down.

## Decision

- **The GUI keeps a display list:** every drawing event since the last `CLEAN` (moves, arcs, shapes, fills, labels, background changes), in order. It's the event stream with the timing events left out.
- **The pixel buffer is rendered at the canvas's real size in device pixels.** Moves are drawn into it as they finish.
- **On resize or zoom, the GUI re-renders the pixel buffer from the display list,** on the UI thread. Spike 02 measured about 1 µs a segment at 2560 × 1600 with vello_cpu's worker threads (ADR 0010): 55 ms for 60,000 segments, and 139 ms for 141,000. That's fine up to about 100k segments. Re-rendering in the background, showing the old buffer scaled until the new one is ready, can wait until drawings get bigger.
- **The turtle area defaults to 1000 × 1000 turtle steps,** from -500 to 500 on both axes, and is configurable. The GUI scales it to fit the canvas without stretching. On a canvas that isn't square, the space outside the turtle area is shown in a slightly different shade, since the turtle can't draw there in `WRAP` or `FENCE` mode.
- **The CLI uses the same renderer** at whatever size is asked for, starting from the same display list.

## Consequences

- A flood `FILL` is replayed at the new resolution, so its edges can differ slightly from the original. That's acceptable.
- Memory for the display list grows with the drawing. That's fine for hundreds of thousands of events. Endless programs that draw forever without `CLEAN` will eventually need a limit, for example flattening old events into a pixel buffer at a fixed resolution.
- PNG export can render at any resolution, not only the canvas's.
