# Replaying a program to a number of instructions

Status: needs-triage
Type: task
Blocked by: 02, 03

## What

A way to start a program so that it plays out of sight up to `n` instructions shown, then carries on in a given mode (Running, Paused or StepInstruction, which is Single Step's mode) at the slider's speed.

In the animator, a replay target: until `instructions_shown` reaches `n`, it draws as at Instant, replies to each `Sync` at once (the run stays in lockstep, so the interpreter is at most a line ahead), shows no highlight and no thinking, and doesn't animate. When it gets there, it shows the highlight and the watch panels as they are, and takes the mode. If the program ends first, it ends as any run does. The run reports that it ended before reaching `n`, but only so the app knows the run is over: the learner isn't told (see the spec).

This is the machinery "run to here" will use later, so it takes a count, not anything about pausing.

## Done when

Animator tests: replaying a program to `n` ends with the same marks, turtle and highlight as playing it and pausing after `n` instructions; a program shorter than `n` ends, with the same marks as playing it to the end, and reports that it ended early; at a slow speed, the replay takes about as long as at Instant.

## Comments
