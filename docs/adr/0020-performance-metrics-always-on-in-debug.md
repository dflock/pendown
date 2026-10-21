# 20. Performance metrics: always on in debug builds, with budgets and a baseline

Status: Accepted
Date: 2026-10-21

## Context

After phase 1, running a program in the GUI kept one CPU core fully busy, even at slow speeds where the turtle barely moves. Nothing showed where the time went. A one-off fix would leave the next regression just as hard to see, so performance needs to be measured all the time, and regressions need to be obvious and get fixed.

## Decision

- **Each crate that does work has a `metrics` module:** counters and timings kept as atomics, cheap enough to leave in the code.
- **On by default in debug builds, off by default in release builds.** `PENDOWN_METRICS=1` turns it on in a release build and `PENDOWN_METRICS=0` off in a debug one. Off, a measurement costs one check of a flag.
- **What's measured:**
  - **The UI thread's frames:** rendering, the animator, drawing into the pixel buffer, and uploading the canvas.
  - **The interpreter:** its instruction rate, and its time waiting in lockstep or on a full channel.
  - **CPU time for each thread.**
- **Where it's shown:**
  - A one-line summary on stderr at the end of each run, in the GUI and the CLI.
  - A live overlay in the GUI, toggled by a key.
- **Regressions are caught two ways:**
  - **Budgets in `cargo test`** for numbers that don't depend on the machine, such as renders a frame or canvas uploads a run.
  - **Release timings against `perf/baseline.json`,** committed in the repo, by a `perf` binary that fails when something is more than 20% slower.
  
  Changing a budget or the baseline is a deliberate change in its own commit, with the reason.
- **Until there's CI,** each phase's check, and each merge to main, runs `perf`.

## Consequences

- Debug runs print a summary on stderr, which tests capture and the GUI's terminal shows.
- Timings from debug builds are only good for comparing with each other. Budgets that use time are measured in release.
- Reading per-thread CPU time is platform-specific. Linux comes first, from `/proc/self/task`.
- A baseline is tied to the machine it was measured on. A new machine needs a new baseline, committed with a note of the machine.
