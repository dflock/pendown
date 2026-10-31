# Pause, edit and carry on

From testing with real users (`.scratch/pause-and-edit/feedback-from-user-testing.md`): people want to change the program while it's paused, then carry on with the change. And having both Stop and Pause is confusing: from where they sit, both stop the turtle. So is having both Step line and Step instruction: with one instruction per line, which is how beginners write, they do the same thing.

So: there's no Stop, only Pause, and one step button, **Single Step**. Pause lets the current instruction finish, then pauses. While paused, the editor can be changed. Resume and Single Step carry on with the changed program by **replaying** it: running the new program from a clean slate, with the same `RANDOM` seed, at Instant and out of sight, up to as many instructions as the old run had shown, and then carrying on from there at the chosen speed.

Replaying applies every change the same way, wherever it is: in a procedure that's running, in a loop that's halfway round, or at the top level. Changing the `REPEAT 12` that's running to `REPEAT 8` works straight away. Keeping the paused run alive and swapping its procedures wouldn't do that: the procedure and loops it's in the middle of keep their old code, so some changes would work and others wouldn't, with no way to see why.

## What "done" looks like

- **No Stop button, and one step button.** The toolbar is Run, Pause (Resume while paused) and Single Step.
- **Single Step runs to the next instruction,** as Step instruction does now: inside brackets, and into procedures.
- **Pause finishes the current instruction, then pauses.** Kame is never left halfway along a line or an arc, and the highlight is on the next instruction to run. What's left of the instruction is drawn at once, so Pause takes effect straight away even at slow speeds.
- **The editor can be changed while paused,** not while running. As soon as the program differs from the one that's paused, the highlight goes, since it would point at the wrong text. Undoing back to the same program brings it back.
- **Resume after a change replays.** The drawing clears, the changed program runs at Instant up to as many instructions as the paused run had shown, then carries on at the slider's speed, with the highlight back. Single Step replays the same way, then takes its step.
- **Resume without a change carries on,** as Resume does now: no replay.
- **If the changed program doesn't parse,** Resume shows the errors as Run does (ADR 0005) and stays paused.
- **If the changed program ends, or stops with an error, before it gets that far,** it just finishes, as any run does: no message. An error is shown as at any other time, on the line that caused it.
- **The help says how it works:** the tutorial's first lesson, where it introduces Pause, says that while Kame's paused you can change your program, and Resume runs the changed version from the start, quickly, up to where you were, then carries on.
- **The console works while paused.** An entry ends the paused program first, as Stop did, and runs against what it left (ADR 0006). After that there's nothing to resume, and Run starts afresh. This is how a `FOREVER` program is ended to use the console, now Stop's gone.
- **Run while paused** starts the editor's program afresh, as now, with a new seed.

## Decisions

- **Replay, not live editing.** See above. A program replays from a clean slate (ADR 0006), so a replay draws what the changed program would have drawn by then.
- **"As far" means as many instructions shown,** counted by the animator (`instructions_shown`), one per `Instruction` event. It's what the learner saw. If the change alters what happens before that point, Kame carries on from wherever the new program is after that many instructions, which may not be the same place.
- **The same seed:** the GUI picks each Run's seed and sends it with the job, instead of the interpreter picking one, so a replay can use it again. Unchanged code then draws the same picture, `RANDOM` and all. Run picks a new seed, as now.
- **The replay runs in lockstep,** with the animator replying to each `Sync` at once. So the interpreter is never more than a line ahead of the replay, and when the replay arrives, Step and Resume work exactly as they do after a pause. Free-running Instant would let the interpreter run ahead of the point, including effects that aren't events, like `MAKE`.
- **Nothing is shown during a replay** but the drawing appearing: no highlight, no animation and no thinking. The watch panels catch up when it arrives.
- **The highlight goes when you change the program,** not when you pause, since stepping needs it. Whether the program has changed is a comparison of the editor's text with the paused run's.
- **Pause finishing the instruction at once.** Finishing at the slider's speed would make Pause seem not to work at speed 1, where a long `FD` takes seconds. An instruction is the unit Single Step uses: a procedure call is one, and the first instruction inside it is the next.
- **No message when a replay ends early.** To a learner, the program is the code in the editor. They pressed Resume and it finished, as it does whenever you resume near the end. A message about "your changed program" and "where you paused" would ask them to picture an old program and a new one, which is how the replay works inside, not anything they can see. If this confuses people in testing, the fix belongs in the help, not in a console message.
- **Single Step, not Step line.** Single Step can do everything Step line could, and Step line can't step inside a one-line loop like `FOREVER [FD 1 RT 1]`. What goes is stepping a whole line of several instructions in one press. Resume then Pause, or a faster slider, still does that, and "run to here" will do it better.
- **Help examples aren't replayed.** Their code isn't the editor's. Changing the editor while an example is paused doesn't affect the example, and its Resume carries on as now. The same goes for a paused console entry.
- **Ending a run** (Run again, or a console entry while paused) still cancels the interpreter as Stop did, so `CATCH` still never catches it, and the metrics still measure how long it takes.

## Not in this plan

- **Run to here,** also from the feedback: breakpoints, planned in `.scratch/breakpoints/`, which builds on this. The replay ignores breakpoints until it gets back to where you paused.
- **A keyboard shortcut for Pause.**

## How the work is split

| Issue | What | Blocked by |
|---|---|---|
| 01 | ADR 0028: pause, edit and replay, and no Stop | |
| 02 | Pause finishes the current instruction | |
| 03 | The GUI picks each Run's seed | |
| 04 | Replaying a program to a number of instructions | 02, 03 |
| 05 | Editing while paused, and Resume and Single Step replaying | 01, 04 |
| 06 | No Stop: the toolbar, the console while paused, help and the tutorial, README and the session | 05, 07 |
| 07 | Single Step: one step button | |
