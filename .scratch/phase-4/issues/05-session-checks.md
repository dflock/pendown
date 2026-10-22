# Checks in the scripted session, and cost

Status: done
Type: task
Blocked by: 04

## What

The scripted session pauses a program with nested loops and a procedure, checks the panels' text, steps, checks again, and takes screenshots. `perf` and the budget tests pass.

## Done when

The new checks pass in release and in debug, and `perf` is within its baseline.

## Comments

### 2026-10-22: done

Two new sections in the scripted session: stepping instruction by instruction through a procedure called from `FOREACH`, checking the highlight and the panels at each step, then resuming at Instant to the end; and a `FOREVER` at Instant, then Stop. `CheckPanelsDrawn` checks the panes drew the latest `Watch`, which is how the stale panes were found. Screenshots `panels` and `panels-instant`.

The session passes all 54 checks in release. `perf` passes, every case within 8% of the baseline; the lockstep case, which builds a `Watch` for every instruction, is about 7% slower.

A session with the window unfocused failed the tree's frame-interval check once: gpui throttles an unfocused window to about 30 fps, which a 60 Hz display rounds to every third vsync. It passed on the next run.
