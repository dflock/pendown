# Session checks for Undo and Teach Kame

Status: done
Type: task
Blocked by: 02, 03

## What

Add sections to the scripted session (`autotest.rs`): type entries, undo some (including one that `CLEAN`s and one that errors), check the drawing's marks and Kame's position and heading after each; then teach Kame the rest, check the editor's text, and type the new word at the console.

## Done when

The session passes, and its screenshots are saved with the others.

## Comments

### 2026-10-29: from issue 02

Undo's checks are in the session already, in the "Console undo" sections, with the `ConsoleKeys`, `UndoConsole` and `CheckConsoleTyped` steps. This issue adds Teach Kame's, and anything Undo still lacks.

### 2026-10-29: done

New sections, each starting from its own Run so `PENDOWN_AUTOTEST_FROM` can start at it: Undo brings the background back; long and multi-line entries' labels are cut short; Run leaves nothing to undo or teach; with an example showing, Undo brings the user's drawing back first; the dialog's rows (a definition that can't be ticked, a "Uses TRI" note, an error unticked), its picture, a name defined at the console, All and None; Escape closes it with nothing changed; the editor's Undo takes a taught procedure back out, and a copied one. Screenshot `teach-rows`. New steps: `TeachAll`, `CheckTeachRows`, `CheckTeachProblem`, `CheckTeachPicture`, `CheckDialog`, `CheckUndoLabel`, `CheckCanTeach`, `Keys`, `EditorKeys`, `CopyDefinitions` and `CheckBackground`.

Not checked in the session: a multi-line entry that can be taught. Only definitions can span lines at the console, and they can't be ticked; `teach::procedure`'s unit test covers the indenting.

The checks found three bugs, now fixed:

- Teach Kame and the console's Copy to editor replaced the editor's whole text, so the editor's Undo couldn't take them back out, though the spec said it could. They now insert at the end as Help's Copy to editor does (`append_to_program` calls `copy_to_editor`).
- A procedure defined at the console, then again after a Run, was kept twice. Teach Kame's picture then failed, as its program defined it twice, and Copy to editor would have put it in the editor twice. Both now use only the newest of each name (`newest_definitions`).
- A definition's row said it used the procedure it defines.
