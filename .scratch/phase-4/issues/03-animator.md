# The animator plays Watch in step

Status: done
Type: task
Blocked by: 02

## What

The animator keeps the latest `Watch` it has reached, and a frame says when it changed, so the window shows it with the highlight it belongs to.

## Done when

An animator test steps instruction by instruction through a loop that changes a variable, and checks the counter and the value at every step.

## Comments

### 2026-10-22: done

`Animator::watch` holds the last `Watch` reached, and `Frame::watch_changed` says when it changes. A `Watch` waits for the dwell on an instruction that did nothing, as the next `Instruction` does, so the panels never get ahead of the highlight. Tests step instruction by instruction through a loop, and follow a running program.
