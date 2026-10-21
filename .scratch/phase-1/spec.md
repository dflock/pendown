# Phase 1: turtle drawing

The smallest Pendown that is fun to draw with: the GUI playground with the turtle speed system, the CLI, and the 87 phase-1 commands plus our `SETSPEED` and `SPEED`. See "Phase 1" in `implementation-plan.md` for the scope, `docs/adr/` for the decisions, and `.scratch/phase-1-spikes/` for what the spikes measured.

## What "done" looks like

- `pendown`, the GUI, opens a window with:
  - the editor, the canvas and the console;
  - Run, Pause, Step, Step instruction and Stop;
  - a speed slider with levels 1 to 11.
  
  Every phase-1 command works in it, with the running instruction highlighted, and errors show on the code.
- `pendown-cli` renders a program to a PNG.
- `.logo` files open and save.
- Every phase-1 command has tests in `pendown-logo`.

## Crates

A Cargo workspace at the repo root (ADR 0019), with the crates in `crates/`:

| Crate | What it is | Starts from |
|---|---|---|
| `pendown-logo` | The interpreter. No GUI or drawing code (ADR 0009). | `spikes/03-interpreter-core` |
| `pendown-render` | The display list and the pixel buffer, drawn with vello_cpu (ADRs 0003 and 0010), and PNG encoding. No gpui, so the GUI and the CLI share it. | `spikes/04-end-to-end-animator/src/canvas.rs`, minus the gpui parts |
| `pendown` | The GUI app: window, editor, canvas, console, animator. | `spikes/04-end-to-end-animator` |
| `pendown-cli` | The CLI. | new |

`pendown-render` is a fourth crate that ADR 0019 doesn't list. ADR 0003 says the CLI uses the same renderer as the GUI. The renderer can't live in `pendown-logo`, which has no drawing code, or in `pendown`, which the CLI shouldn't depend on. Issue 01 records the new crate in ADR 0019.

The spikes stay in `spikes/` as a record, outside the workspace. Each keeps its own `[workspace]`, and they share `spikes/target`.

## The CLI

```
pendown-cli <program.logo> [--output <file.png>] [--size <width>x<height>]
```

- **`--output`** defaults to the program's name with `.png`.
- **`--size`**, as in `--size 250x300`, is in pixels and defaults to `1000x1000`. The turtle area is fitted to it without stretching, as in the GUI (ADR 0003).
- It runs the program with no timing (`WAIT` and `SETSPEED` do nothing) and writes the picture when the program ends.
- Errors go to stderr as `program.logo:3:5: I don't know how to FOO`, and the exit code is non-zero.
- A program that never ends (`FOREVER`) needs a way out: a `--limit` on instructions (default 10 million), after which it stops with an error. Issue 04 decides the details.
- **Later:** SVG export, from the event stream rather than the pixel buffer. It can't show `FILL` or the pen modes, which arrive in phase 2.

## How the work is split

Each issue delivers something that runs end to end and has tests. Issues 01 and 02 come first. Most of the rest only need 01 and 03, so they can run in parallel.

| Issue | What | Blocked by |
|---|---|---|
| 01 | Workspace, with `pendown-logo` and `pendown-render` from the spikes | |
| 02 | `SETSPEED` and `SPEED` in `pendown-logo.kdl` | |
| 03 | The GUI app from spike 04 | 01 |
| 04 | `pendown-cli`: PNG output | 01 |
| 05 | Turtle motion, queries and visibility | 01 |
| 06 | Turtle modes: `WRAP`, `WINDOW`, `FENCE` | 05 |
| 07 | Pen and colours | 05 |
| 08 | `ARC` and the shapes | 05, 07 |
| 09 | Procedures, variables and control | 01 |
| 10 | Arithmetic and logic | 01 |
| 11 | Lists, words and printing | 01 |
| 12 | `WAIT`, `SETSPEED` and `SPEED` | 02, 03 |
| 13 | The console | 03 |
| 14 | Files: New, Open, Save, and PNG export | 03 |
| 15 | The editor: syntax colours and errors while typing | 03 |
| 16 | Phase 1 check: examples, HiDPI and Wayland | all others |

For issues 05 to 11, each command is done when:
- it's implemented in `pendown-logo`, with tests for its behaviour and its errors;
- its events are drawn by `pendown-render` and animated by the GUI, where it has any;
- `pendown-logo.kdl` marks it `supported #true`, which is already the case for phase 1.

## Testing

- **`pendown-logo`:** unit and integration tests, as in spike 03. Every command gets a test of its output or events, and of its errors.
- **The animator:** tests with a real interpreter thread and no window, as in spike 04.
- **Pictures:** `pendown-cli` renders small test programs, and the tests compare the result with saved PNGs.
  - This starts in issue 04. Phase 3 extends it to every example.
  - The comparison allows small differences in anti-aliasing.
- **The GUI:** a scripted session like spike 04's (`PENDOWN_AUTOTEST=1`), for what can't be tested without a window.

## Out of scope

- Phase 2 and later commands, including `FILL`, `LABEL`, the pen modes and filled shapes.
- Help, examples and the tutorial (phase 3).
- SVG export.
- Input from the keyboard or mouse (phase 4).
