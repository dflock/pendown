# 7. Recursive evaluator, with tail calls eliminated and a depth limit

Status: Accepted
Date: 2026-10-18

## Context

Recursion is idiomatic Logo, and often endless: `TO SPIRAL :n FD :n RT 90 SPIRAL :n + 1 END` never stops. UCBLogo eliminates tail calls, so programs like this run in constant memory. Without that, any interpreter runs out of stack however large it is. In Rust, a stack overflow aborts the whole process and can't be caught.

The evaluator also has to support dynamic scope (kept from UCBLogo, see `implementation-plan.md`), Stop, Pause and Step, and the higher-order commands in phase 2 (`MAP`, `FOREACH`, `APPLY`).

## Decision

**An ordinary recursive tree-walking evaluator** in Rust, running on the interpreter thread with a large stack (`std::thread::Builder::stack_size`).

- `STOP`, `OUTPUT`, `THROW` and errors travel back up as `Err` or return values. Stop from the GUI is a separate kind of error that `CATCH` never handles.
- Pausing and stepping work by blocking the thread on the channel, as `implementation-plan.md` describes, so the evaluator doesn't need to save its state.
- Phase 2's higher-order commands simply call back into the evaluator.

**Tail calls are eliminated by trampolining.**

- A call is in tail position when it's the last instruction of a procedure body, or the expression after `OUTPUT`. The last instruction of an `IF` or `IFELSE` list that is itself in tail position counts too, as in UCBLogo.
- A tail call returns a "call this next" value to the procedure-call loop, which runs it without growing the Rust stack.

**Dynamic scope uses a stack of variable frames on the heap,** separate from the Rust stack.

- On a tail call, the callee's inputs and locals are added to the current frame, replacing any variables with the same names, instead of pushing a new frame.
- Under dynamic scope this gives exactly the same results as a new frame: the caller never resumes, and any of its variables the callee doesn't replace would be visible to the callee anyway.
- The frame stays a bounded size, because a program has a fixed set of variable names. So `SPIRAL` runs forever in constant memory.
- The heap frames are also what a later variable inspector shows.

**There's a depth limit on calls that aren't tail calls.** The evaluator counts them and stops with a friendly error, such as "SQUARE called itself too many times", well before the stack runs out. Spike 03 measures how deep a 256 MB stack allows, and the limit is set from that.

## Alternatives considered

**An explicit-stack machine** (bytecode with heap frames, no Rust recursion) would have no depth limit, and saving and inspecting state would be easier. But pausing already works by blocking the thread, and the variable frames are already on the heap. It would be much more work, mostly in phase 2's higher-order commands, which would all have to be written as resumable frames, for little gain.

## Consequences

- Deep recursion that isn't a tail call (a deep tree fractal, a naive recursive `FIB`) has a fixed limit. It should be generous enough that real programs don't reach it.
- The tail-position rules must be tested carefully, including calls inside `IF` and `IFELSE`.
