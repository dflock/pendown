# `pendown-cli`: PNG output

Status: ready-for-agent
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
