# Plan 1: Logo playground

## Scope and phases

The goal is a really good GUI for writing **new** Logo programs, not running the existing body of old Logo programs. So we implement a subset of UCBLogo, in phases.

The headline features are:

- You can set the speed of the turtle, and the source code is highlighted instruction by instruction as it executes, making a direct visual link between what the turtle is doing and the code.
- Extensive help, with reference/help panel generated from the our-logo.kdl
- Built-in tutorial & example code for each command

`our-logo.kdl` is the source of truth for which commands we support. It has every command from the Berkeley Logo manual, and each one has:

- **`obsolete` / `obsolete_reason`:** 103 commands are obsolete and will never be supported. Most of them are replaced by the GUI: printing out definitions, workspace listings, `EDIT`, `HELP`, `TRACE`/`STEP`, `PAUSE`, screen layout and console fonts. The rest are about old hardware or the UCBLogo installation (memory, file paths, aspect-ratio correction), or are superseded within the language (`TEST`/`IFTRUE`, `GOTO`).
- **`phase`:** 1, 2, 4 or 5 for the remaining 263 (257 from the manual plus 6 of our own), or `#null` for obsolete commands.

The file has the manual's description of every command, so the GUI's reference/help panel can be generated from it (phase 3).

### Phase 1: Turtle drawing (87 commands)

The smallest Logo that is fun to draw with:

- **Turtle and pen:** all movement and position commands, `ARC` (our version, see "Dialect and extensions"), `HOME`, show/hide turtle, `CLEAN`/`CLEARSCREEN`, `WRAP`/`WINDOW`/`FENCE`, and pen up/down, colour, size and background.
- **Shapes and colours (our extensions):** `CIRCLE`, `ELLIPSE`, `SQUARE` and `RECTANGLE` outlines, and every colour form (palette number, RGB or RGBA list, CSS name, hex) everywhere a colour is accepted.
- **Language:** `TO`/`END` with inputs and recursion, `MAKE`/`LOCAL`/`LOCALMAKE`/`THING`, `REPEAT`/`REPCOUNT`/`FOREVER`, `IF`/`IFELSE`, `STOP`/`OUTPUT`, `WAIT`, `PRINT`/`TYPE`/`SHOW`.
- **Arithmetic and logic:** arithmetic, `SIN`/`COS`/`ARCTAN`, `RANDOM`, comparisons, `AND`/`OR`/`NOT`.
- **Lists:** only what drawing needs, mainly for `POS`/`SETPOS`: `LIST`, `SENTENCE`, `FIRST`, `LAST`, `BUTFIRST`, `BUTLAST`, `ITEM`, `COUNT`, `EMPTYP`.
- **Syntax that isn't a command in the manual:** `END`, the infix operators `+ - * / = < > <= >= <>`, `:var` and `"word`, `[...]` lists, `(...)` for variable numbers of inputs, `;` comments. It's described in the manual's Introduction (chapter 1), which `our-logo.kdl` leaves out.

In the GUI and runtime, phase 1 includes:

- **The canvas and pixel buffer.** The pixel buffer is part of the design from the start, even though `FILL` and the pen modes that require it arrive in phase 2.
- **The turtle speed system:** speed slider, Pause, Step, Stop, highlighting the running instruction, and `SETSPEED`. This is the playground's headline feature.
- **Events:** `Move`, `Turn`, `Arc`, `Ellipse`, `Rect`, `Clean`, `SetBackground`, `ShowTurtle`/`HideTurtle`, `Print`, `Wait`, `SetSpeed`, `Instruction` and `Sync`.
- **The CLI** rendering the event stream to a file.

### Phase 2: Complete core language and richer drawing (88 commands)

- **Data:** the rest of the word, list and array commands, multi-dimensional arrays, `PUSH`/`POP`/`QUEUE`/`DEQUEUE`, type and membership predicates, `ASCII`/`CHAR`, `LOWERCASE`/`UPPERCASE`.
- **Control:** `RUN`/`RUNRESULT`, `FOR`, `WHILE`/`UNTIL`/`DO.WHILE`/`DO.UNTIL`, `CASE`/`COND`, `CATCH`/`THROW`/`ERROR`/`ERRACT`, `IGNORE`.
- **Higher-order procedures:** `APPLY`, `INVOKE`, `FOREACH`, `MAP`, `MAP.SE`, `FILTER`, `FIND`, `REDUCE`.
- **Arithmetic:** `EXP`/`LN`/`LOG10`, the radian trig functions, `ISEQ`/`RSEQ`, `RERANDOM`, `FORM`.
- **Drawing:** `FILL`, `FILLED`, `LABEL`/`SETLABELHEIGHT`/`LABELSIZE`, the pen modes `PENPAINT`/`PENERASE`/`PENREVERSE`, `SETPALETTE`/`PALETTE`, `SETPEN`/`PEN`. These add the `Fill`, `Filled` and `Label` events and the pen mode in `Move`.
- **Filled shapes (our extension):** `SETFILLCOLOR`/`FILLCOLOR`, which make the shapes draw filled.
- **Settings:** `NAME`, `GLOBAL`, `CASEIGNOREDP`, `FULLPRINTP`, `PRINTDEPTHLIMIT`/`PRINTWIDTHLIMIT`, `LOGOVERSION`.

### Phase 3: Help, examples and tutorial (no new commands)

Two of the headline features. They come after phase 2 so they cover the complete core language, and so the commands they document have settled down.

- **Help panel:** a reference panel generated from `our-logo.kdl`.
  - The GUI embeds the file at build time (`include_str!`) and parses it with the `kdl` crate (version 6, as used by the validator).
  - Browse by chapter and section, search by name and alias, and show help for the command under the editor's cursor.
  - It lists the commands the interpreter implements. Obsolete commands can be shown on request, with their `obsolete_reason`.
- **Our own descriptions:** each command's `desc` is the manual's text, which describes UCBLogo: it mentions wxWidgets, percentage RGB values, `SETSCRUNCH` and so on. Add a `help` node to each supported command with our own description, written for our dialect, with the `differs` notes folded in. `desc` stays as the manual's reference for whoever implements the command. Writing them is real work: about 175 commands in phases 1 and 2.
- **Errors for obsolete commands:** use `obsolete_reason` in the error when someone runs an obsolete command, for example "EDIT isn't supported: use the editor", not just "I don't know how to EDIT". This is cheap and could be done earlier.
- **Examples:** add one or more `example` nodes to each supported command. Each holds a short runnable program, with an optional title. This is different from `eg`, which only shows how to call the command. In the help panel, each example has **Run** and **Copy to editor** buttons.
- **Examples are tests:** a test runs every example through the interpreter and the CLI renderer and compares the result with a saved PNG. That keeps the documentation and the interpreter in agreement. Examples that use `RANDOM` need a fixed seed, so their output is repeatable.
- **Tutorial:** a sequence of lessons, each with text and code. The code loads into the editor and runs from there. The lessons follow the phase 1 commands: moving and turning, `REPEAT`, procedures, inputs and variables, recursion, then colours and shapes.
- **GUI:** a side panel next to the editor, with tabs for Help and Tutorial.

Open questions:

- **Markup:** what format the `help` text and tutorial text use, for example a small subset of markdown with code spans and links between commands.
- **Tutorial format:** a `tutorial.kdl` alongside `our-logo.kdl`, or a folder of markdown files?
- **Where examples run:** on the main canvas (replacing the user's drawing), or on a small canvas in the help panel?
- **Pictures:** show a pre-rendered picture of each example's output in the help panel? The snapshot tests already produce them.

### Phase 4: Interactive programs (14 commands)

- **Mouse:** `MOUSEPOS`, `CLICKPOS`, `BUTTONP`/`BUTTON`, and the `BUTTONACT`/`KEYACT` handlers.
- **Keyboard and console:** `READLIST`, `READWORD`, `READRAWLINE`, `READCHAR`, `READCHARS`, `KEYP`, `LINEP`, `CLEARTEXT`.

These need the parts of the speed design that synchronise with the GUI: sync points before input, shared mouse state in turtle coordinates, and a console mode for when the program is waiting for input. It's worth building them as a separate step.

### Phase 5: Advanced and niche (74 commands)

- **Language features:** objects (`KINDOF`, `ASK`, `TALKTO`, `USUAL`…), macros and backquote, property lists.
- **I/O:** file access, `SHELL`.
- **Low-level:** bitwise operations.
- **Introspection and metaprogramming:** `DEFINE`, `TEXT`, `COPYDEF`, `ARITY`, `PROCEDUREP` and similar.
- **Obscure library procedures:** `CASCADE`, `TRANSFER`, `CROSSMAP`.
- **Tokenizer details:** `PARSE`, `RUNPARSE`, `VBARREDP`, `RAWASCII`.
- **Settings:** `ALLOWGETSET`, `REDEFP`, `COMMANDLINE`.

## Dialect and extensions

We implement **UCBLogo**: it's the closest thing Logo has to a standard, it's the best documented, and most later Logos (FMSLogo, jsLogo) follow it. Where the dialects differ, it's mostly in commands we've marked obsolete or put in phase 5.

"Our Logo" is UCBLogo's core plus a few deliberate changes and extensions, listed here. They're recorded in `our-logo.kdl` too, so the help panel covers them:

- **New commands** have `extension #true` and a `source` naming where the idea came from. They're in a `shapes` section and in the pen sections of `graphics`.
- **Changed commands** keep the manual's `desc` and add a `differs` node describing how ours behaves.

References in `refs/`: `ucblogo-code` (the C source, for checking exact behaviour), `jslogo` (a UCBLogo-compatible interpreter with a canvas, the closest prior art to this playground) and `logo-rust`.

### Kept from UCBLogo: dynamic scope

A procedure can see the local variables of whatever procedure called it, not just globals. Every Logo works this way, and Logo teaching material relies on it.

- **Implementation:** a stack of frames. Looking up a variable walks from the current frame down through its callers to the globals. `MAKE` sets the nearest existing variable with that name, or creates a global if there isn't one.
- **GUI:** a later variable inspector should show the whole chain of frames, since that's what a running line can actually see.

### Colours

Everywhere a colour is accepted (`SETPENCOLOR`, `SETBACKGROUND`, `SETFILLCOLOR`, `SETPALETTE`, `FILLED`), all of these forms work:

- **Palette number:** 0 to 15, with UCBLogo's default colours: 0 black, 1 blue, 2 green, 3 cyan, 4 red, 5 magenta, 6 yellow, 7 white, 8 brown, 9 tan, 10 forest, 11 aqua, 12 salmon, 13 purple, 14 orange, 15 grey. `SETPALETTE` (phase 2) can redefine numbers from 8 up, as in UCBLogo.
- **RGB list:** `[r g b]` with each value **from 0 to 255**. This is a deliberate change: UCBLogo uses percentages from 0 to 99, but 0 to 255 is what everyone knows from CSS and image editors. FMSLogo also uses 0 to 255.
- **RGBA list:** `[r g b a]`, where `a` is the opacity, also from 0 to 255 (0 is invisible, 255 is solid). The pixel buffer blends it.
- **CSS colour name:** `SETPENCOLOR "red`, `SETBACKGROUND "cornflowerblue`, the 148 CSS named colours, case-insensitive. jsLogo does the same.
- **Hex:** `"#ff8800`, plus the CSS short form `"#f80` and the forms with opacity, `"#ff880080` and `"#f808`. `#` is allowed in a Logo word, so no new syntax is needed.

The forms can't be confused: a number is always a palette index, a list is always RGB or RGBA, a word starting with `#` is hex, and any other word is a name. An unknown name or bad hex is an error, and an unknown name suggests the closest known names.

The background is always drawn solid, so the opacity of a background colour is ignored.

Reporters (`PENCOLOR`, `BACKGROUND`, `FILLCOLOR`) output the colour in the form it was set, so `SETPENCOLOR PENCOLOR` always works. Events carry the resolved RGBA, so the GUI and CLI never need to know about palettes, names or hex.

### `ARC`: the turtle walks along the arc

In UCBLogo, `ARC angle radius` draws an arc *around* the turtle, with the turtle as the centre, and the turtle doesn't move. That surprises nearly everyone. Ours does what you'd draw by hand:

- **`ARC angle radius`** moves the turtle along an arc of a circle with that radius, turning through `angle` degrees as it goes. A positive angle curves right (clockwise, like `RT`), and a negative angle curves left.
- It finishes at the end of the arc, with its heading changed by `angle`. So `ARC 360 50` draws a full circle and returns the turtle to where it started, and `REPEAT 4 [ARC 90 50]` also draws a full circle.
- It draws only when the pen is down, like `FD`, and it's affected by `WRAP`/`FENCE` like any other move.
- For a positive angle, it's the same as `REPEAT angle [FD (2 * PI * radius / 360) RT 1]`, but drawn as a smooth curve.

UCBLogo programs that use `ARC` will draw differently. That's accepted. `CIRCLE` covers the "around the turtle" case.

### Shapes: `CIRCLE`, `ELLIPSE`, `SQUARE` and `RECTANGLE`

From LibreLogo and FMSLogo. Shapes are drawn **centred on the turtle** and turn with its heading, and the turtle doesn't move. All four are phase 1.

- **`CIRCLE radius`**.
- **`ELLIPSE across along`:** `across` is the radius from side to side relative to the turtle, and `along` is the radius in the direction it's heading. At heading 0 that's the x radius then the y radius.
- **`SQUARE side`**, and **`(SQUARE side corner)`** for rounded corners with radius `corner`.
- **`RECTANGLE across along`**, and **`(RECTANGLE across along corner)`** for rounded corners. `across` and `along` are the full width and height, like `SQUARE`'s side.

Circles and ellipses take radii, and squares and rectangles take side lengths, because that's how people usually measure each. The optional corner radius uses UCBLogo's usual parentheses for extra inputs, as with `(SENTENCE a b c)`.

- **Outline:** drawn with the current pen (colour, size and, in phase 2, mode) when the pen is down.
- **Fill (phase 2):** `SETFILLCOLOR colour` sets a fill colour, and `SETFILLCOLOR "none` turns filling off. `"none` is the default. When a fill colour is set, shapes are filled, even when the pen is up. So pen up with a fill colour gives a filled shape with no outline.
- **Events:** `Ellipse { center, radii, heading, pen, fill }`, which `CIRCLE` uses too, with equal radii, and `Rect { center, size, corner, heading, pen, fill }` for `SQUARE` and `RECTANGLE`. At animated speeds the animator can draw the outline sweeping round, then the fill.

Filling turtle-drawn polygons is already covered by UCBLogo's `FILLED colour [instructions]` and `FILL` (phase 2).

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
| `CIRCLE`, `ELLIPSE`   | `arc_to` (or a polygon), `fill()` |
| `SQUARE`, `RECTANGLE` | `line_to` + `arc_to` corners      |
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

- `Move { from, to, pen }`, which also covers pen-up moves, because those animate too. `pen` carries up/down, the mode (paint, erase or reverse), colour as resolved RGBA, and size.
- `Turn { from, by }`, where `by` is the signed angle turned. Logo measures headings clockwise from north, and `RT 270` should visibly turn 270° clockwise, not take the short way round. So `from`/`to` alone isn't enough.
- `Arc { center, radius, start_heading, angle, pen }`. Our `ARC` moves the turtle along the arc (see "Dialect and extensions"), so the animator moves the turtle sprite with it.
- `Ellipse { center, radii, heading, pen, fill }` for `CIRCLE` and `ELLIPSE`, and `Rect { center, size, corner, heading, pen, fill }` for `SQUARE` and `RECTANGLE`. The turtle doesn't move.
- `Label { at, text }`, drawn with gpui's `shape_line` + `ShapedLine::paint`.
- `Fill`, `Filled { color, points }`, `Clean`, `SetBackground`, `ShowTurtle` / `HideTurtle`.
- `Print(text)`, so console output stays in step with the drawing (see "Turtle speed" below).
- `Wait(duration)` and `SetSpeed(speed)`.

The interpreter does all the geometry: splitting a move into several segments when it wraps at an edge, and `FENCE` stops. (`SETSCRUNCH` scaling is obsolete.) The GUI only scales and flips coordinates.

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
- **Step**: while paused, run exactly one line of code, the same way UCBLogo's `STEP` command does. All the drawing and output that line produces plays at the current speed, with each instruction highlighted as it runs, then the program pauses again at the start of the next line.
- **Stop**: set a cancel flag that the interpreter checks on every evaluation step, and drop the channel so a blocked interpreter wakes up and exits. This should behave like UCBLogo's `THROW "TOPLEVEL`, which ends all running procedures, but the program's own `CATCH` must not be able to intercept it.

#### How highlighting and stepping work

Before running each instruction, the interpreter sends an `Instruction { span, starts_line }` event. `span` covers just that instruction, so in `REPEAT 4 [FD 50 RT 90]` the highlight moves from `REPEAT` to `FD 50` to `RT 90` and round again. `starts_line` is set on the first instruction run from each line.

- **Running normally:** `Instruction` is an ordinary event. The animator only uses it to move the highlight in the editor.
- **Paused or stepping:** the GUI sets a shared `stepping` flag, an atomic the interpreter reads. While it's set, the interpreter treats each `Instruction` with `starts_line` like `Sync`: it waits for the animator to reply before running that line.

This matters because the channel lets the interpreter run several events ahead of the drawing. If `Instruction` were only a marker, a paused program could already have run a few lines past the highlighted one, including effects that aren't events, such as `MAKE` or file writes. Waiting at each line keeps the highlight and the program's real position in step, which a later variable inspector would also rely on.

Stepping therefore needs the source spans described below. They're required, not optional.

The Pause button is not the same as Logo's `PAUSE` command. Logo's `PAUSE` stops the program inside a procedure and opens a prompt where its local variables can be inspected, and UCBLogo also opens one of these when the user presses its quit key. Logo's `PAUSE` and `CONTINUE` are marked obsolete. A later version could offer "Pause and inspect" in the GUI instead, but the first version's Pause button only stops the animator.

### Highlighting the running code

Highlighting and stepping depend on source spans, and every node in the parsed program needs one from the start. With them, the editor highlights the instruction the turtle is carrying out, using the `Instruction` events. At slow speeds that's the direct link between the code and the drawing.

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
- ~~What does the highlight show?~~ The running instruction, not the whole line.
- **What does Step step over:** Step currently runs one source line, so `REPEAT 4 [FD 50 RT 90]` is one step, with the highlight moving through its instructions as it plays. Would a second "Step instruction" button, which stops inside the brackets, be useful as well?
- ~~Does `STEP` itself work?~~ No: `STEP`, `TRACE` and their relatives are marked obsolete in `our-logo.kdl`. Stepping and tracing are debugger controls in the GUI.
- **Channel size:** how big should it be? Small keeps the turtle closely in step. Larger makes Instant mode more efficient.

## Checked against the Berkeley Logo manual

These checks were made against `refs/logo/Berkeley-Logo-Reference-Manual.html`.

### What the manual confirms

- **`WAIT time`** delays for `time` 60ths of a second. It also flushes any buffered terminal output, and `WAIT 0` does only that. Because `Print` travels in the same event stream as drawing, output is never held back, so `WAIT 0` needs no special handling.
- **UCBLogo has no speed control.** The manual says it doesn't do "animation, parallelism, and multimedia". So `SETSPEED` / `SPEED` is our own extension. It doesn't clash with any UCBLogo name.
- **One turtle only.** "Multiple turtles, dynamic turtles, and collision detection are not supported." The single-turtle event stream is enough.

### What the manual adds to the plan

- **The graphics window has a fixed size in turtle coordinates.** The default turtle mode is **`WRAP`**, where the turtle reappears on the opposite edge. **`FENCE`** stops the turtle at the edge with an error, and **`WINDOW`** lets it move off-screen. The interpreter needs to know where the edges are, and UCBLogo sizes the window so that [-100 -100] to [100 100] fit at a 1:1 aspect ratio. Decision: the interpreter uses a fixed, configurable area in turtle coordinates. The GUI scales it to fit the canvas, so resizing the window never changes how a program behaves, and the GUI never needs to send the window size back to the interpreter.
- **Defaults:** black background, white pen. Colours can be numbers or RGB lists, where each RGB value is a percentage from 0 to 99, not 0 to 255. Colours 0 to 15 have fixed starting values, and `SETPALETTE` reassigns numbers from 8 up. Events carry the resolved RGB, so the GUI doesn't need to know about the palette. (We use 0 to 255 instead, and add opacity, CSS names and hex: see "Dialect and extensions".)
- **`SETPENSIZE`** can take `[width height]`. Some implementations ignore the height and always use a square pen.
- **Turtle sprite:** an isosceles triangle. The turtle's position is the midpoint of the short side, and the triangle is drawn one step behind that position so it doesn't hide a line drawn at right angles to it.
- **`HOME` is a move** like any other, so it draws a line when the pen is down. `CLEARSCREEN` is `CLEAN` plus `HOME`.
- **`FILLED color [instructions]`** runs the instructions, remembers every point the turtle visits, then fills the resulting polygon. It's marked as wxWidgets-only. To animate it, play the moves first, then send the fill as one `Filled` event at the end.
- **Mouse queries** `MOUSEPOS`, `CLICKPOS` and `BUTTONP` are polled and return turtle coordinates. The GUI keeps the latest mouse state somewhere the interpreter can read it, such as `Arc<Mutex<_>>`, converted to turtle coordinates. Each query is a sync point, so a loop like `FOREVER [SETPOS MOUSEPOS]` runs at most once per frame, which is the right speed for following the mouse.
- **Keyboard input** with `READCHAR`, `READLIST` and `KEYP` reads from what the manual calls the "read stream". In the GUI that's the console, so the console needs a mode where the running program is waiting for input.
- **Obsolete, not supported** (see `our-logo.kdl`): `REFRESH` / `NOREFRESH` (we always keep the picture), `SAVEPICT` / `LOADPICT` / `EPSPICT` (replaced by GUI export), and `TEXTSCREEN` / `SPLITSCREEN` / `FULLSCREEN` (the GUI layout is fixed). Note for export: `EPSPICT` ignored `FILL`, `PENERASE` and `PENREVERSE`, and an SVG export from the event stream would have the same limits. PNG export from the pixel buffer would not.
