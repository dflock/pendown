# 11. Turtle speed is a GUI concern: interpreter thread, bounded channel, animator

Status: Accepted
Date: 2026-10-17 (recorded later, from commit 8104814 and `implementation-plan.md`). Updated 2026-10-21 after spike 04.

## Context

On modern hardware Logo draws everything instantly. A headline feature of the playground is letting the user choose the turtle's speed, from instant down to slow enough to watch every step, and pause, step and stop while a program runs. Running the program first and replaying a recording isn't enough: endless programs never finish, input has to happen while the user can see the drawing, and Stop has to halt the real program.

## Decision

- **The interpreter runs on its own `std::thread`,** with a large stack for deep recursion (ADR 0007). Not on gpui's background executor, because a blocked task would tie up one of its shared worker threads.
- **It sends events (ADR 0009) through a bounded channel** to an **animator** on the UI thread, which plays them onto the canvas at the chosen speed. When the channel is full the interpreter blocks, so pausing the animator pauses the program. Events go in batches of up to 256, and the channel holds 4 batches (spikes 03 and 04).
- **At animated speeds, the interpreter waits at each line** until the animator reaches it (lockstep, ADR 0012), so it's never more than the current line ahead of the screen. Only Instant lets it run ahead.
- **The interpreter never knows about speed or time.** Speed is purely the animator's concern, so changes apply immediately, even mid-move, and the CLI stays simple.
- **Sync points:** before anything that depends on what the user sees (reading input, the mouse or keyboard, the end of the program), the interpreter sends a `Sync` event with a one-shot reply channel and waits for the animator to reach it.
- **Frames:** the animator calls `window.request_animation_frame()` while it has work, and takes events with `try_recv`, so no async plumbing is needed.
- **Speeds are levels from 1 to 11.** 11 is Instant. Below it, each level doubles the speed of the one before, from 5 turtle steps a second at level 1 to 2,560 at level 10, and the turtle turns at 3 degrees per step. So the turtle glides along moves and visibly turns.
- **Instant** takes events for 8 ms each frame, waiting on the channel if it's empty until the slice is used up, and draws at most about 6 ms of segments. Taking only what's already in the channel would cap Instant at a channelful a frame: spike 04's 131,000-move tree took 8.9 s that way, and 0.2 s with waiting.
- **An instruction that moves nothing** keeps the highlight for the time of 10 steps, at most 0.25 s, so it can be seen at slow speeds.
- **`WAIT`** is honoured at every speed, including Instant.
- **`SETSPEED n` / `SPEED`** are our own extension: a `SetSpeed` event changes the speed at exactly that point in the program.
- **Stop** sets a cancel flag that the interpreter checks on every evaluation step, and drops the channel so a blocked interpreter wakes up. Programs can't `CATCH` it.

## Consequences

- Pause, Step and the running-instruction highlight build on this (ADR 0012).
- The speed units and the channel size were open questions in `implementation-plan.md`; spikes 03 and 04 answered both.
- The animator is a plain state machine with no gpui code, so it can be tested with a real interpreter thread and made-up frame times, as spike 04's tests do.
- Stop takes well under a millisecond, from any state: running, blocked on a full channel, or waiting at a held line (spike 04).
