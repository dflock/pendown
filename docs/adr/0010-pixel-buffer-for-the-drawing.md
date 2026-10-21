# 10. Keep the finished drawing in a pixel buffer, drawn with vello_cpu

Status: Accepted
Date: 2026-10-17 (recorded later, from commit 8104814 and `implementation-plan.md`). Updated 2026-10-21 after spike 02.

## Context

gpui's canvas keeps nothing between frames, so every visible line must be painted again every frame. Drawing only with vector paths would get slow for drawings with 100k+ segments. The Berkeley Logo manual also adds commands that need pixels:

- `FILL` is a flood fill bounded by lines drawn earlier, so it has to read pixels.
- `PENREVERSE` inverts each point along the turtle's path, and gpui has no XOR or invert blend mode.
- `PENERASE` can't be faked by drawing in the background colour, because the background can change later.

Spike 02 (`.scratch/phase-1-spikes/issues/02-pixel-buffer-streaming.md`) measured this on an Intel HD 530. Showing a 2560 × 1600 buffer every frame is fast enough. Drawing lines into it is the slow part: tiny-skia takes about 13 µs a segment at that size, and vello_cpu 3.5 µs on one thread, or 0.9 µs with 7 worker threads. The two look the same.

## Decision

- **The finished drawing lives in a pixel buffer** from the start of phase 1, even though `FILL` and the pen modes arrive in phase 5.
- **vello_cpu draws into it,** with worker threads (one fewer than the CPUs), drawing onto the existing pixels with `TargetInit::SrcOver`. The buffer itself is a plain premultiplied RGBA byte buffer that we own, and our own code (flood fill, PNG export) works on it directly, so changing rasterizer later stays contained.
- **Vector paths are only used for what's changing:** the move in progress and the turtle sprite. Frame cost then depends on what's changing, not on how much has been drawn.
- **Completed moves are drawn into the buffer, and the buffer is shown with `window.paint_image`,** following these rules:
  - **Draw with red and blue swapped.** `RenderImage` wants BGRA with straight alpha. The buffer is always opaque, so premultiplied and straight are the same, and swapped colours make the bytes BGRA already: only a copy is left. Swap back for PNG export and when reading a colour, such as `FILL`'s seed colour.
  - **Upload only the 256 × 256 tiles that changed,** each as its own `RenderImage`. Re-uploading the whole buffer drops about 1 frame in 10 to 30 fps when the turtle draws a lot.
  - **Drop each replaced image with `window.drop_image`,** or gpui's image atlas grows by hundreds of MiB a second. Drop it in the frame after it was last painted, never the same one: an image dropped in the frame that paints it is never shown.
- **Each move is its own stroke,** so where two moves with a translucent pen meet, the overlap is darker, and the line looks beaded. We accept that rather than merging runs of moves into one path.
- **Versions:** `vello_cpu = "0.3"` with the `multithreading` feature. For reading source, `refs/vello` is checked out at tag `sparse-strips-v0.3.0` (commit `f3000c8`), which matches the crate. Each crate's `.cargo_vcs_info.json` in `~/.cargo/registry/src/` names the commit it was published from.

## Consequences

- PNG export comes straight from the buffer, and includes fills and pen modes. An SVG export from the event stream couldn't show those.
- How the buffer's resolution follows the canvas is decided in ADR 0003.
- vello_cpu is young, its API is still changing, and it brings about 44 crates, including rayon and kurbo. Upgrades may need code changes.
- The colour docs (`SETPENCOLOR` in `pendown-logo.kdl`, the plan and ADR 0015) say that a translucent pen darkens where moves meet.
- Not yet checked on a real HiDPI display, on Wayland, or on any GPU other than an Intel HD 530.
