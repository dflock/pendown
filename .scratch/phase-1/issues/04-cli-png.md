# `pendown-cli`: PNG output

Status: done
Type: task
Blocked by: 01

## What

```
pendown-cli <program.logo> [--output <file.png>] [--size <width>x<height>]
```

- Run the program with `pendown-logo`, collect the events, and draw them with `pendown-render` at the given size. Then write a PNG.
- `--output` defaults to the program's name with `.png`, and `--size` to `1000x1000`.
- **No timing:** `WAIT` and `SETSPEED` do nothing, and `Instruction` events aren't asked for (`Limits::instruction_events = false`).
- **Errors** go to stderr as `program.logo:LINE:COLUMN: message`, with the call chain after it, and a non-zero exit code. A parse error writes no picture. A runtime error writes the picture as far as it got, so you can see where it stopped, and says so.
- **Endless programs:** decide how the CLI stops `FOREVER`. The suggestion in `spec.md` is `--limit <instructions>`, with a default of 10 million and an error when it's reached. Record the decision in `implementation-plan.md` ("CLI").
- Use a small argument parser (`clap` with `derive`, or `lexopt`), with `--help`.
- Update the "CLI" section of `implementation-plan.md` with the options, and note that SVG export comes later.

## Done when

- `pendown-cli` renders the spike examples to PNG.
- **Picture tests:** a few small programs (a square, a coloured spiral, a non-square `--size`) are rendered and compared with saved PNGs in `crates/pendown-cli/tests/`.
  - They allow small anti-aliasing differences.
  - There's a documented way to update the saved pictures, such as `UPDATE_PICTURES=1 cargo test`.
- Error output and exit codes have tests.

## Comments

### 2026-10-21: done

- **The command:** `pendown-cli <program.logo> [--output|-o <file.png>] [--size|-s WxH] [--limit N]`, using `clap`.
  - The library (`src/lib.rs`) runs the program on a 256 MiB thread and draws the moves into `pendown-render`.
  - It sends no `Instruction` events, and draws segments in batches of 4,096.
- **Endless programs:** `--limit`, with a default of 10 million instructions. It's a new `Limits::max_instructions` in `pendown-logo`, with a test, and the error is "Stopped after N instructions: does this program ever end?".
- **Output:**
  - Printed output goes to stdout. Errors go to stderr as `file:line:column: message`, with the call chain.
  - A parse error saves nothing. A runtime error saves the picture so far, and says so.
  - The exit code is 1 for program errors and 2 for bad options.
- **Tests:**
  - `tests/cli.rs` (7 tests): the default output name, the options, a bad size, parse and runtime errors, the limit, and a missing file.
  - `tests/pictures.rs` (4 pictures): a square, a palette-coloured spiral, a non-square size, and a tree.
  - The picture tests allow a channel difference of 40 on up to 0.2% of pixels. `UPDATE_PICTURES=1` updates the saved pictures, and a failing test saves its picture next to them as `NAME.actual.png`.
- **Docs:** `implementation-plan.md` ("CLI") now has the options, outputs and exit codes, and notes that SVG export comes later.
