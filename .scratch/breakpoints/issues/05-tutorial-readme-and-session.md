# The tutorial, README and the scripted session

Status: needs-triage
Type: task
Blocked by: 02, 04

## What

- The tutorial's first lesson (`tutorial/01-moving-and-turning.md`), after Pause and Single Step: click a line number to put a red dot there, and Kame stops when it gets to that line, so you can watch closely from there. Click the dot to take it away. If the program has a loop or procedure, the lesson where it's introduced may suit this better; choose when writing.
- The README, in the paragraph about running, pausing and stepping.
- The scripted session, in a section of its own:
  - It sets a breakpoint inside a procedure called in a `REPEAT`, runs at an animated speed, and checks that the run stops there each time round, with the highlight on the line.
  - It does the same at Instant.
  - It sets a breakpoint while running, and checks that the run stops there.
  - It edits above a breakpoint while paused, and checks that the dot moves with its line.
  - It checks that a help example ignores the editor's breakpoints.
  - It takes screenshots of each.

## Done when

The tutorial and README explain breakpoints, and the whole session passes.

## Comments
