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

## Phase 3: what we built and learned

Added 2026-10-22.

- **Metrics are values, not globals.** Each part keeps its own in a struct of atomics: the interpreter's in `Control`, the drawing's in `Drawing`, the animator's in `Animator`, and the window's in `App`. Budget tests read the counts of one run while other tests run beside them. The shared parts, such as the flag, `Counter`, `Timer` and `ThreadCpu`, are in `pendown_logo::metrics`.
- **Per-thread CPU** comes from `/proc/self/task/*/schedstat`, in nanoseconds. vello_cpu's workers are rayon threads with no names, so they're counted as "other".
- **Waits are counted while they happen** (`Timer::total_now`), so an interpreter blocked for seconds shows as waiting. The interpreter publishes its instruction count on every instruction: a relaxed store, too cheap to measure.
- **What the numbers found.** The window, not the interpreter, used the CPU. At 60 Hz each frame cost the UI thread 3–5 ms, the same with nothing changing, and slow runs drew every frame.
  - Frames are drawn only when the picture will look different: `Animator::next_change`.
  - The editor, console and toolbar are cached views.
  - Highlighting no longer scrolls the editor past its end, which set off a third of a second of spring animation at 60 frames a second.
  - An empty frame still costs gpui about 2 ms of UI time here, so drawing fewer frames matters most.
- **`perf`** is `crates/pendown-perf`.
  - It takes the best of 7 runs, since cases vary by up to about 15% from run to run.
  - On a machine other than the baseline's, it reports without failing.
  - The scripted GUI session checks render rates, which need a window.

