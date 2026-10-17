# Plan 1: Canvas for the turtle

## Question

Does gpui or gpui-kit provide a canvas component that the Logo turtle can draw onto?

## Answer

**Yes. gpui has a canvas element that will work for the turtle. gpui-kit doesn't add its own canvas; it re-uses gpui's, and its `brush` example shows the pattern working.**

## What gpui provides

`gpui::canvas(prepaint, paint)` in `refs/zed/crates/gpui/src/elements/canvas.rs:10` is described as "a canvas element, meant for accessing the low level paint API".

- **Sizing:** it can be styled like any other element, for example `.size_full()`, so it fills whatever space the layout gives it.
- **Painting:** you get a callback with the canvas `Bounds<Pixels>` and a `&mut Window`. From there you paint directly:
  - **`window.paint_path(path, color)`** (`window.rs:4769`) draws a path. You build the path with **`PathBuilder`** (`path_builder.rs`), which has `stroke(width)` / `fill()`, `move_to`, `line_to`, `arc_to`, curves, `add_polygon`, dashes, and transforms (`translate`, `rotate`, `scale`).
  - **`paint_quad`** draws rectangles and backgrounds.
  - **`paint_svg`** could draw the turtle sprite.
  - **`paint_image`** draws `RenderImage` pixel buffers, which you can create from raw pixel frames (`RenderImage::new`, `assets.rs:61`).
  - **`with_content_mask`** clips drawing to the canvas edges.
- **Mouse input:** put the canvas inside a `div` that has `on_mouse_down/move/up` handlers and you get panning and zooming.
- **Example:** `refs/zed/crates/gpui/examples/painting.rs` is a working drawing demo built this way.

### How turtle commands map to `PathBuilder`

| Logo                  | gpui                              |
| --------------------- | --------------------------------- |
| `FD`, `BK`, `SETPOS`  | `line_to` (pen down)              |
| `PU` / `PD`           | `move_to` (pen up)                |
| `SETPENSIZE`          | `PathBuilder::stroke(width)`      |
| `SETPENCOLOR`         | colour passed to `paint_path`     |
| `ARC`                 | `arc_to`                          |
| `FILLED`              | `PathBuilder::fill()`             |

## What gpui-kit adds

It has no dedicated canvas component. Its components only call `gpui::canvas` to measure their own bounds.

However, `refs/gpui-kit/examples/brush/src/main.rs:196` (`render_canvas`) is a full drawing app built with gpui-kit:

- It stores the strokes on the view and draws them all in the paint callback with `PathBuilder::stroke` and `paint_path`.
- It has an optional grid, mouse handlers, and saves the canvas bounds with `on_prepaint`.
- Sliders, buttons and a colour picker around the canvas are gpui-kit components.

That's very close to the structure we want: a gpui-kit layout for the editor and console, with a raw gpui canvas for the turtle.

## Things to plan for

### 1. No saved drawing between frames

The canvas keeps nothing from one frame to the next, so the paint callback redraws every segment each time. That's fine for typical Logo drawings. For very large ones, such as deep recursive fractals with 100k+ segments, use one of these:

- Combine runs of segments with the same pen into a single path.
- Rasterize the drawing into a pixel buffer, either in the logo crate or with `tiny-skia`, and show it with `paint_image`.

A pixel buffer would also make `FILL` (flood fill) and saving to PNG straightforward.

### 2. Keep the logo crate free of GUI code

Have the interpreter output a stream of turtle events in its own coordinate system. This must include everything the user should see happen, not only lines that leave ink:

- `Move { from, to, pen }`, which also covers pen-up moves, because those animate too.
- `Turn { from, to }`.
- `Fill`, `Clear`, `ShowTurtle` / `HideTurtle`, pen colour and size changes.
- `Print(text)`, so console output stays in step with the drawing (see "Turtle speed" below).
- `Wait(duration)` and `SetSpeed(speed)`.

How each side uses it:

- The CLI can render the stream to SVG or PNG.
- The GUI draws it on the canvas.
- Logo's y-axis points up, so the GUI flips it when converting to screen positions.

## Turtle speed

### Goal

Let the user choose how fast the turtle draws, from "instant" down to slow enough to watch every step. They should be able to change speed, pause, single-step, and stop while a program is running.

### Core idea: run the interpreter and the animation separately

The interpreter never knows about speed or time. It runs as fast as it can and sends the events above to the GUI. The GUI has a component we'll call the **animator**, which plays those events onto the canvas at the chosen speed. Speed is therefore purely a GUI concern, and the logo crate and CLI stay simple.

Running the whole program first and then replaying the recording isn't enough:

- **Endless programs:** `FOREVER [FD 1 RT 1]` never finishes, so there'd be nothing to replay.
- **Input:** `READLIST`, `MOUSEPOS` and key presses have to happen while the user can see the current drawing.
- **Stop:** the stop button has to halt the real program, not just the playback.

### Architecture

```
interpreter thread ──(bounded channel of events)──▶ animator (UI thread) ──▶ canvas
        ▲                                              │
        └──────────── cancel flag, sync replies ───────┘
```

- **Interpreter on its own thread.** Use `std::thread` rather than gpui's background executor. The interpreter blocks when the channel is full, and a blocking task would tie up one of the executor's shared worker threads. Having our own thread also lets us give it a large stack (`std::thread::Builder::stack_size`) for deeply recursive Logo procedures.
- **A bounded channel keeps the two in step.** The channel holds only a limited number of events, so when it's full the interpreter blocks until the animator catches up. At slow speeds the interpreter therefore stays only a few events ahead of what's on screen, and pausing the animator automatically pauses the program.
- **Turtle queries never wait for the GUI.** `POS`, `HEADING`, `PENDOWNP` and similar are answered from the interpreter's own turtle state. Only the drawing lags behind.
- **Sync points.** Before anything that depends on what the user sees, the interpreter sends a `Sync` event with a one-shot reply channel and waits until the animator reaches it. That applies to reading input, reading the mouse or keyboard, and the end of the program.
- **Driving frames.** While events are queued or a move is part-way through, the animator calls `window.request_animation_frame()` (`window.rs:2730`) each frame, and stops calling it when idle. Each frame it uses `try_recv` to take as many events as the current speed and elapsed time allow, so no async plumbing is needed. gpui's docs say *decorative* animations should respect `App::reduce_motion`. This animation isn't decorative, because the user chose the speed, so it shouldn't be skipped.

### Speed model

- **Instant:** each frame, process events for a fixed time slice (around 8 ms) and paint once. This gives the same "everything appears at once" feel as other Logos, while the UI stays responsive on huge or endless programs.
- **Animated speeds:** a slider sets a distance per second and an angle per second. The turtle glides along each `FD` and visibly rotates on each `RT`/`LT`, rather than jumping from one command to the next. Small moves cost little, so recursive drawings speed up naturally when they get into fine detail.
- **Changes apply immediately**, even mid-move, because only the animator reads the speed.
- **`WAIT`** is a `Wait` event that the animator honours at every speed, including Instant, because a program that says `WAIT` means it.
- **`SETSPEED n` / `SPEED`** would be a Logo-level extension that sends a `SetSpeed` event, so the speed changes at exactly that point in the program. Python's `turtle.speed()` is a well-known example of the same idea.

### Controls

- **Speed slider**: changes only the animator's speed.
- **Pause / Resume**: the animator stops taking events, the channel fills up, and the interpreter blocks.
- **Step**: while paused, release exactly one event.
- **Stop**: set a cancel flag that the interpreter checks on every evaluation step, and drop the channel so a blocked interpreter wakes up and exits.

### Highlighting the running code (cheap if planned now)

If each event carries the source span of the instruction that produced it, the editor can highlight the line the turtle is drawing at slow speeds or in step mode. That needs spans on every AST node from the start, which is easy to build in now and painful to add later.

### How this fits with rendering

This combines well with "No saved drawing between frames" above:

- Completed moves get added to the saved drawing, either as batched paths or as a pixel buffer.
- Only the move in progress and the turtle sprite are drawn fresh each frame.

That way frame cost depends on what's changing, not on how much has been drawn so far.

### CLI

The CLI reads the same event stream with no timing and renders it to a file. `WAIT` and `SETSPEED` do nothing there.

### Open questions

- **Speed scale and units:** Python-style levels from 0 to 10, or a slider in pixels per second?
- **What one Step means:** one turtle event, or one Logo instruction? The second needs the interpreter to cooperate, not just the animator.
- **Hidden turtle:** should moves still animate when it's hidden, or jump to the end?
- **Channel size:** how big should it be? Small keeps the turtle closely in step. Larger makes Instant mode more efficient.
