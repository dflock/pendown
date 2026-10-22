# The window's metrics and the end-of-run summary

Status: ready-for-agent
Type: task
Blocked by: 02, 03

## What

- **The window, each frame:** frames, renders of the root view, time in our render code, and the UI thread's CPU time.
- **A one-line summary on stderr when a run ends,** such as `run 3.2 s: 190 frames, UI 4.1 ms a frame, CPU UI 35% interpreter 2% other 0%`, with the interpreter's rate and waits, and the drawing's time.
- **The CLI** prints its own summary: running, drawing and PNG encoding.

## Done when

Running a program in a debug build prints the summary, and a release build doesn't unless `PENDOWN_METRICS=1`. The CLI's is tested.
