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

The manual makes the pixel buffer close to essential rather than optional (see "Checked against the Berkeley Logo manual" below):

- **`FILL`** is a flood fill bounded by lines drawn earlier, so it needs to read pixels.
- **`PENREVERSE`** inverts each point along the turtle's path. gpui has no XOR or invert blend mode, so this can't be done with vector paths.
- **`PENERASE`** could be faked by drawing in the background colour, but that breaks if the background is changed later.

Plan on a pixel buffer for the finished drawing from the start, with vector paths only for the move in progress and the turtle sprite.

### 2. Keep the logo crate free of GUI code

Have the interpreter output a stream of turtle events in its own coordinate system. This must include everything the user should see happen, not only lines that leave ink:

- `Move { from, to, pen }`, which also covers pen-up moves, because those animate too. `pen` carries up/down, the mode (paint, erase or reverse), colour as resolved RGB, and size.
- `Turn { from, by }`, where `by` is the signed angle turned. Logo measures headings clockwise from north, and `RT 270` should visibly turn 270° clockwise, not take the short way round. So `from`/`to` alone isn't enough.
- `Arc { center, radius, start_heading, angle }`. `ARC` draws around the turtle without moving it.
- `Label { at, text }`, drawn with gpui's `shape_line` + `ShapedLine::paint`.
- `Fill`, `Filled { color, points }`, `Clean`, `SetBackground`, `ShowTurtle` / `HideTurtle`.
- `Print(text)`, so console output stays in step with the drawing (see "Turtle speed" below). Output from `TRACE` and `STEP` goes the same way.
- `Wait(duration)` and `SetSpeed(speed)`.

The interpreter does all the geometry: `SETSCRUNCH` scaling, splitting a move into several segments when it wraps at an edge, and `FENCE` stops. The GUI only scales and flips coordinates.

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
- **Step**: while paused, run exactly one line of code, the same way UCBLogo's `STEP` command does. All the drawing and output that line produces plays at the current speed, then the program pauses again with the next line highlighted.

#### How line stepping works

Before running each instruction line, the interpreter sends a `Line { span }` event.

- **Running normally:** `Line` is an ordinary event. The animator only uses it to move the highlight in the editor.
- **Paused or stepping:** the GUI sets a shared `stepping` flag, an atomic the interpreter reads. While it's set, the interpreter treats each `Line` like `Sync`: it waits for the animator to reply before running that line.

This matters because the channel lets the interpreter run several events ahead of the drawing. If `Line` were only a marker, a paused program could already have run a few lines past the highlighted one, including effects that aren't events, such as `MAKE` or file writes. Waiting at each line keeps the highlighted line and the program's real position in step, which a later variable inspector would also rely on.

Stepping therefore needs the source spans described below. They're required, not optional.
- **Stop**: set a cancel flag that the interpreter checks on every evaluation step, and drop the channel so a blocked interpreter wakes up and exits. This should behave like UCBLogo's `THROW "TOPLEVEL`, which ends all running procedures, but the program's own `CATCH` must not be able to intercept it.

The Pause button is not the same as Logo's `PAUSE` command. Logo's `PAUSE` stops the program inside a procedure and opens a prompt where its local variables can be inspected, and UCBLogo also opens one of these when the user presses its quit key. A later version could offer "Pause and inspect" that does the same, but the first version's Pause button only stops the animator.

### Highlighting the running code

Stepping depends on source spans, and every node in the parsed program needs one from the start. With them, the editor can also highlight the line the turtle is drawing at slow speeds, using the `Line` events.

For procedures, the span must point into the text where the procedure was defined. A program might call a procedure that was defined in the editor, typed at the console, or loaded from a file, so a span needs to record which source it came from as well as the position.

### How this fits with rendering

This combines well with "No saved drawing between frames" above:

- Completed moves get added to the saved drawing, either as batched paths or as a pixel buffer.
- Only the move in progress and the turtle sprite are drawn fresh each frame.

That way frame cost depends on what's changing, not on how much has been drawn so far.

### CLI

The CLI reads the same event stream with no timing and renders it to a file. `WAIT` and `SETSPEED` do nothing there.

### Open questions

- **Speed scale and units:** Python-style levels from 0 to 10, or a slider in pixels per second?
- **What counts as a "line":** UCBLogo's `STEP` works on the lines of a procedure body. Our editor has its own lines. Do we step by source line, by top-level instruction, or by each instruction inside a line? For example, `REPEAT 4 [FD 50 RT 90]` is one line: is it one step, or does stepping go into the brackets?
- **Does `STEP` itself work?** UCBLogo's `STEP proc` only steps named procedures. Should our `STEP` command behave the same way, and use the same mechanism as the Step button?
- **Channel size:** how big should it be? Small keeps the turtle closely in step. Larger makes Instant mode more efficient.

## Checked against the Berkeley Logo manual

These checks were made against `refs/logo/Berkeley-Logo-Reference-Manual.html`.

### What the manual confirms

- **`WAIT time`** delays for `time` 60ths of a second. It also flushes any buffered terminal output, and `WAIT 0` does only that. Because `Print` travels in the same event stream as drawing, output is never held back, so `WAIT 0` needs no special handling.
- **UCBLogo has no speed control.** The manual says it doesn't do "animation, parallelism, and multimedia". So `SETSPEED` / `SPEED` is our own extension. It doesn't clash with any UCBLogo name.
- **One turtle only.** "Multiple turtles, dynamic turtles, and collision detection are not supported." The single-turtle event stream is enough.

### What the manual adds to the plan

- **The graphics window has a fixed size in turtle coordinates.** The default turtle mode is **`WRAP`**, where the turtle reappears on the opposite edge. **`FENCE`** stops the turtle at the edge with an error, and **`WINDOW`** lets it move off-screen. The interpreter needs to know where the edges are, and UCBLogo sizes the window so that [-100 -100] to [100 100] fit at a 1:1 aspect ratio. Decision: the interpreter uses a fixed, configurable area in turtle coordinates. The GUI scales it to fit the canvas, so resizing the window never changes how a program behaves, and the GUI never needs to send the window size back to the interpreter.
- **Defaults:** black background, white pen. Colours can be numbers or RGB lists, where each RGB value is a percentage from 0 to 99, not 0 to 255. Colours 0 to 15 have fixed starting values, and `SETPALETTE` reassigns numbers from 8 up. Events carry the resolved RGB, so the GUI doesn't need to know about the palette.
- **`SETPENSIZE`** can take `[width height]`. Some implementations ignore the height and always use a square pen.
- **Turtle sprite:** an isosceles triangle. The turtle's position is the midpoint of the short side, and the triangle is drawn one step behind that position so it doesn't hide a line drawn at right angles to it.
- **`HOME` is a move** like any other, so it draws a line when the pen is down. `CLEARSCREEN` is `CLEAN` plus `HOME`.
- **`FILLED color [instructions]`** runs the instructions, remembers every point the turtle visits, then fills the resulting polygon. It's marked as wxWidgets-only. To animate it, play the moves first, then send the fill as one `Filled` event at the end.
- **Mouse queries** `MOUSEPOS`, `CLICKPOS` and `BUTTONP` are polled and return turtle coordinates. The GUI keeps the latest mouse state somewhere the interpreter can read it, such as `Arc<Mutex<_>>`, converted to turtle coordinates. Each query is a sync point, so a loop like `FOREVER [SETPOS MOUSEPOS]` runs at most once per frame, which is the right speed for following the mouse.
- **Keyboard input** with `READCHAR`, `READLIST` and `KEYP` reads from what the manual calls the "read stream". In the GUI that's the console, so the console needs a mode where the running program is waiting for input.
- **`REFRESH` / `NOREFRESH`** control whether Logo remembers turtle motions so it can redraw them later. With a pixel buffer we always keep the picture. `NOREFRESH` can be a no-op, or let us discard the motion history kept for zooming and export.
- **`SAVEPICT`, `LOADPICT` and `EPSPICT`** save and load pictures. `EPSPICT` ignores `FILL`, `PENERASE` and `PENREVERSE`, and an SVG export from the event stream would have the same limits. PNG export from the pixel buffer would not.
- **`TEXTSCREEN`, `SPLITSCREEN` and `FULLSCREEN`** rearrange the text and graphics windows. They can be events the GUI turns into layout changes. The CLI ignores them.
