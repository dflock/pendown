# The window's metrics and the end-of-run summary

Status: done
Type: task
Blocked by: 02, 03

## What

- **The window, each frame:** frames, renders of the root view, time in our render code, and the UI thread's CPU time.
- **A one-line summary on stderr when a run ends,** such as `run 3.2 s: 190 frames, UI 4.1 ms a frame, CPU UI 35% interpreter 2% other 0%`, with the interpreter's rate and waits, and the drawing's time.
- **The CLI** prints its own summary: running, drawing and PNG encoding.

## Done when

Running a program in a debug build prints the summary, and a release build doesn't unless `PENDOWN_METRICS=1`. The CLI's is tested.

## Comments

### 2026-10-22: done

`WindowMetrics` (renders, canvas paint, tile uploads), `Snapshot` and `Interval` in `pendown`'s `metrics.rs`. The window takes a snapshot when a job starts and prints `Interval::summary` on stderr when it finishes or is stopped. The CLI's `Output::summary` covers running, drawing and PNG encoding, and `main` prints it. Tests: the summary's format in `metrics.rs`, and `metrics_print_a_summary_on_stderr` in the CLI's tests, which otherwise run with `PENDOWN_METRICS=0`.

First numbers, debug build, 60 Hz: a slow run uses 4.6 ms of UI CPU a frame, of which our render code is 0.25 ms and the canvas 0.14 ms. The rest is gpui laying out and painting the whole window.
