# The GUI app from spike 04

Status: ready-for-agent
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
