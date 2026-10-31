# No Stop: the toolbar, the console while paused, help and the tutorial, README and the session

Status: resolved
Type: task
Blocked by: 05, 07

## What

- Take Stop off the toolbar. Run, Pause/Resume and Single Step stay.
- The console works while paused: an entry ends the paused program (as `App::stop` does now, which stays, without its button) and runs against what it left. After that there's nothing to resume.
- The help for `CATCH` (`pendown-logo.kdl`) says the Stop button is never caught: it becomes ending a run, by Run or a console entry while paused.
- The tutorial's first lesson (`tutorial/01-moving-and-turning.md`), where it introduces **Pause** and **Single Step**:
  - Pause lets Kame finish what it's doing, then waits.
  - While Kame's paused, you can change your program. **Resume** runs the changed version from the start, quickly, up to where you were, then carries on.
  - To end a program that would go on forever, pause it and type in the console.
- The help's other mentions of the toolbar stay true. The obsolete stepping commands' reasons name Pause and Single Step (issue 07). Check for any other mentions of Stop.
- The README: Stop, Pause and the editor while paused.
- The scripted session's 25 uses of Stop end runs with Pause and Run, or Pause and a console entry, instead. Its timing of how long ending a run takes stays.

## Done when

No Stop anywhere in the UI, help, tutorial or README; the tutorial explains editing while paused; a `FOREVER` program can be paused and the console used; the whole session passes.

## Comments

2026-10-31: Done.
- The toolbar is Run, Pause/Resume and Single Step. The help panel's examples and lesson programs had Stop buttons too: they now always say Run, which starts them again, and Back to my drawing ends an example.
- A console entry while paused ends the paused program (printing "Stopped.", as Stop did) and then runs.
- The scripted session's `Stop` step is now `EndRun`, which still calls `App::stop`, what Run and console entries use, so its timings stay. There's a new section, "typing in the console ends a paused program".
- The redraw-count check in phase 3 now takes focus off the window first. A focused editor redraws to blink its caret, so an earlier section that pressed keys in the editor made it fail.
