# Spike: streaming a pixel buffer into gpui

Status: ready-for-agent
Type: prototype

## Question

Can we draw into a full-resolution pixel buffer and show it on a gpui canvas every frame, fast enough, without leaking memory? This is the biggest performance risk in phase 1 (see ADR 0003).

## Build

On the canvas from spike 01, or a bare gpui window:

- A `tiny-skia` pixmap the size of the canvas in device pixels, around 2560 × 1600 on HiDPI.
- Each frame, draw a batch of anti-aliased lines into it (simulating a fast-drawing turtle), wrap it in a `RenderImage`, and show it with `window.paint_image`.
- Resize the window while it runs, and re-render the pixmap at the new size.

## Answer these

1. **Upload cost:** how long does a full-texture upload take per frame at 2560 × 1600? Can we hold 60 frames per second?
2. **Memory:** does gpui's image atlas grow without bound? We expect to need `window.drop_image` (`refs/zed/crates/gpui/src/window.rs:5175`) on the previous `RenderImage` each frame. Confirm memory stays flat over a few minutes.
3. **Pixel format:** what does `RenderImage` expect (BGRA or RGBA, premultiplied alpha or not)? What conversion is needed from `tiny-skia`'s premultiplied RGBA, and what does it cost?
4. **Fallback, if 1 or 2 fails:** draw recent lines as vector paths (`PathBuilder` + `paint_path`), and only flatten them into the pixel buffer every so often. Does that work?
5. **Flood fill with anti-aliasing:** draw an anti-aliased closed shape and flood-fill inside it. How bad are the halos or leaks at the edges, and what threshold fixes them? (`FILL` is phase 2, but it affects the choice of renderer now.)
6. **Opacity:** draw lines with an RGBA pen colour (our colour extension) and check they blend correctly.

## Done when

The questions are answered in a comment below, with measurements, and ADR 0003 is updated if anything changes.

## Comments
