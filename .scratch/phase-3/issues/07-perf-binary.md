# The `perf` binary and its baseline

Status: done
Type: task
Blocked by: 02, 03

## What

A `perf` binary, run in release, that times fixed programs:
- the interpreter alone, with no events, events, and batches through a channel (spike 03's `bench`);
- the renderer drawing a large fixed drawing;
- the animator with no window, at Instant and at an animated speed;
- Instant end to end: the interpreter, the animator and the drawing.

It compares the results with `perf/baseline.json` and fails if anything is more than 20% slower. `perf --save` writes a new baseline, recording the machine. `bench`'s stack and memory reports move into it too.

## Done when

`cargo run --release -p pendown-perf` passes against a committed baseline, and fails when a result is made slower on purpose.

## Comments

### 2026-10-22: done

`crates/pendown-perf`, binary `perf`. It runs 11 cases, the best of 7 runs each:
- six through the interpreter;
- the renderer drawing the 131,070-line tree, and a 3000 × 3000 PNG of it;
- the animator with no window, at Instant and in lockstep;
- the tree at Instant end to end, drawn and its tiles copied out as for upload.

It refuses to run in a debug build.
- **On the baseline's machine,** anything over 20% slower fails it.
- **On another machine,** it says so and doesn't fail.
- **`--save`** writes `perf/baseline.json` with the machine's CPU.
- **`--stack` and `--memory`** print spike 03's reports. `bench` is gone: its speed cases are timed cases now.

Run to run, cases agree within about 15% at worst (the renderer and lockstep). That's why it takes the best of 7, and why the PNG case encodes a large picture: at 1000 × 1000 it took 4 ms and varied by 40%. A 1 ms sleep added to each `draw_pending` showed as the renderer case 45% slower, and failed.
