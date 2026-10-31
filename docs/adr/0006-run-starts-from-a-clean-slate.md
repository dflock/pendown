# 6. Run starts from a clean slate; the console uses what the last run left

Status: Accepted
Date: 2026-10-18

## Context

Classic Logo has a workspace: procedures and variables pile up as you type them, and stay until you erase them. In a GUI with an editor that's confusing, because a procedure deleted from the editor can still exist and still be called. We need to define what the Run button does, how the console relates to the editor, and what spans point at (ADR 0001).

## Decision

**Run** does the following, in order:

1. Stops any running program.
2. Clears all procedures and variables.
3. Resets the screen: clears the drawing, puts the turtle home, sets the pen and colours back to their defaults, and picks a new random seed. The speed slider keeps its setting.
4. Parses the whole editor buffer. If there are parse errors, it shows them (ADR 0005) and doesn't run.
5. Defines every `TO` in the buffer, then runs the top-level instructions from top to bottom.

So the same code always draws the same picture, apart from `RANDOM`.

**The console**, when no program is running, runs instructions against whatever the last run left: its procedures, variables, turtle and drawing. After running a program you can try `SQUARE 50` at the console. `TO` typed at the console works, but the next Run clears it. The console can offer "Copy to editor" for those definitions.

**The editor is read-only while a program is running or paused.** Stop unlocks it. This keeps spans and the instruction highlight accurate without having to map them through edits. (Changed by ADR 0028: the editor can be changed while paused, and Resume replays the changed program. There's no Stop.)

**Files:** one program per file, plain UTF-8 text with a `.logo` extension, with Open and Save. No imports in phase 1.

**Spans** name their source: either the editor buffer for this run, or a numbered console entry.

## Consequences

- No obsolete workspace commands are needed (`ERASE`, `PO`, `SAVE`, `LOAD`…), as `pendown-logo.kdl` already says.
- Making the editor read-only while running may feel restrictive, for example when a `FOREVER` program is running. Revisit after spike 04 if so.
- Procedures defined at the console only last until the next Run, which the GUI should make clear.
