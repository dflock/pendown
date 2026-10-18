# Pendown

**A Logo playground for learning to code, where you can watch the turtle think.**

Most Logos draw everything instantly. Pendown lets you slow the turtle right down and watch it draw, with the instruction it's carrying out highlighted in your code as it goes. Pause it, step through a line at a time, speed it up again, or stop it. You can always see which bit of code made which bit of the picture.

Pendown speaks [UCBLogo](https://people.eecs.berkeley.edu/~bh/logo.html), the most widely used Logo, with a few modern touches:

- **Colours your way:** palette numbers, `[r g b]` lists from 0 to 255, CSS names like `"cornflowerblue`, and hex like `"#ff8800`, with transparency.
- **Shapes:** `CIRCLE`, `ELLIPSE`, `SQUARE` and `RECTANGLE`, outlined and filled.
- **An `ARC` that does what you'd expect:** the turtle walks along it.
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

Pendown is written in Rust. With a [Rust toolchain](https://rustup.rs) installed:

```sh
git clone <repo-url> pendown
cd pendown
cargo run --release -p pendown
```

Type a program in the editor and press **Run**. Use the speed slider to choose how fast the turtle draws. Try the console too: after a run, it knows every procedure your program defined, so you can type `FLOWER 80` and watch.

### The CLI

`pendown-cli` runs a program without a window and saves the drawing:

```sh
cargo run --release -p pendown-cli -- flower.logo -o flower.png
```

## What's in the box

| Crate | What it is |
|---|---|
| `pendown` | The app: editor, canvas, console, help and tutorial |
| `pendown-cli` | Runs programs from the command line and renders them to images |
| `pendown-logo` | The Logo interpreter, as a library with no GUI code |

## Learn more

- `pendown-logo.kdl` lists every Logo command, which ones Pendown supports, and how ours differ from UCBLogo.
- `docs/adr/` records the design decisions and why we made them.
