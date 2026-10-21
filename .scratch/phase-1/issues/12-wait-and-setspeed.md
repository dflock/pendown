# `WAIT`, `SETSPEED` and `SPEED`

Status: done
Type: task
Blocked by: 02, 03

## What

- **`WAIT time`** waits `time` 60ths of a second.
  - It sends a `Wait` event, which the animator honours at every speed, including Instant (ADR 0011).
  - The highlight stays on the `WAIT` while it waits.
  - Pause, Step and Stop work during a `WAIT`.
  - `WAIT 0` does nothing visible.
- **`SETSPEED level`** sends a `SetSpeed` event.
  - The animator changes speed when it reaches the event, at exactly that point in the program, and the slider moves to match.
  - It's an error outside 1 to 11 or for a fractional level (issue 02).
- **`SPEED`** outputs the speed the program last set, or the slider's speed when the run started. The interpreter knows nothing about time, so the GUI passes the starting level in when it starts a run.
- **What a slider change while running means for `SPEED`:** the interpreter can't see it without asking the GUI. Decide between:
  - `SPEED` reports the last value set by the program or at the start;
  - the GUI shares the current level through `Control`.
  
  Record the decision in the kdl.
- **The CLI** ignores `WAIT` and `SETSPEED`. `SPEED` outputs 11 there.

## Done when

- `pendown-logo` has tests for the events, `SPEED`'s output, and the errors.
- Animator tests cover: a `WAIT` at Instant taking its time; Stop and Pause during a `WAIT`; and `SETSPEED` changing the speed mid-program.

## Comments

### 2026-10-21: done

- **`WAIT time`** sends `Event::Wait(duration)`, in 60ths of a second.
  - The animator counts it down in real frame time at every speed, including Instant, with the highlight on the `WAIT`.
  - Pausing freezes it, and Stop ends it.
  - `WAIT 0` sends nothing, and a negative time is an error.
- **`SETSPEED level`** sends `Event::SetSpeed(level)`, from 1 to 11. Anything else is an error.
  - The animator changes speed when it gets there, and reports it in its frame, so the app moves the slider.
- **`SPEED`, decided:** it outputs the level the slider shows.
  - `Control::speed` is an atomic that the animator keeps up to date, whether the slider moved or a `SETSPEED` ran.
  - It's 0 when there's no slider, so `SPEED` outputs 11 in the CLI.
  - The kdl's `SPEED` description says so, and `WAIT` has a `differs` note: it waits at every speed, and does nothing in the CLI.
- **The CLI** ignores both events.
- **Tests:** `tests/speed.rs` (3 tests), and 3 animator tests: a `WAIT` at Instant taking half a second with its highlight, Pause and Stop during a `WAIT`, and `SETSPEED` changing the speed mid-program.
