# 11. Turtle speed is a GUI concern: interpreter thread, bounded channel, animator

Status: Accepted
Date: 2026-10-17 (recorded later, from commit 8104814 and `plan-1.md`)

## Context

On modern hardware Logo draws everything instantly. A headline feature of the playground is letting the user choose the turtle's speed, from instant down to slow enough to watch every step, and pause, step and stop while a program runs. Running the program first and replaying a recording isn't enough: endless programs never finish, input has to happen while the user can see the drawing, and Stop has to halt the real program.

## Decision

- **The interpreter runs on its own `std::thread`,** with a large stack for deep recursion (ADR 0007). Not on gpui's background executor, because a blocked task would tie up one of its shared worker threads.
- **It sends events (ADR 0009) through a bounded channel** to an **animator** on the UI thread, which plays them onto the canvas at the chosen speed. When the channel is full the interpreter blocks, so it stays only a few events ahead of the screen, and pausing the animator pauses the program.
- **The interpreter never knows about speed or time.** Speed is purely the animator's concern, so changes apply immediately, even mid-move, and the CLI stays simple.
- **Sync points:** before anything that depends on what the user sees (reading input, the mouse or keyboard, the end of the program), the interpreter sends a `Sync` event with a one-shot reply channel and waits for the animator to reach it.
- **Frames:** the animator calls `window.request_animation_frame()` while it has work, and takes events with `try_recv`, so no async plumbing is needed.
- **Speeds:** Instant processes events for a fixed time slice (around 8 ms) per frame. Animated speeds use a distance per second and an angle per second, so the turtle glides along moves and visibly turns.
- **`WAIT`** is honoured at every speed, including Instant.
- **`SETSPEED n` / `SPEED`** are our own extension: a `SetSpeed` event changes the speed at exactly that point in the program.
- **Stop** sets a cancel flag that the interpreter checks on every evaluation step, and drops the channel so a blocked interpreter wakes up. Programs can't `CATCH` it.

## Consequences

- Pause, Step and the running-instruction highlight build on this (ADR 0012).
- Open questions in `plan-1.md`: the speed units and the channel size. Spike 04 answers both.
