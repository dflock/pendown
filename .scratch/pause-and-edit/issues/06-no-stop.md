# No Stop: the toolbar, the console while paused, help and the tutorial, README and the session

Status: needs-triage
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
