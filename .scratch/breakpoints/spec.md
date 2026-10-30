# Breakpoints

From testing with real users (`feedback-from-user-testing.md`): people want to "run to here", getting quickly to the part of the program they care about, then watching or stepping from there. That's a breakpoint: click a line number in the editor, and Kame stops when the program gets to that line.

Builds on pause-and-edit (`.scratch/pause-and-edit/`): Pause finishing the instruction, replaying a changed program, and Single Step. Do that first.

## What "done" looks like

- **Click a line number to set a breakpoint;** a red dot appears over the number, and the line is faintly tinted. Click again to take it away. There can be as many as you like.
- **Breakpoints can be set at any time:** before a run, while it's running, and while paused. A breakpoint set while running takes effect straight away.
- **A run stops at a breakpoint,** before the first instruction of that line runs, with the highlight on it, as if Pause had been pressed there. Run, Resume and Single Step all stop at them. "Run to here" is a breakpoint and Run, or Resume.
- **It stops every time the program gets to that line:** each time round a loop that spans several lines, and each time a procedure with a breakpoint in it is called.
- **At every speed, Instant too,** and it stops exactly on the line.
- **Only lines with code take a breakpoint.** A blank line, a comment or `END` can't have one: clicking its number does nothing. A breakpoint on a `TO` line stops each time the procedure is called, at its first instruction.
- **Breakpoints stay on their lines as you edit.** Adding a line above one moves it down with its line. Deleting a breakpoint's line deletes the breakpoint. Undo brings back the line, but not the breakpoint.
- **No message** when a breakpoint stops Kame: the highlight on the line, the dot, and the Resume button show what happened.
- **The tutorial explains them,** where it explains Pause and Single Step.

## Decisions

- **As many breakpoints as you like.** One would be no easier: it's a set of lines either way.
- **The gutter, not a button or a menu.** It's where every code editor puts them, and it's one click. gpui-kit's editor has no gutter click event and no gutter markers, so we add them from outside (issue 04), with a patch to gpui-kit as the fallback.
- **The interpreter knows the breakpoints,** through `Control`, as it knows the cancel flag. On a breakpoint line it sends a `Sync` and waits even when not in lockstep, and the animator holds it and pauses. At Instant the interpreter runs ahead of the animator, so without this it would be lines past a breakpoint before the animator saw it (ADR 0012). Checking only breakpoint lines keeps Instant fast.
- **A breakpoint stops where a line starts,** as `starts_line` does (ADR 0012): when the program comes to that line from another one. So `REPEAT 4 [FD 50 RT 90]`, all on one line, stops once, before the `REPEAT`. To go round it, use Single Step. Spreading the loop over several lines makes each line a place to stop.
- **Replays ignore breakpoints** until they get back to where you paused (pause-and-edit), so a change that's replayed doesn't stop at a breakpoint it already passed.
- **Only the editor's program stops at them.** A help example or console entry ignores them, since their code isn't the editor's: the interpreter only checks lines from the editor's source.
- **Not saved with the program.** Breakpoints are for now, like the cursor and the speed. Opening a file or New clears them.
- **No message when one stops Kame.** The highlight and the dot show it. As with a changed program finishing early, a message would explain the machinery, not anything the learner needs.
- **Clearing is clicking each dot.** A "Clear breakpoints" button can come later, if testing shows it's wanted.

## Not in this plan

- Conditional breakpoints ("stop when `:size` is less than 10").
- Breakpoints in help examples.
- A list of breakpoints, or turning one off without removing it.

## How the work is split

| Issue | What | Blocked by |
|---|---|---|
| 01 | ADR 0029: breakpoints | |
| 02 | The run stops at breakpoints | pause-and-edit 02, 04, 07 |
| 03 | Breakpoints follow their lines through edits | |
| 04 | The gutter: clicking sets and clears, and the dots | 03 |
| 05 | The tutorial, README and the scripted session | 02, 04 |
