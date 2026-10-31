# The run stops at breakpoints

Status: needs-triage
Type: task
Blocked by: pause-and-edit 02, 04, 07

## What

- `Control` gets the breakpoints: the editor's line numbers, shared with the interpreter thread and changeable while a program runs, like `cancel` and `lockstep`.
- In `Interp::begin` (`pendown-logo/src/interp.rs`), when an instruction starts a line from the editor's source and that line is a breakpoint, it sends a `Sync` and waits, even when not in lockstep. Lines from other sources (help examples, console entries) never stop.
- The `Instruction` event says it's at a breakpoint, or a separate event says so, so the animator knows that `Sync` is one to hold, not one to answer straight away.
- The animator, when it reaches a breakpoint's `Sync` while Running or in a Single Step, holds it and pauses, with the highlight on the line's first instruction. Resume and Single Step from there work as after any pause, and don't stop again at the breakpoint they start on.
- During a replay (pause-and-edit 04), the animator answers breakpoint `Sync`s at once until it gets back to its target.
- `App` keeps the breakpoints, and sets `Control`'s copy whenever they change. A breakpoint on a `TO` line is sent to the interpreter as the line of the procedure's first instruction.

## Done when

Interpreter tests show it stopping at a breakpoint line with lockstep off, not at other lines, and not on another source's line with the same number. Animator tests show:
- a run pausing at a breakpoint at a slow speed and at Instant, with the same marks as stepping there;
- pausing each time round a multi-line loop;
- stopping once on a one-line `REPEAT`;
- Resume from a breakpoint carrying on;
- a breakpoint added while running stopping the run;
- a replay passing a breakpoint before its target without stopping.

## Comments
