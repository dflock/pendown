# Interpreter metrics

Status: done
Type: task
Blocked by: 01

## What

In `Control`, which the interpreter and the GUI share: instructions run, time running, time waiting in lockstep for the animator's reply, and time waiting on a full channel. Instructions are published as the program goes, so the overlay can show a live rate even for programs that draw nothing.

## Done when

Tests show the counts for a fixed program, and waits measured when the receiver is slow.

## Comments

### 2026-10-22: done

`InterpreterMetrics` in `metrics.rs`, held by `Control`: instructions (published every 4,096 and at the end of a job), running, lockstep and channel timers. `BatchSender::measured` records the channel waits; the GUI's session uses it. Tests: `tests/metrics.rs`.
