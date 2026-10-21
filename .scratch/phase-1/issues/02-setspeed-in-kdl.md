# `SETSPEED` and `SPEED` in `pendown-logo.kdl`

Status: ready-for-agent
Type: task

## What

`SETSPEED` and `SPEED` are our own extension (`implementation-plan.md`, "Speed model"), but they aren't in `pendown-logo.kdl` yet. Add both, following the other extensions (`extension #true`, a `source`), in the pen-and-background or turtle-and-window section:

- **`SETSPEED level`**, phase 1: sets the turtle's speed to a level from 1 to 11 (ADR 0011).
  - Level 11 is Instant. Below it, each level doubles the speed, from 5 turtle steps a second at level 1 to 2,560 at level 10.
  - The speed changes at exactly that point in the program, and the slider moves to match.
  - It's an error outside 1 to 11 or for a fractional level.
  - The CLI ignores it.
- **`SPEED`**, phase 1: outputs the current level.
- `source "Python turtle.speed()"`, as the plan says.

Also check that the phase-1 count is right: 87 from the manual, plus these two. The plan's "Phase 1: Turtle drawing (87 commands)" heading should then say 89, or count the extensions separately.

## Done when

The two entries are in the file, it still parses, and the plan's count matches the file.

## Comments
