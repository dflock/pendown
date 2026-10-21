# Pendown

**A Logo playground for learning to code, where you can watch the turtle think.**

Most Logos draw everything instantly. Pendown lets you slow the turtle right down and watch it draw, with the instruction it's carrying out highlighted in your code as it goes. Pause it, step through a line at a time, speed it up again, or stop it. You can always see which bit of code made which bit of the picture.

Pendown speaks [UCBLogo](https://people.eecs.berkeley.edu/~bh/logo.html), the most widely used Logo, with a few modern touches:

- **Colours your way:** palette numbers, `[r g b]` lists from 0 to 255, CSS names like `"cornflowerblue`, and hex like `"#ff8800`, with transparency.
- **Shapes:** `CIRCLE`, `ELLIPSE`, `SQUARE` and `RECTANGLE`, outlined and filled.
- **An `ARC` that does what you'd expect:** the turtle walks along it.
- **The whole core language:** words, lists and arrays, `FOR` and `WHILE`, `MAP`, `FILTER` and `FOREACH` with templates, and `CATCH` and `THROW`.
- **Help built in:** every command is documented in the app, with examples you can run.
- **A tutorial,** where Kame the turtle teaches you Logo one step at a time.

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

`PENDOWN_AUTOTEST=1 cargo run --release -p pendown` runs a scripted session in the app: running, stepping, stopping, the console, files, errors and filled shapes. It writes a log and screenshots to `target/autotest/`, and fails if any of its checks do.

Type a program in the editor and press **Run**. Use the speed slider to choose how fast the turtle draws. Try the console too: after a run, it knows every procedure your program defined, so you can type `FLOWER 80` and watch.

### The CLI

`pendown-cli` runs a program without a window and saves the drawing:

```sh
cargo run --release -p pendown-cli -- flower.logo --output flower.png --size 800x600
```

## What's in the box

| Crate | What it is |
|---|---|
| `pendown` | The app: editor, canvas, console, help and tutorial |
| `pendown-cli` | Runs programs from the command line and renders them to images |
| `pendown-logo` | The Logo interpreter, as a library with no GUI code |
| `pendown-render` | Draws the turtle's moves into a picture, for the app and the CLI |

The crates are in `crates/`. `spikes/` holds the experiments made before phase 1, kept as a record; each builds on its own.

## Learn more

- `examples/phase-1/` and `examples/phase-2/` have programs that between them use every command Pendown supports so far.
- `pendown-logo.kdl` lists every Logo command, which ones Pendown supports, and how ours differ from UCBLogo.
- `docs/adr/` records the design decisions and why we made them.
