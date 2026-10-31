# Single Step: one step button

Status: needs-triage
Type: task

## What

Step line and Step instruction become one button, **Single Step**, which does what Step instruction does now: it runs to the next instruction, including inside brackets and into procedures. With one instruction per line, which is how beginners write, the two buttons did the same thing, and having both was confusing (`.scratch/pause-and-edit/feedback-from-user-testing.md`). Step instruction is the one to keep because it can do everything. Step line can't step inside a one-line loop like `FOREVER [FD 1 RT 1]`, which is why ADR 0012 added the second button.

What goes is stepping a whole line of several instructions in one press. Resume then Pause, or a faster slider, still does that, and "run to here" later will do it better.

- `panes.rs`: one button, "Single Step", with the footprints icon, where the two were. `app.rs`: `step` loses its `instruction` flag.
- `animator.rs`: no `Mode::StepLine`, `step_line()` or holding the next line's `Sync`. Line `Sync`s and lockstep stay: Pause needs them, so the interpreter is waiting at the next line (ADR 0012). Remove `after_line_start`, `own_line` and `shown_this_step` if nothing else then uses them.
- Wording: the six obsolete stepping commands' reasons in `pendown-logo.kdl` ("Pendown's Pause and Step line buttons do that") name Pause and Single Step; the tutorial's first lesson and the README the same.
- ADR 0012 gets a line saying the two buttons became Single Step, and why. If ADR 0028 (issue 01) is written by then, the line goes in it, with 0012 pointing at it. If not, the line goes in 0012 itself.

## Done when

The toolbar has one step button, Single Step. The animator tests' Step line cases are removed, or become Single Step cases where they test something Single Step still does. The scripted session's uses of Step line step with Single Step instead, and the whole session passes. No "Step line" or "Step instruction" is left in the UI, help, tutorial or README.

## Comments
