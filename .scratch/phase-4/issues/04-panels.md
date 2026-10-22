# The panels

Status: done
Type: task
Blocked by: 03

## What

The console area is split, resizably, into the console, the counters and the variables. Each panel is a cached view that scrolls. A changed value or counter flashes and fades. At Instant the window asks for a `Watch` each frame.

## Done when

The panels show a running program's counters and variables, in step with the highlight, and the screenshots show them.

## Comments

### 2026-10-22: done

`watching.rs` has `Watched`, which works out what flashes, and the two panes, drawn cached in a resizable split beside the console. A change flashes yellow and fades over 0.7 s (gpui's `with_animation`, at most 30 fps). Run clears the panels, and Stop keeps only the globals, which are what the interpreter has left once it has unwound.

Two things learned on the way:
- **Notifying `Watched` while the window draws doesn't redraw the cached panes,** so they showed a value from seconds earlier. The window now notifies the panes themselves, and asks for one more frame so they redraw when paused.
- **gpui can drop an animation frame,** as when it throttles a window that isn't focused, and then nothing woke the run: the welcome program hung part-way once in three sessions. Frame pacing now sets a 100 ms timer as well as asking for an animation frame.
