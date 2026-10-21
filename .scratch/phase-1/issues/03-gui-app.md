# The GUI app from spike 04

Status: done
Type: task
Blocked by: 01

## What

Build `crates/pendown` from `spikes/04-end-to-end-animator`, as the real app rather than a spike:

- **The window,** titled "Pendown":
  - the editor, the canvas and the console, in resizable panels;
  - Run, Pause/Resume, Step, Step instruction and Stop;
  - the speed slider, with levels 1 to 11, and the steps per second next to it.
- **The animator,** from the spike, with its tests (ADRs 0011 and 0012):
  - lockstep at animated speeds;
  - Instant waiting out its slice;
  - the dwell on instructions that don't move the turtle;
  - holding and releasing `Sync`s for Pause and Step.
- **The canvas:**
  - `pendown-render`'s pixel buffer, uploaded as dirty tiles;
  - the move in progress and the turtle sprite drawn as paths (ADR 0010);
  - the turtle area fitted without stretching, with the space outside it in a slightly different shade (ADR 0003).
- **The interpreter thread:** 256 MiB stack, batched channel of 4 × 256 events, `Control` for Stop and lockstep (spike 04's `run.rs`).
- **While running:**
  - the editor is read-only (ADR 0006);
  - the running instruction is highlighted, and the editor scrolls to follow it;
  - errors show as diagnostics and in the console, with their call chain, and repeated entries collapsed (ADR 0005).
- **Run** clears the drawing, the procedures and the variables (ADR 0006).
- `PENDOWN_AUTOTEST=1` runs a scripted session for what needs a window, as spike 04 did.

Use whatever commands `pendown-logo` has. Issues 05 to 11 add the rest, and each one checks its own events in the GUI.

## Done when

- Running the spike's examples (Square, Flower, Stepper, Forever, Tree) behaves as in spike 04.
- The animator tests pass in `crates/pendown`.
- The scripted session checks:
  - Stop from each state;
  - Step's exactness (no lines run ahead of the highlight);
  - a speed change mid-move;
  - the Tree at Instant with frames at 16.7 ms.

## Comments

### 2026-10-21: done

`crates/pendown`:
- **The library** (`src/lib.rs`), with no window:
  - `animator` and `run` from spike 04, now drawing `pendown-render`'s `Segment`s;
  - `speed`, the levels from 1 to 11;
  - `errors`, call chains with runs of the same call collapsed: "called from DOWN (line 2), 9,999 times".
  
  17 tests: spike 04's 12 animator tests, and 5 unit tests.
- **The window:**
  - `app.rs`: the toolbar (Run, Pause/Resume, Step, Step instruction, Stop, the speed slider and its "Speed 5 · 80 steps/s" label, and a status line), the editor, the canvas and the console.
  - The editor is read-only while running, the highlight is followed by scrolling, and errors show as diagnostics and in the console with their chains.
  - `canvas.rs`: `pendown-render`'s dirty tiles, the move in progress, the turtle, and the shading outside the turtle area (ADR 0003).
  - `highlighter.rs`: spike 01's lexer, until issue 15.
- **The scripted session:** `PENDOWN_AUTOTEST=1 cargo run --release -p pendown` (`autotest.rs`) runs 17 checks, and exits non-zero if any fail. All pass:
  - the highlight's order, and it being on the move while the turtle moves;
  - a speed change mid-move (50, then 200 steps/s);
  - no lines run ahead after each Step;
  - Stop from four states, within 0.001–0.07 ms;
  - the tree at Instant: 0.68 s, at 16.7 ms a frame;
  - errors in the console.
  
  The log and screenshots go to `target/autotest/`.
