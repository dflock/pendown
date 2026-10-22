# Phase 3: performance, with metrics that stay

While a program runs, Pendown keeps a CPU core busy, even at slow speeds where the turtle barely moves. Rather than a one-off fix, this phase builds in measurement that stays (ADR 0020), so the cause shows up now and regressions show up later. Then it makes the first fix. See "Phase 3" in `implementation-plan.md`.

There are no new Logo commands.

## What's known before starting

A first look, with `top`-style sampling of each thread's CPU time from `/proc`, on this machine (8 CPUs, Intel HD 530, X11, 60 Hz):
- The UI thread uses 20–30% of a core in both release and debug builds, and **the same with no program running**, as long as the window keeps asking for frames. The interpreter and vello_cpu's workers barely register.
- So the cost is re-rendering the whole window (editor, console and toolbar) every frame, about 4 ms of CPU a frame. On a 120 or 144 Hz display, or a larger hi-DPI window, that's most of a core.
- gpui re-renders a view when it's notified, along with its ancestors. `request_animation_frame` notifies the root view, which is the whole window, so everything is rebuilt each frame. Views drawn with `.cached()` are reused unless they, or an entity they read, were notified.

## What "done" looks like

- **Metrics, on in debug builds and off in release** unless `PENDOWN_METRICS` says otherwise. Off, a measurement costs one check of a flag.
- **What's measured:**
  - the interpreter: instructions, time running, and time waiting in lockstep or on a full channel;
  - the animator and the drawing: events taken, time in the animator, marks drawn, time drawing, and tiles uploaded;
  - the UI thread, each frame: frames, window renders, time in our render code, and CPU time;
  - CPU time for each thread, from `/proc/self/task` on Linux.
- **A one-line summary on stderr** at the end of each run, in the GUI and the CLI.
- **A live overlay in the GUI,** toggled by F12.
- **Budget tests in `cargo test`** for numbers that don't depend on the machine.
- **A `perf` binary** that times fixed programs in release and compares them with `perf/baseline.json`, failing when anything is more than 20% slower. Spike 03's `bench` becomes part of it.
- **The first fix:** a slow run costs little more than an idle window, and Instant is as fast as before.

## Decisions

- **Metrics are values, not globals.** Each part keeps its own counters in a struct of atomics: the interpreter's in its `Control`, which the GUI already shares, the drawing's in `Drawing`, the animator's in `Animator`, the window's in `App`. Tests run in parallel in one process, so budget tests need counts that belong to one run. ADR 0020's "a `metrics` module in each crate" is the module that defines those structs.
- **The shared parts** (the flag, counters, timers and per-thread CPU) are in `pendown_logo::metrics`, since every crate depends on `pendown-logo`.
- **Per-thread CPU** comes from `/proc/self/task/*/schedstat`, which counts in nanoseconds. vello_cpu's workers are rayon threads with no names, so they show as "other" alongside gpui's threads; the drawing's own timings say how much of that is vello.
- **The summary is printed when metrics are on,** whatever the build, to stderr.

## How the work is split

| Issue | What | Blocked by |
|---|---|---|
| 01 | `pendown_logo::metrics`: the flag, counters, timers, per-thread CPU | |
| 02 | Interpreter metrics | 01 |
| 03 | Drawing and animator metrics | 01 |
| 04 | The window's metrics and the end-of-run summary, in the GUI and the CLI | 02, 03 |
| 05 | The live overlay | 04 |
| 06 | Budget tests | 03, 04 |
| 07 | The `perf` binary and its baseline | 02, 03 |
| 08 | The first fix | 04 |
| 09 | Phase 3 check | all others |

## Testing

- **Unit tests** of the metrics module: the flag, counters and timers, and that per-thread CPU finds the test's own thread.
- **Budget tests** of the animator and drawing with no window, and checks in the scripted GUI session for what needs one.
- **`perf`** in release, against the baseline.

## Out of scope

- CI. Until there is some, each phase's check and each merge to main runs `perf` by hand (ADR 0020).
- Per-thread CPU on macOS and Windows.
