# Plan 1: Pendown

## Scope and phases

**Pendown** is a Logo playground: crates `pendown` (the GUI app), `pendown-cli` and `pendown-logo` (the interpreter library). See ADR 0019.

The goal is a really good GUI for writing **new** Logo programs, not running the existing body of old Logo programs. So we implement a subset of UCBLogo, in phases.

The headline features are:

- You can set the speed of the turtle, and the source code is highlighted instruction by instruction as it executes, making a direct visual link between what the turtle is doing and the code.
- Extensive help, with reference/help panel generated from the pendown-logo.kdl
- Built-in tutorial & example code for each command

`pendown-logo.kdl` is the source of truth for which commands we support. It has every command from the Berkeley Logo manual, and each one has:

- **`obsolete` / `obsolete_reason`:** 103 commands are obsolete and will never be supported. Most of them are replaced by the GUI: printing out definitions, workspace listings, `EDIT`, `HELP`, `TRACE`/`STEP`, `PAUSE`, screen layout and console fonts. The rest are about old hardware or the UCBLogo installation (memory, file paths, aspect-ratio correction), or are superseded within the language (`TEST`/`IFTRUE`, `GOTO`).
- **`phase`:** 1, 2, 6 or 7 for the remaining 265 (257 from the manual plus 8 of our own), or `#null` for obsolete commands.

The file has the manual's description of every command, so the GUI's reference/help panel can be generated from it (phase 5).

### Phase 1: Turtle drawing (90 commands)

Done: see `.scratch/phase-1/`.

The smallest Logo that is fun to draw with:

- **Turtle and pen:** all movement and position commands, `ARC` (our version, see "Dialect and extensions"), `HOME`, show/hide turtle, `CLEAN`/`CLEARSCREEN`, `WRAP`/`WINDOW`/`FENCE`, and pen up/down, colour, size and background.
- **Shapes and colours (our extensions):** `CIRCLE`, `ELLIPSE`, `SQUARE` and `RECTANGLE` outlines, and every colour form (palette number, RGB or RGBA list, CSS name, hex) everywhere a colour is accepted.
- **Language:** `TO`/`END` with inputs and recursion, `MAKE`/`LOCAL`/`LOCALMAKE`/`THING`, `REPEAT`/`REPCOUNT`/`FOREVER`, `IF`/`IFELSE`, `STOP`/`OUTPUT`, `WAIT`, `PRINT`/`TYPE`/`SHOW`.
- **Arithmetic and logic:** arithmetic, `SIN`/`COS`/`ARCTAN`, `RANDOM`, comparisons, `AND`/`OR`/`NOT`.
- **Lists and words:** only what drawing needs, mainly for `POS`/`SETPOS`: `LIST`, `SENTENCE`, `FIRST`, `LAST`, `BUTFIRST`, `BUTLAST`, `ITEM`, `COUNT`, `EMPTYP`, and `WORD`.
- **Syntax that isn't a command in the manual:** `END`, the infix operators `+ - * / = < > <= >= <>`, `:var` and `"word`, `[...]` lists, `(...)` for variable numbers of inputs, `;` comments. It's described in the manual's Introduction (chapter 1), which `pendown-logo.kdl` leaves out.

In the GUI and runtime, phase 1 includes:

- **The canvas and pixel buffer.** The pixel buffer is part of the design from the start, even though `FILL` and the pen modes that require it arrive in phase 7.
- **The turtle speed system:** speed slider, Pause, Step, Stop, highlighting the running instruction, and `SETSPEED`. This is the playground's headline feature.
- **Events:** `Move`, `Turn`, `Arc`, `Ellipse`, `Rect`, `Clean`, `SetBackground`, `ShowTurtle`/`HideTurtle`, `Print`, `Wait`, `SetSpeed`, `Instruction` and `Sync`.
- **The CLI** rendering the event stream to a file.

### Phase 2: Complete core language and filled shapes (76 commands)

Done: see `.scratch/phase-2/` and ADR 0021.

- **Data:** the rest of the word, list and array commands, multi-dimensional arrays, `PUSH`/`POP`/`QUEUE`/`DEQUEUE`, type and membership predicates, `ASCII`/`CHAR`, `LOWERCASE`/`UPPERCASE`.
- **Control:** `RUN`/`RUNRESULT`, `FOR`, `WHILE`/`UNTIL`/`DO.WHILE`/`DO.UNTIL`, `CASE`/`COND`, `CATCH`/`THROW`/`ERROR`/`ERRACT`, `IGNORE`.
- **Higher-order procedures:** `APPLY`, `INVOKE`, `FOREACH`, `MAP`, `MAP.SE`, `FILTER`, `FIND`, `REDUCE`.
- **Arithmetic:** `EXP`/`LN`/`LOG10`, the radian trig functions, `ISEQ`/`RSEQ`, `RERANDOM`, `FORM`.
- **Drawing:** `SETPALETTE`/`PALETTE`. The rest of UCBLogo's drawing commands, which work on pixels or need text, are in phase 7.
- **Filled shapes (our extension):** `SETFILLCOLOR`/`FILLCOLOR`, which make the shapes draw filled.
- **Settings:** `NAME`, `GLOBAL`, `CASEIGNOREDP`, `FULLPRINTP`, `PRINTDEPTHLIMIT`/`PRINTWIDTHLIMIT`, `LOGOVERSION`.

### Phase 3: Performance (no new commands)

Done: see `.scratch/phase-3/` and ADR 0020. The cause was the window, not the interpreter: every frame laid out and painted the whole window, and slow runs drew 60 frames a second when the picture changed far less often. A slow run now uses about a fifth of the UI time it did, and Instant is as fast as before.

While a program runs, Pendown keeps one CPU core fully busy, even at slow speeds where the turtle barely moves. Rather than a one-off fix, this phase builds in measurement that stays (ADR 0020), so the cause shows up now and regressions show up later.

- **A `metrics` module** in each crate that does work: counters and timings cheap enough to leave in, kept as atomics and read by whoever reports them.
  - **On by default in debug builds, off in release.** `PENDOWN_METRICS=1` turns it on in a release build, and `PENDOWN_METRICS=0` off in a debug one. Off, each measurement is one check of a flag.
- **What's measured:**
  - **UI thread, per frame:** the time spent rendering, in the animator, drawing into the pixel buffer and uploading the canvas image, the events taken, and how often the window re-renders.
  - **Interpreter:** instructions a second, the time spent waiting in lockstep or on a full channel, and the time spent running.
  - **CPU time for each thread:** the UI, the interpreter and vello_cpu's workers. That answers "who is using the core" directly. It's read from `/proc/self/task` on Linux at first; other platforms come later.
- **Where it's shown:**
  - At the end of each run, a one-line summary on stderr, such as `run 3.2 s: 190 frames, UI 4.1 ms a frame, CPU UI 35% interpreter 2% vello 0%`.
  - In the GUI, an overlay of the live numbers, toggled by a key.
  - The CLI prints its summary on stderr: the run, the drawing and the PNG encoding.
  - Debug builds are much slower than release, so their timings are for comparison with each other, not for budgets.
- **Catching regressions:**
  - **Budgets in `cargo test`,** for numbers that don't depend on the machine. For example, a slow run of a fixed program renders the window at most once per animation frame, and re-uploads the canvas only when it changed. The animator tests already drive a real interpreter with made-up frame times, so they can count these exactly. A test that fails is a regression to fix, or a budget to change on purpose, in the same commit.
  - **Timings against a baseline:** a `perf` binary runs fixed programs in release (the interpreter, the renderer, the animator with no window, and Instant mode end to end). It compares them with `perf/baseline.json`, committed in the repo, and fails if anything is more than 20% slower. Spike 03's `bench` becomes part of it. Updating the baseline is a deliberate commit that says why.
  - There's no CI yet, so each phase's check, and every merge to main, runs `perf`. If CI is added, it runs the budget tests, and timings only on a fixed machine.
- **Then the first fix,** from what the numbers show. Suspects:
  - every frame re-renders the whole window, including the editor and the console, while only the canvas changes;
  - the canvas image may be uploaded to the GPU again every frame;
  - the lockstep round trip per line.
- **Goal:** a slow run costs little more than an idle window, and Instant mode is as fast as before. Spike 04's measurements are the starting baseline.

### Phase 4: Watching a program run (no new commands)

Done: see `.scratch/phase-4/` and ADR 0022. The interpreter sends the panels' contents in the event stream, just before the instruction that sees them, so they keep in step with the highlight however far ahead the interpreter is.

Two panels that update as the turtle goes, in step with the highlighted instruction. A value flashes when it changes.

- **Repeat counters:** the counter of every `REPEAT` and `FOREVER` running, outermost first, such as `REPEAT 3 of 36 › REPEAT 2 of 4`. The innermost one is what `REPCOUNT` outputs. Phase 2's `FOR` adds its own counter.
- **Variables:** the current value of every variable the running line can see: the globals, and the inputs and locals of each procedure in the call chain, since scope is dynamic (see "Kept from UCBLogo: dynamic scope"). Names keep the spelling of their definition (ADR 0004).
- **Keeping in step:** at animated speeds the interpreter waits at every line (lockstep), so it can publish its counters and changed variables just before it waits, and the panels show exactly what the highlighted line sees. Shallow binding (ADR 0007) makes the changes cheap to find. At Instant the panels update once a frame, and may be ahead of the drawing, as when pausing from Instant.
- **Layout:** two panels in the console area, beside the console's output, always visible. Both scroll and can be resized, for long lists of variables or values.

### Phase 5: Help, examples and tutorial (no new commands)

Done: see `.scratch/phase-5/` and ADR 0023. The reference lives in `pendown-logo`, so the interpreter's errors and the GUI's panel share it. Where the result differs from the plan below:
- An example's picture stays on the canvas when it ends, until **Back to my drawing** or Run, rather than going at once.
- The pictures are drawn when a page shows them, the same way the examples test draws them, rather than embedded in the app.
- Errors offer help with a button in the console's title bar, since the console's lines can't hold links.
- The tutorial's lesson picker is a bar of the lessons, and the search results don't take ↑ and ↓.
- The grammar is in `tree-sitter-logo/`, ready for a repository of its own, and leaves UCBLogo's optional inputs out.

Two of the headline features. They come after phase 2 so they cover the complete core language, and so the commands they document have settled down.

The UI design is on a canvas of mockups: https://claude.ai/artifact/Pcqcncf1iK6d3gnTQpyohp (private to the owner unless shared).

- **The side panel:** on the right of the window, beside the canvas, the height of the editor and canvas, with tabs for Help and Tutorial. It's resizable like the other panes, and the console strip below is unchanged. A **Help** button in the toolbar shows and hides it. **F1** opens it on Help for the word under the editor's cursor. The plan first put it next to the editor, but there it would separate the editor from the canvas, which show the code and what it draws.
- **Help panel:** a reference panel generated from `pendown-logo.kdl`.
  - The GUI embeds the file at build time (`include_str!`) and parses it with the `kdl` crate (version 6, as used by the validator).
  - **The top of the panel:** Back and Forward through the pages visited, a Contents button, and a search box.
  - **Following the cursor:** while the search box is empty, the panel shows help for the command under the editor's cursor, aliases included, so `FD` shows FORWARD. A strip says what it's following ("Following the cursor: FD, line 3"), with **Pin** to keep the page open while you type elsewhere.
  - **A command's page:**
    - where it sits, as links (Graphics › Turtle motion), its name and aliases, and the usage from `eg`;
    - our description;
    - the examples, each with its picture;
    - "See also": the other commands in the same section.
  - **Contents:** the manual's chapters and sections, in the kdl's order, with how many commands each holds. Chapters and sections with no implemented commands are left out, such as Objects. Commands that are our extensions (`extension #true`), such as `ARC`, get a small "Pendown" tag.
  - **Search:**
    - It matches names and aliases by prefix, with the matched part highlighted.
    - Each result shows the first sentence of its description and its section.
    - ↑ and ↓ choose a result, and Enter opens it.
  - **What it lists:** only the commands the interpreter implements. A **Show obsolete commands** checkbox at the bottom adds the obsolete ones, greyed out in their own group, each with its reason.
  - **Syntax:** the first entry in the contents, for what isn't a command in the manual:
    - `TO … END`, `:name`, `"word`, `[...]` lists, `{...}` arrays, `(...)`, `;` comments and the infix operators;
    - each gets a page, so following the cursor works on them too.
    - They need new entries in the kdl, say a `syntax` chapter outside `logo`, since the manual describes them in its Introduction (chapter 1), which `pendown-logo.kdl` leaves out.
- **Our own descriptions:** each command's `desc` is the manual's text, which describes UCBLogo: it mentions wxWidgets, percentage RGB values, `SETSCRUNCH` and so on. Add a `help` node to each supported command with our own description, written for our dialect, with the `differs` notes folded in. `desc` stays as the manual's reference for whoever implements the command. Writing them is real work: about 165 commands in phases 1 and 2.
- **Markup:** the `help` text and the tutorial use a small subset of markdown:
  - paragraphs, `code` spans, bold, italics, lists and fenced code blocks;
  - `[FORWARD]`, which is a link to that command's page.
  - A test checks that every link names a command or syntax entry.
- **Errors for obsolete commands:** use `obsolete_reason` in the error when someone runs an obsolete command, for example "EDIT isn't in Pendown: it opens an external text editor, and Pendown has its own", not just "I don't know how to EDIT". This is cheap and could be done earlier.
  - **Reword the reasons:** they're written as notes to us, such as "Opens an external text editor. The GUI has its own editor." Rewrite each so that it reads well after "X isn't in Pendown: ", and say "Pendown", not "the GUI". There are about 100.
- **Help links in errors:** an error in the console that names a command, including the ones for obsolete commands, ends with a "Help on FORWARD" link. The link opens the panel on that command's page, through its alias if the error used one. It needs the command an error is about to travel with the error, next to its span (ADR 0005).
- **Examples:** add one or more `example` nodes to each supported command. Each holds a short runnable program, with an optional title. This is different from `eg`, which only shows how to call the command. In the help panel, each example has **Run** and **Copy to editor** buttons.
  - **Pictures:** each example shows a small picture of what it draws. They are the PNGs the example tests already compare against, made smaller and embedded in the GUI.
  - **Run:** the example runs on the main canvas, at the current speed, with the usual animation, Step and counters.
    - The highlight is on the example's own code, in the help panel, and its Run button becomes Stop.
    - The status says "Running the example …", and a banner on the canvas says where the drawing came from.
    - The editor's text is never touched.
    - When the example ends, or **Back to my drawing** is pressed, the user's own drawing comes back. The window keeps the user's display list while the example runs.
  - **Copy to editor:** adds the example to the end of the editor's text. Undo takes it back out.
- **Examples are tests:** a test runs every example through the interpreter and the CLI renderer and compares the result with a saved PNG. That keeps the documentation and the interpreter in agreement. Examples that use `RANDOM` need a fixed seed, so their output is repeatable.
- **Tutorial:** a sequence of lessons, each with text and code, featuring Kame the turtle (ADR 0019).
  - The lessons follow the phase 1 commands, six in all:
    1. moving and turning;
    2. `REPEAT`;
    3. procedures ("Teaching Kame new words");
    4. inputs and variables;
    5. recursion;
    6. colours and shapes.
  - **Its files:** a folder of markdown files, one for each lesson, such as `tutorial/02-repeat.md`, embedded at build time. Code blocks are fenced as `logo`. Markdown is easier to write and review for long text than strings in a `tutorial.kdl`. The examples test also runs every code block in the tutorial.
  - **The tab:**
    - a lesson picker and a progress bar;
    - the lesson's text;
    - "Try this" boxes;
    - Previous and Next buttons, named after the lessons they go to.
  - **Code blocks:** each has **Load into editor** and **Run**. Run works like an example's. Load into editor replaces the editor's text, and asks first if there are unsaved changes, as New does.
- **Icons on the toolbar buttons:** New, Open, Save, Export PNG, Run, Pause/Resume, Step, Step instruction, Stop and the new Help button get an icon next to their label. The labels stay, since they help beginners.
  - The icons are Lucide's, which gpui-kit ships (`gpui_kit::assets::IconName`):
    - Run uses `play`, Pause uses `pause`, and Resume uses `play`;
    - Step uses `step-forward`, and Step instruction uses `footprints`;
    - Stop uses `square`, and Help uses `book-open`;
    - New uses `file-plus`, Open uses `folder-open`, Save uses `save`, and Export PNG uses `image-down`.
  - The ones that aren't in gpui-kit's default set are added through the app's own `AssetSource`, with `icon_assets!`. That costs a few KiB, not the whole catalogue.
  - Done, ahead of the rest of the phase, except for the Help button, which comes with the panel.
- **A tree-sitter grammar for Logo,** published on its own, with a Zed extension that uses it. The editor's colours come from `pendown-logo`'s own tokens (ADR 0008), so there's no grammar yet. This comes after phase 2, which adds the last new syntax, `{...}` arrays.
  - **Its own repository,** say `tree-sitter-logo`, since Zed extensions fetch grammars by git URL and commit. It has `grammar.js`, the generated parser, `queries/highlights.scm`, and tests.
  - **What it can parse:** tree-sitter can't know a procedure's arity, so it can't group an instruction with its inputs as our parser does (ADR 0001). It parses tokens and structure: words, numbers, `"words`, `:variables`, operators, `[...]` lists, `(...)`, `{...}` arrays, comments, and `TO … END` definitions, both one-line and multi-line. That's enough for colours, brackets, outlines and folding.
  - **Built-in commands:** `highlights.scm` lists them, generated from `pendown-logo.kdl`, with a test that it's up to date, like the `kdl` test that checks the command table.
  - **The Zed extension:** `extension.toml`, `languages/logo/config.toml` (the `.logo` suffix, `;` comments, brackets) and the queries.
  - **Open question:** should Pendown's own editor switch to the grammar, through gpui-kit's tree-sitter support? Our highlighter knows which procedures the program defines, which a grammar can't, so it probably stays.

Decided (these were open questions):

- **Markup:** a small subset of markdown, with `[NAME]` links between commands.
- **Tutorial format:** a folder of markdown files, not a `tutorial.kdl`.
- **Where examples run:** on the main canvas, with the user's drawing put back afterwards. A second canvas in the panel would need its own animation and controls, while the main canvas already has them.
- **Pictures:** yes, made from the PNGs the example tests already compare against.

### Phase 6: Interactive programs (14 commands)

- **Mouse:** `MOUSEPOS`, `CLICKPOS`, `BUTTONP`/`BUTTON`, and the `BUTTONACT`/`KEYACT` handlers.
- **Keyboard and console:** `READLIST`, `READWORD`, `READRAWLINE`, `READCHAR`, `READCHARS`, `KEYP`, `LINEP`, `CLEARTEXT`.

These need the parts of the speed design that synchronise with the GUI: sync points before input, shared mouse state in turtle coordinates, and a console mode for when the program is waiting for input. It's worth building them as a separate step.

### Phase 7: Advanced and niche (85 commands)

- **Pixel drawing and text:** `FILL`, `FILLED`, `LABEL`/`SETLABELHEIGHT`/`LABELSIZE`, the pen modes `PENPAINT`/`PENERASE`/`PENREVERSE`/`PENMODE`, and `SETPEN`/`PEN`, which include the mode. These add the `Fill`, `Filled` and `Label` events and the pen mode in `Move`. Two spikes, drafted and then put off, come first: `.scratch/pixel-drawing-spikes/`.
- **Language features:** objects (`KINDOF`, `ASK`, `TALKTO`, `USUAL`…), macros and backquote, property lists.
- **I/O:** file access, `SHELL`.
- **Low-level:** bitwise operations.
- **Introspection and metaprogramming:** `DEFINE`, `TEXT`, `COPYDEF`, `ARITY`, `PROCEDUREP` and similar.
- **Obscure library procedures:** `CASCADE`, `TRANSFER`, `CROSSMAP`.
- **Tokenizer details:** `PARSE`, `RUNPARSE`, `VBARREDP`, `RAWASCII`.
- **Settings:** `ALLOWGETSET`, `REDEFP`, `COMMANDLINE`.

## Dialect and extensions

We implement **UCBLogo**: it's the closest thing Logo has to a standard, it's the best documented, and most later Logos (FMSLogo, jsLogo) follow it. Where the dialects differ, it's mostly in commands we've marked obsolete or put in phase 7.

"Our Logo" is UCBLogo's core plus a few deliberate changes and extensions, listed here. They're recorded in `pendown-logo.kdl` too, so the help panel covers them:

- **New commands** have `extension #true` and a `source` naming where the idea came from. They're in a `shapes` section and in the pen sections of `graphics`.
- **Changed commands** keep the manual's `desc` and add a `differs` node describing how ours behaves.

References in `refs/`: `ucblogo-code` (the C source, for checking exact behaviour), `jslogo` (a UCBLogo-compatible interpreter with a canvas, the closest prior art to this playground) and `logo-rust`.

### Kept from UCBLogo: dynamic scope

A procedure can see the local variables of whatever procedure called it, not just globals. Every Logo works this way, and Logo teaching material relies on it.

- **Implementation:** a stack of frames. Looking up a variable walks from the current frame down through its callers to the globals. `MAKE` sets the nearest existing variable with that name, or creates a global if there isn't one.
- **GUI:** phase 4's variable panel should show the whole chain of frames, since that's what a running line can actually see.

### Colours

Everywhere a colour is accepted (`SETPENCOLOR`, `SETBACKGROUND`, `SETFILLCOLOR`, `SETPALETTE`, `FILLED`), all of these forms work:

- **Palette number:** 0 to 15, with UCBLogo's default colours: 0 black, 1 blue, 2 green, 3 cyan, 4 red, 5 magenta, 6 yellow, 7 white, 8 brown, 9 tan, 10 forest, 11 aqua, 12 salmon, 13 purple, 14 orange, 15 grey. `SETPALETTE` (phase 2) can redefine numbers from 8 up, as in UCBLogo.
- **RGB list:** `[r g b]` with each value **from 0 to 255**. This is a deliberate change: UCBLogo uses percentages from 0 to 99, but 0 to 255 is what everyone knows from CSS and image editors. FMSLogo also uses 0 to 255.
- **RGBA list:** `[r g b a]`, where `a` is the opacity, also from 0 to 255 (0 is invisible, 255 is solid). The pixel buffer blends it. Each move is drawn separately, so a translucent line is darker where two moves meet.
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

- **Outline:** drawn with the current pen (colour, size and, in phase 7, mode) when the pen is down.
- **Fill (phase 2):** `SETFILLCOLOR colour` sets a fill colour, and `SETFILLCOLOR "none` turns filling off. `"none` is the default. When a fill colour is set, shapes are filled, even when the pen is up. So pen up with a fill colour gives a filled shape with no outline.
- **Events:** `Ellipse { center, radii, heading, pen, fill }`, which `CIRCLE` uses too, with equal radii, and `Rect { center, size, corner, heading, pen, fill }` for `SQUARE` and `RECTANGLE`. At animated speeds the animator can draw the outline sweeping round, then the fill.

Filling turtle-drawn polygons is already covered by UCBLogo's `FILLED colour [instructions]` and `FILL` (phase 7).

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
- Rasterize the drawing into a pixel buffer with `vello_cpu`, and show it with `paint_image`. (Spike 02 picked vello_cpu over `tiny-skia`, which is about 4 to 14 times slower: see ADR 0010.)

A pixel buffer would also make `FILL` (flood fill) and saving to PNG straightforward.

The manual makes the pixel buffer close to essential rather than optional (see "Checked against the Berkeley Logo manual" below):

- **`FILL`** is a flood fill bounded by lines drawn earlier, so it needs to read pixels.
- **`PENREVERSE`** inverts each point along the turtle's path. gpui has no XOR or invert blend mode, so this can't be done with vector paths.
- **`PENERASE`** could be faked by drawing in the background colour, but that breaks if the background is changed later.

Plan on a pixel buffer for the finished drawing from the start, with vector paths only for the move in progress and the turtle sprite.

### 2. Keep the `pendown-logo` crate free of GUI code

Have the interpreter output a stream of turtle events in its own coordinate system. This must include everything the user should see happen, not only lines that leave ink:

- `Move { from, to, pen }`, which also covers pen-up moves, because those animate too. `pen` carries up/down, the mode (paint, erase or reverse), colour as resolved RGBA, and size.
- `Turn { from, by }`, where `by` is the signed angle turned. Logo measures headings clockwise from north, and `RT 270` should visibly turn 270° clockwise, not take the short way round. So `from`/`to` alone isn't enough.
- `Arc { center, radius, start_heading, angle, pen }`. Our `ARC` moves the turtle along the arc (see "Dialect and extensions"), so the animator moves the turtle sprite with it.
- `Ellipse { center, radii, heading, pen, fill }` for `CIRCLE` and `ELLIPSE`, and `Rect { center, size, corner, heading, pen, fill }` for `SQUARE` and `RECTANGLE`. The turtle doesn't move.
- `Label { at, text }`, drawn into the pixel buffer with vello_cpu, so the CLI draws it too. A spike works out how (`.scratch/pixel-drawing-spikes/issues/02-labels.md`).
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

The interpreter never knows about speed or time. It runs as fast as it can and sends the events above to the GUI. The GUI has a component we'll call the **animator**, which plays those events onto the canvas at the chosen speed. Speed is therefore purely a GUI concern, and the `pendown-logo` crate and CLI stay simple.

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
- **Events go in batches.** One channel send per event manages only 2–11 M events a second, more than the rest of the interpreter costs (spike 03). Batching makes runs 2.5–5 times faster: 9–12 M instructions a second for drawing loops, about ten times what the pixel buffer can draw.
  - The interpreter sends a `Vec<Event>` of up to 256 events when it's full, when it has been open for 4 ms, and at the end of the run. The channel holds 4 batches.
  - At animated speeds the interpreter waits at each line anyway (see "How highlighting and stepping work"), so the channel's size doesn't decide how far ahead it gets. Spike 04 tried channels from 4 to 32 batches; once Instant waits out its slice, larger ones gained only about 0.02 s on a 131,000-move drawing.
  - Every sync point sends the open batch first, including each line while stepping, so the animator can reach the `Sync` it's waiting for.
- **Turtle queries never wait for the GUI.** `POS`, `HEADING`, `PENDOWNP` and similar are answered from the interpreter's own turtle state. Only the drawing lags behind.
- **Sync points.** Before anything that depends on what the user sees, the interpreter sends a `Sync` event with a one-shot reply channel and waits until the animator reaches it. That applies to reading input, reading the mouse or keyboard, and the end of the program.
- **Driving frames.** While events are queued or a move is part-way through, the animator calls `window.request_animation_frame()` (`window.rs:2730`) each frame, and stops calling it when idle. Each frame it uses `try_recv` to take as many events as the current speed and elapsed time allow, so no async plumbing is needed. In two cases it waits briefly on the channel (`recv_timeout`) instead: in Instant mode, until its slice is used up, and for up to 4 ms after replying to a `Sync`, when the next line's events are microseconds away. gpui's docs say *decorative* animations should respect `App::reduce_motion`. This animation isn't decorative, because the user chose the speed, so it shouldn't be skipped.

### Speed model

- **Speed levels from 1 to 11.** Level 11 is Instant. Below it, each level doubles the speed of the one before, from 5 turtle steps a second at level 1 to 2,560 at level 10. Levels are easy to say ("try speed 3") and to write in a program (`SETSPEED 3`). The GUI shows the steps per second next to the slider.
- **Instant:** each frame, process events for a fixed time slice (8 ms) and paint once. This gives the same "everything appears at once" feel as other Logos, while the UI stays responsive on huge or endless programs.
  - The animator must wait for events until the slice is used up, not just take what's in the channel. Otherwise the channel's size caps Instant at a channelful a frame: a 131,000-move tree took 8.9 s that way, and 0.2 s with waiting (spike 04).
  - Drawing is capped at about 6 ms of segments a frame, estimated from the measured cost per segment.
- **Animated speeds:** turtle steps per second, and turning at 3 degrees per step, so at 100 steps a second `FD 50` takes 0.5 s and `RT 90` takes 0.3 s. The turtle glides along each `FD` and visibly rotates on each `RT`/`LT`, rather than jumping from one command to the next. Small moves cost little, so recursive drawings speed up naturally when they get into fine detail.
- **Instructions that don't move the turtle,** such as `REPEAT`, `MAKE` or a procedure call, keep the highlight for the time it takes to move 10 steps, at most 0.25 s. Otherwise they'd flash past too fast to see at any speed.
- **Changes apply immediately**, even mid-move, because only the animator reads the speed.
- **`WAIT`** is a `Wait` event that the animator honours at every speed, including Instant, because a program that says `WAIT` means it.
- **`SETSPEED n` / `SPEED`** would be a Logo-level extension that sends a `SetSpeed` event, so the speed changes at exactly that point in the program. `n` is a level from 1 to 11. Python's `turtle.speed()` is a well-known example of the same idea.

### Controls

- **Speed slider**: levels from 1 to 11. It changes only the animator's speed, and applies on the next frame, even in the middle of a move.
- **Pause / Resume**: the animator stops taking events. At animated speeds the interpreter is already waiting at the start of the next line; at Instant, it carries on until the channel is full.
- **Step**: while paused, run exactly one line of code, the same way UCBLogo's `STEP` command does. All the drawing and output that line produces plays at the current speed, with each instruction highlighted as it runs, then the program pauses again at the start of the next line.
- **Step instruction**: while paused, run to the next instruction, including inside brackets. One-line loops need it: a one-line `FOREVER [FD 1 RT 1]` never reaches another line, so Step alone could never stop in it.
- **Stop**: set a cancel flag that the interpreter checks on every evaluation step, and drop the channel so a blocked interpreter wakes up and exits. This should behave like UCBLogo's `THROW "TOPLEVEL`, which ends all running procedures, but the program's own `CATCH` must not be able to intercept it.

#### How highlighting and stepping work

Before running each instruction, the interpreter sends an `Instruction { span, starts_line }` event. `span` covers just that instruction, so in `REPEAT 4 [FD 50 RT 90]` the highlight moves from `REPEAT` to `FD 50` to `RT 90` and round again. `starts_line` is set on the first instruction run from each line.

- **Running normally:** `Instruction` is an ordinary event. The animator only uses it to move the highlight in the editor.
- **Lockstep:** the GUI sets a shared `lockstep` flag, an atomic the interpreter reads. While it's set, the interpreter sends a `Sync` after each `Instruction` with `starts_line`, and waits for the animator to reply before running that line.
  - It's set while paused or stepping, and at every animated speed. Only Instant, running, leaves it off.
  - Lockstep costs a round trip of about 10 µs per line, which allows about 25,000 lines a second at animated speeds (spike 04), far faster than anyone can watch.
- **Pausing:** the animator holds the next `Sync` it reaches instead of replying, so the interpreter waits before the highlighted line. Step releases the held `Sync` and plays until it holds the next one. Resume releases it and carries on.
- **Pausing from Instant** leaves the interpreter ahead, a few hundred lines at most, because it wasn't in lockstep. The first Step plays those lines, which have already run, and after that stepping is exact. That's accepted: lockstep at Instant would make programs with many short lines several times slower.

This matters because the channel lets the interpreter run several events ahead of the drawing. If `Instruction` were only a marker, a paused program could already have run a few lines past the highlighted one, including effects that aren't events, such as `MAKE` or file writes. Waiting at each line keeps the highlight and the program's real position in step, which a later variable inspector would also rely on.

Stepping therefore needs the source spans described below. They're required, not optional.

The Pause button is not the same as Logo's `PAUSE` command. Logo's `PAUSE` stops the program inside a procedure and opens a prompt where its local variables can be inspected, and UCBLogo also opens one of these when the user presses its quit key. Logo's `PAUSE` and `CONTINUE` are marked obsolete. A later version could offer "Pause and inspect" in the GUI instead, but the first version's Pause button only stops the animator.

### Highlighting the running code

Highlighting and stepping depend on source spans, and every node in the parsed program needs one from the start. With them, the editor highlights the instruction the turtle is carrying out, using the `Instruction` events. At slow speeds that's the direct link between the code and the drawing.

In the editor (spike 01), the highlight is a gpui-kit `RangeDecoration` fill over the instruction's span, and the editor scrolls to keep it in view by setting its scroll offset when the highlight nears an edge. Syntax colours are separate: our own `InputHighlighter`, driven by the Logo lexer, so they follow the theme. Re-lexing the whole buffer on every keystroke is fast enough.

For procedures, the span must point into the text where the procedure was defined. A program might call a procedure that was defined in the editor, typed at the console, or loaded from a file, so a span needs to record which source it came from as well as the position.

### How this fits with rendering

This combines well with "No saved drawing between frames" above:

- Completed moves get added to the saved drawing, either as batched paths or as a pixel buffer.
- Only the move in progress and the turtle sprite are drawn fresh each frame.

That way frame cost depends on what's changing, not on how much has been drawn so far.

### CLI

The CLI reads the same event stream with no timing and renders it to a file. `WAIT` and `SETSPEED` do nothing there.

```
pendown-cli <program.logo> [--output <file.png>] [--size <width>x<height>] [--limit <instructions>] [--seed <n>]
```

- **`--output`** (`-o`) defaults to the program's name with `.png`.
- **`--size`** (`-s`) defaults to `1000x1000`. The turtle area is fitted to it without stretching.
- **`--seed`** fixes `RANDOM`'s seed, so the picture is the same every time. Without it, each run is different.
- **`--limit`** defaults to 10 million instructions. A program that never ends, such as `FOREVER`, stops there with an error, and the picture is saved as far as it got.
- **Output:** what the program prints goes to stdout. Errors go to stderr as `program.logo:LINE:COLUMN: message`, with the chain of calls after it.
- **Exit codes:**
  - 1 for a parse error, which saves nothing;
  - 1 for a runtime error, which saves the picture as far as it got;
  - 2 for a bad option.
- **Later:** SVG export, drawn from the event stream rather than the pixel buffer.

### Open questions

- ~~**Speed scale and units:**~~ Levels from 1 to 11, where 11 is Instant and each level below doubles the speed, from 5 to 2,560 steps a second (spike 04; see "Speed model").
- ~~What does the highlight show?~~ The running instruction, not the whole line.
- ~~**What does Step step over:**~~ Step runs one source line, so `REPEAT 4 [FD 50 RT 90]` is one step. A second button, Step instruction, stops inside the brackets, and one-line loops need it (spike 04; see "Controls").
- ~~Does `STEP` itself work?~~ No: `STEP`, `TRACE` and their relatives are marked obsolete in `pendown-logo.kdl`. Stepping and tracing are debugger controls in the GUI.
- ~~**Channel size:**~~ Batches of up to 256 events, at most 4 ms old, with room for 4 batches (spike 03, confirmed by spike 04; see "Architecture").

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
- **Obsolete, not supported** (see `pendown-logo.kdl`): `REFRESH` / `NOREFRESH` (we always keep the picture), `SAVEPICT` / `LOADPICT` / `EPSPICT` (replaced by GUI export), and `TEXTSCREEN` / `SPLITSCREEN` / `FULLSCREEN` (the GUI layout is fixed). Note for export: `EPSPICT` ignored `FILL`, `PENERASE` and `PENREVERSE`, and an SVG export from the event stream would have the same limits. PNG export from the pixel buffer would not.
