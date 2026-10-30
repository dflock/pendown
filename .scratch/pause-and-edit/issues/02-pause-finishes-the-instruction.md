# Pause finishes the current instruction

Status: needs-triage
Type: task

## What

Pause, while running or during a Single Step, lets the instruction being drawn finish, then pauses before the next `Instruction` event, so Kame is never halfway along a move, an arc or a shape. What's left of the instruction is drawn at once, not at the slider's speed.

In the animator: Pause plays what's left of `current` straight away, then waits in a mode like `StepInstruction`, which pauses when the next `Instruction` arrives. An instruction that draws nothing pauses at once. With nothing left to come (the program ended), Pause does nothing.

## Done when

Animator tests show Pause mid-move ending with the move drawn whole and the highlight on the next instruction, at a slow speed and at Instant. The scripted session's pause screenshot shows it.

## Comments
