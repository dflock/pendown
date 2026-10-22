# The `perf` binary and its baseline

Status: ready-for-agent
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
