# 28. Pause, edit and replay, no Stop, and Single Step

Status: Accepted
Date: 2026-10-31

## Context

Testing with real users (`.scratch/pause-and-edit/feedback-from-user-testing.md`) found three things:
- People want to change the program while it's paused, then carry on with the change.
- Having both Stop and Pause is confusing: from where the learner sits, both stop the turtle.
- So is having both Step line and Step instruction: with one instruction per line, which is how beginners write, they do the same thing.

ADR 0006 made the editor read-only while a program is running or paused, to keep spans and the highlight accurate, and noted it might feel restrictive.

## Decision

- **One Pause, no Stop.** The toolbar is Run, Pause (Resume while paused) and Single Step.
- **Pause finishes the current instruction, at once,** then pauses before the next. Kame is never left part-way along a move, an arc or a shape, and the highlight is on the next instruction to run. Finishing at the slider's speed would make Pause seem not to work at speed 1, where a long `FD` takes seconds. A `WAIT` in progress ends.
- **Single Step replaces Step line and Step instruction,** doing what Step instruction did: it runs to the next instruction, inside brackets and into procedures. Step line couldn't step inside a one-line loop. What goes is stepping a whole line of several instructions in one press.
- **The editor can be changed while paused,** not while running. As soon as its text differs from the paused run's, the highlight goes, since it would point at the wrong text. Undoing back to the same text brings it back.
- **Resume and Single Step after a change replay the changed program:**
  - It runs from a clean slate (ADR 0006), with the same `RANDOM` seed, out of sight, up to as many instructions as the paused run had shown.
  - Then it carries on, or takes its step, at the slider's speed.
  - The GUI picks each Run's seed and sends it with the job, so a replay can use it again.
  - The replay runs in lockstep, with the animator answering each `Sync` at once. So the interpreter is never more than a line ahead, and when the replay gets there, Resume and Single Step work as after any pause.
  - Nothing is shown during a replay but the drawing appearing: no highlight, no animation and no thinking.
- **Replay, not live editing.** Keeping the paused run alive and swapping in the changed procedures would leave the procedure and loops it's in the middle of on their old code, so some changes would work and others wouldn't, with no way to see why. Replaying applies every change the same way, wherever it is.
- **Without a change, Resume carries on** as it did.
- **A changed program that doesn't parse** shows its errors as Run does (ADR 0005), and stays paused.
- **A changed program that ends, or stops with an error, before it gets back there** just finishes, as any run does, with no message. To a learner, the program is the code in the editor: they pressed Resume and it finished. A message about "your changed program" and "where you paused" would explain the machinery, not anything they can see.
- **A console entry while paused ends the paused program,** as Stop did, and runs against what it left (ADR 0006). After that there's nothing to resume. This is how a `FOREVER` program is ended to use the console, now Stop's gone.
- **Run while paused** starts the editor's program afresh, with a new seed.
- **Only the editor's program replays.** A paused help example or console entry carries on as before, whatever's in the editor, since its code isn't the editor's.
- **Ending a run** (Run again, or a console entry while paused) still cancels the interpreter as Stop did (ADR 0011), so `CATCH` still never catches it.

## Consequences

- ADR 0006's read-only editor while paused, ADR 0012's two step buttons and Step holding a `Sync`, and the Stop button of ADRs 0011, 0021 and 0022 are changed by this ADR.
- A replay is "run quickly to a point", so breakpoints (`.scratch/breakpoints/`) can use the same machinery.
- After a change that alters what happens before the paused point, Kame carries on from wherever the new program is after that many instructions, which may not be the same place.
