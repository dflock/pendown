# The metrics module

Status: done
Type: task

## What

`pendown_logo::metrics`, shared by every crate:
- **The flag:** `enabled()`, on by default in debug builds and off in release. `PENDOWN_METRICS=1` turns it on, `PENDOWN_METRICS=0` off. Read once.
- **`Counter`** and **`Timer`** (total time and count), kept as atomics. Off, recording one is a check of the flag.
- **Per-thread CPU time,** from `/proc/self/task/*/schedstat`, grouped as the UI (the main thread), the interpreter, and the others. Empty on other platforms.

## Done when

Unit tests cover the flag's parsing, counters and timers, and finding the test's own thread's CPU time.

## Comments

### 2026-10-22: done

`pendown_logo::metrics`: `enabled()` and `set_enabled()`, `Counter`, `Timer`, `ThreadCpu` (main, interpreter, other) and `this_thread_cpu()`, all from `schedstat` in nanoseconds. Unit tests in the module.
