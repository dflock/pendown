# The live overlay

Status: done
Type: task
Blocked by: 04

## What

F12 shows or hides a small overlay on the canvas with the live numbers: frame rate, UI time and CPU a frame, the interpreter's instruction rate and waits, the drawing's time and uploads. It works in any build: pressing F12 turns metrics on if they were off.

## Done when

The scripted session toggles it and takes a screenshot.

## Comments

### 2026-10-22: done

F12 (`ToggleMetrics`) shows the overlay at the canvas's top left and turns metrics on. Its numbers cover the last half second, refreshed by a timer so they update even with no frames. Two things learned on the way:
- the interpreter now publishes its instruction count on every instruction, a relaxed store, since every 4,096 left slow programs at 0;
- `Timer::total_now` includes a wait still in progress, so an interpreter blocked for seconds shows as waiting while it waits.

The scripted session toggles it and screenshots it (`metrics`).
