# 7. Recursive evaluator, with tail calls eliminated and a depth limit

Status: Accepted
Date: 2026-10-18. Updated 2026-10-21 after spike 03.

## Context

Recursion is idiomatic Logo, and often endless: `TO SPIRAL :n FD :n RT 90 SPIRAL :n + 1 END` never stops. UCBLogo eliminates tail calls, so programs like this run in constant memory. Without that, any interpreter runs out of stack however large it is. In Rust, a stack overflow aborts the whole process and can't be caught.

The evaluator also has to support dynamic scope (kept from UCBLogo, see `implementation-plan.md`), Stop, Pause and Step, and the higher-order commands in phase 2 (`MAP`, `FOREACH`, `APPLY`).

## Decision

**An ordinary recursive tree-walking evaluator** in Rust, running on the interpreter thread with a large stack (`std::thread::Builder::stack_size`).

- `STOP`, `OUTPUT`, `THROW` and errors travel back up as `Err` or return values. Stop from the GUI is a separate kind of error that `CATCH` never handles.
- Pausing and stepping work by blocking the thread on the channel, as `implementation-plan.md` describes, so the evaluator doesn't need to save its state.
- Phase 2's higher-order commands simply call back into the evaluator.

**Tail calls are eliminated by trampolining.**

- A call is in tail position when it's the last instruction of a procedure body, or the expression after `OUTPUT`. The last instruction of an `IF` or `IFELSE` list that is itself in tail position counts too, as in UCBLogo. So does the last expression of the list chosen by `OUTPUT IFELSE ...`. `REPEAT` and `FOREVER` lists are never in tail position.
- A tail call written as an instruction can't output: if `TO F G END` and `G` outputs, running `F` is "You don't say what to do with" that value, as in UCBLogo.
- A tail call returns a "call this next" value to the procedure-call loop, which runs it without growing the Rust stack.

**Dynamic scope uses a stack of variable frames on the heap,** separate from the Rust stack.

- On a tail call, the callee's inputs and locals are added to the current frame, replacing any variables with the same names, instead of pushing a new frame.
- Under dynamic scope this gives exactly the same results as a new frame: the caller never resumes, and any of its variables the callee doesn't replace would be visible to the callee anyway.
- The frame stays a bounded size, because a program has a fixed set of variable names. So `SPIRAL` runs forever in constant memory.
- The heap frames are also what a later variable inspector shows.
- **Variables use shallow binding** (spike 03). Each name has one slot holding its current value. A frame records the values its inputs and locals replaced, and puts them back when it ends. That gives the same results as walking the frames, but every lookup is an array index. The inspector rebuilds the chain from the saved values.

**There's a depth limit on calls that aren't tail calls.** The evaluator counts them and stops with a friendly error, such as "SQUARE called itself too many times", well before the stack runs out.

- **The limit is 10,000 calls,** on a 256 MB stack. Spike 03 measured up to 4.8 KB of stack a call in release, so 256 MB fits about 56,000; 10,000 leaves five times headroom.
- **A stack guard backs it up.** At every call and every list parse, the evaluator checks how much stack is left, from the stack pointer. With less than 256 KB left, it gives the same friendly error, whatever the depth. So recursion that uses more stack than measured, or a debug build, still can't crash.
- **`pendown-logo` builds at `opt-level = 1` in the dev profile.** Unoptimised, a call uses up to 43 KB of stack, so debug builds would stop at about 6,000 calls. At `opt-level = 1` it's about 5 KB, close to release.

## Alternatives considered

**An explicit-stack machine** (bytecode with heap frames, no Rust recursion) would have no depth limit, and saving and inspecting state would be easier. But pausing already works by blocking the thread, and the variable frames are already on the heap. It would be much more work, mostly in phase 2's higher-order commands, which would all have to be written as resumable frames, for little gain.

## Consequences

- Deep recursion that isn't a tail call (a deep tree fractal, a naive recursive `FIB`) has a fixed limit. It should be generous enough that real programs don't reach it. 10,000 is "good for now"; raise it if real programs reach it.
- The 256 MB stack is reserved address space: only the pages a program actually uses take memory.
- The tail-position rules must be tested carefully, including calls inside `IF` and `IFELSE`.
