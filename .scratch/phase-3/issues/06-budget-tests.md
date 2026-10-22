# Budget tests

Status: done
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

## Comments

### 2026-10-22: done

- **`cargo test`** (`tests/animator.rs`, `mod budgets`): the animator, a real interpreter and a `Drawing`, played as the window plays them, waiting as long as `next_change` says.
  - A speed 3 square takes at most 450 frames, where every frame at 60 a second would be 1,575.
  - A 2 s `WAIT` takes about 20 frames.
  - A speed 10 square takes 10 to 40 frames.
  - No tiles are uploaded on frames that drew nothing.
  - Paused and finished runs want no frames and take nothing.
- **The scripted session** measures 3 s of a speed 3 run.
  - The window renders at most 25 times a second, and at most once an animation frame.
  - The editor renders only when the highlight moves, and the console only when something is printed, give or take the renders that start and end the measurement.

Breaking the pacing on purpose fails both: 1,575 frames in the test, and 60 renders a second in the session.
