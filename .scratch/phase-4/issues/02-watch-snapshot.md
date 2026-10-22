# The Watch snapshot, and when it's sent

Status: done
Type: task
Blocked by: 01

## What

`Event::Watch` carries the counters, outermost first, and the scopes the running instruction can see: the globals, then each frame of the call chain with its variables, each marked if a later scope hides it. Values are formatted in short form on the interpreter thread.

It's sent with lockstep on, after an `Instruction`, when the snapshot differs from the last one sent; at Instant, when `Control::watch` asks; and at the end of each run. Only with instruction events on, so the CLI never pays for it.

## Done when

Tests cover scopes, shadowing, spellings, hidden special variables, counters, a tail call's frame, `FOR`, lambda templates, `LOCAL` with no value, an array changed by `SETITEM`, and each of the times it's sent.

## Comments

### 2026-10-22: done

`pendown_logo::watch` has the types, and `Interpreter::watch()` builds one from the shallow-bound `vars` and each frame's saved values. Two things changed on the way:
- **It's sent just before the `Instruction` that sees it,** not after. "Step instruction" pauses as soon as it reaches an `Instruction`, so a `Watch` after it would wait until the next step.
- **Each job starts with no last `Watch`,** so a console entry after Stop sends one even if the interpreter sent the same one into the stopped run's channel.

`Style` has a `length` limit, so a 300,000-member list is shown in microseconds. A tail call replaces its caller's scope, as ADR 0007 says, so the caller's variables show in the callee's scope. 16 tests in `tests/watch.rs`.
