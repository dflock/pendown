# Spike: streaming a pixel buffer into gpui

Status: done
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

### 2026-10-21: spike results

Code: `spikes/02-pixel-buffer-streaming/`. All spikes now share one build directory, `spikes/target`, set in `spikes/.cargo/config.toml`.

- **The window:** `cargo run --release`. `SPIKE_AUTOTEST=1` runs 16 timed phases and saves `autotest.log` and the screenshots. `SPIKE_SOAK=180` streams frames for 3 minutes and writes memory figures every 10 s to `soak.log`.
- **Offline measurements:** `cargo run --release --bin offline --no-default-features --features vello`. These need no window and write `offline.log`, `fill-edges.png` and `rasterizers.png`.

Setup:
- A random-walk turtle draws a batch of anti-aliased segments (pen width 1.5 turtle steps, round caps) into a pixmap every frame, and the pixmap is shown with `window.paint_image`.
- The test machine is an Intel HD 530 (integrated) with 1920 × 1080 monitors at scale 1. To get the 2560 × 1600 buffer a HiDPI display needs, the "2x" phases draw a pixmap twice the canvas's device size and let gpui scale it down. The upload costs the same either way. `GPUI_X11_SCALE_FACTOR=2` gives a real 2× window, but a 2× window can't be bigger than a 1080p monitor.

**Summary: yes, with three changes to the plan.**
1. A full 2560 × 1600 upload every frame fits in a 60 fps frame. The bottleneck is drawing the segments, not uploading them.
2. vello_cpu draws 3–14 times faster than tiny-skia, so use it instead.
3. Update only the 256 × 256 tiles that changed, rather than uploading the whole buffer.

#### 1. Upload cost

| Phase (2560 × 1600 unless 1x) | Frame interval p50 / p95 | Our CPU per frame |
|---|---|---|
| Static: same image every frame | 16.66 / 16.76 ms | ~0 |
| Full upload, 20 segments a frame | 16.66 / 16.79 ms | draw 0.3 + copy 1.4 + paint 1.5 ms |
| Full upload, tiny-skia, 500 a frame | 33.29 / 33.45 ms | draw 7.5 + 1.4 + 1.4 ms |
| Full upload, vello_cpu, 500 a frame | 16.67 / 33.24 ms | draw 2.9 + 2.0 + 2.3 ms |
| Full upload, vello_cpu, 2000 a frame | 16.68 / 33.36 ms | draw 3.8 + 2.3 + 1.9 ms |
| Tiles, vello_cpu, 500 a frame | 16.67 / 16.76 ms | draw 2.3 + tiles 1.4 + paint 1.1 ms |
| 1x (1280 × 800) full upload, tiny-skia, 500 a frame | 16.67 / 16.77 ms | draw 3.7 + 0.5 + 0.5 ms |

- **A full upload fits easily.** With almost nothing to draw, a new 2560 × 1600 image every frame holds 60 fps. Our CPU cost is about 3 ms: our copy of the 16 MB buffer, plus gpui's own copy inside `paint_image`. gpui's atlas copies the bytes again before `queue.write_texture`.
- **Busy frames drop to 30 fps.** With heavy drawing, about 1 frame in 10 takes 33 ms, even though our CPU work is only 7 ms. It's probably the GPU copy competing for memory bandwidth on the integrated GPU, since vello's worker threads are busy at the same time.
- **Tiles fix the dropped frames.** The 256 × 256 tiles each frame touched are re-uploaded as separate small images, which share gpui's atlas textures. That held a steady 60 fps. A full upload is still needed after a resize or `CLEAN`, and that's fine.
- **The sampler is linear.** At 1× the tiles are drawn 1:1 and no seams show (`screenshot-2-tiles.png`). When an image is drawn scaled, the sampler could blend in pixels from a neighbouring atlas tile. So draw the buffer at device resolution (ADR 0003) and never scale tiles.

#### 2. Memory

- **`drop_image` is required.** Each new `RenderImage` has a new id, and the atlas keeps every id it has seen. Without `drop_image`, 2 s at 1280 × 800 left 112 images in the atlas: resident memory grew by 430 MiB and GPU buffers by 545 MiB. Dropping them brought GPU buffers back to 169 MiB.
- **With `drop_image`, memory is flat.** The soak test streamed 2560 × 1600 frames for 180 s, with vello_cpu drawing 500 segments a frame and a `CLEAN` every 10 s. Resident memory was 178 MiB and GPU buffers 140 MiB at every sample from 10 s to 170 s (`soak.log`).
- **How it works:** an image bigger than gpui's 1024 × 1024 atlas pages gets a texture of its own. `drop_image` frees that texture once nothing else uses it.
- **Gotcha: drop the previous frame's image, never the one painted this frame.** `drop_image` also discards the pending upload, so an image dropped in the same frame it's painted simply doesn't appear. The first blend test showed this.

#### 3. Pixel format

- **What gpui expects:** `RenderImage` holds `image::Frame`s built from an `RgbaImage`, but the bytes must be in **BGRA** order, with **straight (not premultiplied) alpha**. The sprite shader premultiplies when the window surface needs it.
  - gpui's atlas is `Bgra8Unorm` when the adapter supports it.
  - Otherwise it's `Rgba8Unorm`, and gpui swaps the bytes itself on upload.
  - Source: `gpui/src/assets.rs`, plus `wgpu_atlas.rs` and `shaders.wgsl` (`blend_color`) in `gpui_wgpu`.
- **What tiny-skia and vello_cpu produce:** premultiplied RGBA. Our buffer is opaque everywhere, because the background is always painted, so premultiplied and straight are the same. Only red and blue need swapping.
- **What the conversion costs at 2560 × 1600:**

| Conversion | p50 |
|---|---|
| Swap red and blue, a pixel at a time | 5.9–7.5 ms |
| The same, a `u32` at a time | 4.6–6.1 ms (3.2 ms with `target-cpu=native`) |
| Demultiply as well (only needed with transparency) | 28–29 ms |
| **Draw with red and blue swapped, then just copy** | **1.2 ms** |

- **Recommendation: draw with red and blue swapped.** Blending treats every channel the same way, so the buffer's bytes are already BGRA, and only a copy is left. Swap them back for PNG export and when reading colours, for example for `FILL`'s seed colour. This applies to vello_cpu as much as tiny-skia.

#### 4. The vector-path fallback

- **It works, but we don't need it.** It saves uploads, and uploads aren't the problem. Drawing the segments is.
- **It makes frames worse.** Flattening into the buffer every 30 frames turns the drawing into spikes. At 20 segments a frame, a flatten took up to 38 ms, and those frames took 50–67 ms. At 500 a frame, a flatten took 200 ms.
- **Vector paths stay for what ADR 0010 already says:** the move in progress and the turtle sprite.

#### 5. Flood fill at anti-aliased edges

`bin/offline.rs` draws closed shapes one side at a time, as Logo moves draw them, and fills them from the centre with a 4-connected scanline fill. The results are in `offline.log`, and `fill-edges.png` shows zoomed crops. Two fill options:
- **Tolerance:** a pixel is filled if no channel differs from the seed pixel by more than this.
- **Edge blending:** after the fill, each unfilled pixel next to it is recoloured. The fill estimates how much of the seed colour the pixel holds, against its least seed-like neighbour, and replaces that share with the fill colour.

| Shape | Tolerance 0, plain | Tolerance 64, plain | Tolerance 128, plain | **Tolerance 0, blended edges** |
|---|---|---|---|---|
| Black circle, 1 px | halo 394 | halo 194 | **leaks** | no halo, no leaks |
| Black circle, 3 px | halo 377 | halo 126 | clean | clean |
| Star, 1 px | halo 537 | halo 268 | **leaks** | halo 2 |
| Square at 30°, 0.5 px | halo 812 | **leaks** | **leaks** | halo 145 |
| Pale grey circle, 1 px | halo 392 | **leaks** (from 32) | **leaks** | clean |
| Red line on blue, 1 px | halo 394 | halo 110 | **leaks** | clean |

- **Tolerance is the wrong tool.** A tolerance big enough to remove the halo lets the fill leak through thin or pale lines. It also eats into the line itself, leaving it dashed (`fill-edges.png`). The safe tolerance depends on the contrast between the line and the background, which the fill can't know.
- **Recommendation for phase 2:** exact match (tolerance 0, as UCBLogo does) plus edge blending. That gave no leaks and no visible halo for every line of 1 px or more, including low contrast and colour on colour. Only 0.5 px lines keep a faint halo.
- **Edge blending costs one extra pass** over the pixels next to the filled area.
- **The renderer choice doesn't matter for fill.** The fill works on the buffer's bytes, so it's the same with tiny-skia or vello_cpu.

#### 6. Opacity

Red `[255 0 0 128]` over blue `[0 0 255]`:

| | Expected | tiny-skia | gpui `paint_path` |
|---|---|---|---|
| One stroke | (128, 0, 127) | (128, 0, 127) | (128, 0, 127) |
| Corner where two separate strokes overlap | (192, 0, 63) | (192, 0, 64) | (192, 0, 63) |
| Corner of one two-segment path | (128, 0, 127) | (128, 0, 127) | — |

- **Blending is correct.** Both blend straight src-over in sRGB space and agree to within 1, so the move in progress (a gpui path) and the finished move (in the buffer) look the same (`screenshot-3-blend.png`).
- **Joints darken, and that needs a decision.** Every Logo move is its own stroke, so where two translucent moves meet, the overlap is darker. A translucent pen therefore draws a "beaded" line. Drawing a run of moves as one path avoids that, but it complicates drawing a move at a time. Suggestion: accept the beads in phase 1 and note it in the colour docs. They're the natural result of drawing one move at a time.

#### The real bottleneck: drawing the segments

These are averages per segment from the offline benchmark. Segments are random-walk moves of 5–40 turtle steps, 1.5 steps wide.

| | 1280 × 800 | 2560 × 1600 |
|---|---|---|
| tiny-skia, one path per segment, round caps | 7.3 µs | 12.6–13.3 µs |
| tiny-skia, butt caps | 5.4 µs | 10.8 µs |
| tiny-skia, one path per run of the same colour | 5.5 µs | 10.0 µs |
| tiny-skia, 1 px wide | 2.5 µs | 4.0 µs |
| vello_cpu, one thread | | 3.5–3.8 µs |
| vello_cpu, 7 worker threads | | 0.9 µs |

- **`target-cpu=native` made tiny-skia's drawing *slower*** (18.6 against 13.2 µs), so don't use it.
- **Redrawing after a resize, with 60,000 segments at 2560 × 1600:**
  - tiny-skia: 800 ms;
  - tiny-skia in 8 horizontal bands, one thread each: 215 ms;
  - vello_cpu with 7 worker threads: **55 ms**.
- **Resizing in the window:** vello_cpu re-rendered 141,000 segments in 139 ms. tiny-skia took 1,016 ms for 73,500, and the window froze for a second.
- **The output looks the same.** vello_cpu's anti-aliasing is indistinguishable from tiny-skia's: the whole-image RMS difference is 1.1%, and `rasterizers.png` shows zoomed crops of each. vello_cpu can draw onto the existing pixels (`TargetInit::SrcOver`), straight into the same premultiplied RGBA buffer.
- **Costs of vello_cpu:**
  - it's young (0.3.0);
  - its API is still changing;
  - it brings about 44 more crates than tiny-skia's 19, including rayon and kurbo.

#### Changes this suggests

- **ADR 0010:**
  - Make vello_cpu the rasterizer, with 7 worker threads, instead of tiny-skia. The buffer is a plain premultiplied RGBA byte buffer that our own code (flood fill, PNG export) works on directly, so swapping rasterizers later is contained.
  - Record the upload rules: drop the previous frame's image, draw with red and blue swapped, and update only the 256 × 256 tiles that changed.
- **ADR 0003:** re-rendering on a resize takes about 1 µs per segment with vello_cpu's threads. Up to about 100k segments it can happen on the UI thread. Moving it to the background can wait until drawings get bigger.
- **Phase 2:** `FILL` uses exact match plus edge blending.
- **For a human to decide:**
  - accept beaded joints for translucent pens;
  - accept a pre-1.0 vello_cpu as a dependency.
- **Not tested:** a real HiDPI display (the buffer was oversampled instead), Wayland, and any GPU other than an Intel HD 530.

### 2026-10-21: closed

Both decisions accepted: vello_cpu is the rasterizer, and translucent pens keep their beaded joints. ADR 0010 now records vello_cpu and the upload rules, and ADR 0003 the re-render cost. The joints are noted in `SETPENCOLOR` in `pendown-logo.kdl`, in `implementation-plan.md` and in ADR 0015. `refs/vello` is checked out at `sparse-strips-v0.3.0`, the commit vello_cpu 0.3.0 was published from.
