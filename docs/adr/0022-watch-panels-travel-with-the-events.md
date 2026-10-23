# 22. The counters and variables panels travel with the events

Status: Accepted
Date: 2026-10-22

## Context

Phase 4 adds two panels that show what the running program sees: the repeat counters of every loop running, and the variables of the globals and of each procedure in the call chain. They have to agree with the highlighted instruction, or they'd teach the wrong thing: a panel that says `:n` is 3 while the highlighted `FD :n` draws a line 2 long is worse than no panel.

The interpreter runs on its own thread, ahead of the animator by up to a channel's worth of events at Instant, and by at most the current instruction at animated speeds (ADR 0012). A `Value` holds `Rc`s, so it can't leave the interpreter's thread.

## Decision

- **The interpreter sends the panels' contents as an event,** `Event::Watch`, in the same stream as the moves and the highlight. The animator shows each one when it reaches it, so the panels are exactly as far behind the interpreter as the drawing is.
- **A `Watch` is sent just before the `Instruction` that sees it.** "Step instruction" pauses as soon as it reaches an `Instruction`, so one sent after would wait for the next step. The animator holds a `Watch` back while an instruction that drew nothing is still on show, as it does the next `Instruction`.
- **When it's sent:**
  - **With lockstep on** (animated speeds, paused, stepping): before any instruction that sees something different from the last `Watch`. Line starts aren't enough, since a line can be a whole loop, such as `REPEAT 36 [FD 10 RT 10]`.
  - **At Instant:** before the next instruction after the animator asks, through `Control::watch`, which it does each frame. The panels then show what the interpreter sees, which may be ahead of the drawing, as pausing from Instant already is.
  - **At the end of each job,** with only the globals. After Stop nothing is listening, so the GUI drops the counters and scopes itself.
- **The interpreter formats the values,** in short form: at most 4 lists deep, 20 members wide and 200 characters long, so a 300,000-member list costs no more than a short one.
- **It carries the pen, fill and background colours too,** in the form they were set and as RGBA for a swatch. They aren't variables, but they change as the program runs, so `SETPC` sends a new `Watch` as `MAKE` does.
- **It finds the scopes from the shallow binding** (ADR 0007): `vars` holds what's visible, and each frame's saved values are what its own variables hid. A tail call shares its caller's frame, so the caller's variables show in the callee's scope.

## Consequences

- A `Watch` is built for every instruction in lockstep, and compared with the last one sent. In `perf`, 5,000 lines in lockstep are about 7% slower; at Instant without being asked, the cost is two atomic loads an instruction.
- The colours are formatted only when they change, and shared between `Watch`es: formatting them for every one made 5,000 lines in lockstep 25% slower, found in phase 5's check.
- The CLI turns off instruction events, and with them every `Watch`.
- The panels are cached views. gpui doesn't redraw a cached view when an entity it read is notified during a draw, so the window notifies the panels themselves when a `Watch` arrives, and asks for one more frame so they redraw even when paused.
