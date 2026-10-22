# Budget tests

Status: ready-for-agent
Type: task
Blocked by: 03, 04

## What

Budgets in `cargo test` for numbers that don't depend on the machine:
- an animated run of a fixed program uploads tiles only on frames where something was drawn;
- a paused run, and a finished one, take no events and draw nothing;
- the scripted GUI session checks that the window renders at most once a frame, and that the editor and console re-render only when they change.

Changing a budget is a deliberate change in its own commit, with the reason.

## Done when

The tests pass, and fail when the behaviour they guard is broken on purpose.
