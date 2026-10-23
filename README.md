# Pendown

**A Logo playground for learning to code, where you can watch the turtle think.**

Most Logos draw everything instantly. Pendown lets you slow the turtle right down and watch it draw, with the instruction it's carrying out highlighted in your code as it goes. Pause it, step through a line at a time, speed it up again, or stop it. You can always see which bit of code made which bit of the picture. Beside the console, two panels show what the highlighted instruction sees: the counter of every loop running, and the value of every variable, in each procedure of the call chain. A value flashes when it changes.

Pendown speaks [UCBLogo](https://people.eecs.berkeley.edu/~bh/logo.html), the most widely used Logo, with a few modern touches:

- **Colours your way:** palette numbers, `[r g b]` lists from 0 to 255, CSS names like `"cornflowerblue`, and hex like `"#ff8800`, with transparency.
- **Shapes:** `CIRCLE`, `ELLIPSE`, `SQUARE` and `RECTANGLE`, outlined and filled.
- **An `ARC` that does what you'd expect:** the turtle walks along it.
- **The whole core language:** words, lists and arrays, `FOR` and `WHILE`, `MAP`, `FILTER` and `FOREACH` with templates, and `CATCH` and `THROW`.
- **Help built in:** every command is documented in the app, with examples you can run on the canvas. Press F1 for help on the word under the cursor.
- **A tutorial,** where Kame the turtle teaches you Logo one step at a time.
- **Errors that help:** an obsolete UCBLogo command says why it isn't here, and an error offers help on its command.

## A taste of Logo

```logo
TO PETAL :size
  REPEAT 2 [ARC 60 :size RT 120]
END

TO FLOWER :size
  REPEAT 12 [PETAL :size RT 30]
END

SETBACKGROUND "midnightblue
SETPENCOLOR "gold
FLOWER 150
SETFILLCOLOR "orange
CIRCLE 20
```

## Getting started

Pendown is written in Rust, so you'll need a [Rust toolchain](https://rustup.rs). On Linux, also install the windowing libraries. On Debian or Ubuntu:

```sh
sudo apt install libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libx11-xcb-dev
```

Then build and run it:

```sh
git clone <repo-url> pendown
cd pendown
cargo run --release -p pendown
```

Run the tests with `cargo test`. Debug builds work too, but drawing is much faster in release.

`PENDOWN_AUTOTEST=1 cargo run --release -p pendown` runs a scripted session in the app: running, stepping, stopping, the console, files, errors, filled shapes, how often the window redraws, the counters and variables panels, and the help panel and tutorial. `PENDOWN_AUTOTEST_FROM="Phase 5"` starts it at the first section with that in its title. It writes a log and screenshots to `target/autotest/`, and fails if any of its checks do.

Type a program in the editor and press **Run**. Use the speed slider to choose how fast the turtle draws. Try the console too: after a run, it knows every procedure your program defined, so you can type `FLOWER 80` and watch.

While a program runs, the **Repeat counters** panel shows every `REPEAT`, `FOREVER`, `FOR` and `FOREACH` running, outermost first, such as `REPEAT 3 of 12`, and marks the one `REPCOUNT` would give. The **Variables** panel shows the globals, then each procedure in the call chain with its inputs and locals; a variable hidden by a later one with the same name is greyed out. Below them are the pen, fill and background colours, as they were set, each with a square of the colour. At animated speeds, and when paused or stepping, they show exactly what the highlighted instruction sees. At Instant they keep up as best they can.

### Help and the tutorial

The **Help** button, or **F1**, opens a panel beside the canvas. Help follows the editor's cursor: put it on `FD`, or on `:size`, and the panel explains it. Each page has our description, examples with a picture of what they draw, and links to the rest. **Run** plays an example on the canvas, with its code highlighted in the panel, and **Back to my drawing** brings yours back. **Copy to editor** adds it to your program. Browse the contents by chapter, search by name, and tick **Show obsolete commands** to see the UCBLogo commands Pendown leaves out, and why.

The **Tutorial** tab has six lessons with Kame: moving and turning, `REPEAT`, procedures, inputs and variables, recursion, and colours and shapes. Each program in them loads into the editor or runs on the canvas.

The help comes from `pendown-logo.kdl`, and the lessons are markdown files in `tutorial/`. Every example and every lesson's program is a test: it must run, and draw the picture saved in `crates/pendown/tests/help-pictures/`.

### The CLI

`pendown-cli` runs a program without a window and saves the drawing:

```sh
cargo run --release -p pendown-cli -- flower.logo --output flower.png --size 800x600
```

`--fit` zooms in on the drawing, rather than showing the whole turtle area.

### Performance

Pendown measures itself all the time (ADR 0020).

- **Metrics are on in debug builds and off in release.** `PENDOWN_METRICS=1` turns them on, and `PENDOWN_METRICS=0` off.
- **When a run ends,** the app prints a summary on stderr, such as:

  ```
  run 5.0 s: 58 renders at 12/s (editor 7, console 2), UI 5.28 ms a frame (ours 0.17 ms, canvas 0.18 ms, animator 0.01 ms); ...; CPU UI 6% interpreter 0% other 1%
  ```

  It shows how often the window redrew and what each frame cost, what the interpreter did and how long it waited, and each thread's share of a CPU. The CLI prints its own.
- **F12 shows the same numbers live,** over the canvas, in any build.
- **`cargo test` checks budgets** that don't depend on the machine, such as how many frames a slow run takes.
- **`perf` times fixed programs** and compares them with `perf/baseline.json`. It fails if anything is more than 20% slower. Run it before merging to main:

  ```sh
  cargo run --release -p pendown-perf             # compare with the baseline
  cargo run --release -p pendown-perf -- --save   # a new baseline, in a commit of its own saying why
  ```

  A baseline belongs to one machine. On another, `perf` says so and doesn't fail.

## What's in the box

| Crate | What it is |
|---|---|
| `pendown` | The app: editor, canvas, console, help and tutorial |
| `pendown-cli` | Runs programs from the command line and renders them to images |
| `pendown-logo` | The Logo interpreter, as a library with no GUI code |
| `pendown-render` | Draws the turtle's moves into a picture, for the app and the CLI |
| `pendown-perf` | The `perf` binary: timings against a baseline |

The crates are in `crates/`. `spikes/` holds the experiments made before phase 1, kept as a record; each builds on its own.

## Learn more

- `examples/phase-1/` and `examples/phase-2/` have programs that between them use every command Pendown supports so far.
- `pendown-logo.kdl` lists every Logo command, which ones Pendown supports, and how ours differ from UCBLogo.
- `docs/adr/` records the design decisions and why we made them.
- `tree-sitter-logo/` is a tree-sitter grammar for Logo, with a Zed extension, ready to become a repository of its own.

## Credits

- Kame's picture, `tutorial/kame-turtle.svg`, is [Turtle](https://openclipart.org/detail/229886/turtle) from Openclipart, in the public domain (CC0).
- The app's font is [Noto Sans](https://notofonts.github.io/), by the Noto Project Authors, under the SIL Open Font License: see `crates/pendown/fonts/OFL.txt`.
