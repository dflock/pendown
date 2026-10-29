# The animator says what to think

Status: done
Type: task

## What

- Each frame, the animator gives what Kame is thinking, beside the partial line and Kame's place: a turn (where, the heading it started from, how far so far) or a move (from, to so far, length so far), and the one before it with how far it has faded.
- It stays until the next instruction that moves or turns Kame starts, then fades over a third of a second.
- Nothing at Instant.
- Unit-tested with the animator's existing made-up frame times.

## Done when

Animator tests cover a turn and a move in progress, the fade, a turn of more than 360°, and Instant showing nothing.

## Comments

### 2026-10-29: done

`animator.rs` has `Thought` (a `Turn` at a point, from a heading, by the degrees so far, signed and past 360 when it is; or a `Move` from a point, to where Kame has got, with the length so far) and `Thinking` (the current thought, and the one before with how much of it is left, from 1 to 0). `Animator::thinking()` gives them. A move or turn that starts becomes the current thought, and the last starts fading over `FADE`, a third of a second of frame time; zero-length moves and zero turns don't count. At Instant there's nothing, and changing to Instant clears it. Arcs (`ARC`) and shapes aren't thoughts.

Tests in `tests/animator.rs`: a turn growing to 90, a move's length so far matching where Kame is, the fade as the next starts and its end, `RT 450` and `LT 30`, and nothing at Instant.

For issue 03: each console job gets a new animator, so its thoughts start empty. Driving's joined entries (three presses of → making `RT 45`) need the window to keep the entry's turn or move from press to press.

`filled_shapes_with_the_pen_up_take_the_time_of_their_outline` failed once while all the animator tests ran together, then passed 30 times alone, with and without this change: a timing flake that was there before.
