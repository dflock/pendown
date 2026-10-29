# Console: Undo, and Teach Kame

Two console improvements from `mindstorms/mindstorm-implementation-plan.md`. **Undo** takes back the last console entry: its marks and Kame's move. This is Mindstorms Figure 3's `ERASE 1`, and it makes trial and error cheap. **Teach Kame** turns entries typed at the console into a procedure: pick the entries, give the word a name, and Pendown puts `TO name … END` in the editor. This is the book's central metaphor as a button: you do it, then teach Kame the word.

They work together. Fiddle at the console, undo the wrong turns, then teach Kame what's left.

There are no new Logo commands. The UI design is on a canvas of mockups: https://claude.ai/artifact/U7KGqpYMoNArvMXhPinaNN (private to the owner unless shared).

## What "done" looks like

- **Undo:**
  - the console's title bar has an **Undo** button, labelled with the entry it takes back: `Undo RT 45`. It's disabled while something is running, and when there's nothing to undo;
  - **Ctrl+Z** (Cmd+Z on macOS) in the console's input line does the same, when the line is empty. With text in the line, it undoes typing, as it does now;
  - undoing puts the drawing and Kame back to how they were before the entry: Kame's position, heading, pen (up or down, colour, size), fill colour, shown or hidden, and the background. The Variables panel's colours follow;
  - you can undo again and again, back to the last Run. Run starts again with nothing to undo;
  - an entry that stopped with an error, or was stopped, can be undone too: it takes back what it drew before it stopped;
  - an entry that `CLEAN`ed brings back what it cleaned;
  - the console prints `Undid: RT 45`. What the entry printed stays in the console: the console is a record of what happened;
  - the undone entry stays in the Up/Down history, so Up and Enter are "redo".
- **Teach Kame:**
  - the console's title bar has a **Teach Kame** button, enabled once there's an entry since the last Run that can go in a procedure;
  - it opens a dialog listing the entries since the last Run, oldest first, each with a tick box. Entries that ran are ticked. Entries that stopped with an error are shown unticked, with the error. Undone entries aren't listed. Definitions (`TO … END`) are listed but can't be ticked, with the reason;
  - **All** and **None** buttons; Shift-click ticks or unticks a range;
  - a name field, with a reason under it when the name can't be used;
  - a preview of the procedure as it'll go in the editor, with the code highlighted, and a picture of what it draws from the centre, facing up;
  - **Teach Kame SQUARE** (the name as typed) puts `TO SQUARE … END` at the end of the editor's program, and defines it at the console too, so `SQUARE` works straight away. The editor's Undo takes it out again, as with Copy to editor;
  - the console prints `Kame knows SQUARE now. It's at the end of your program.`.

## Decisions

- **Undo takes back what Kame did, not what Kame learned.** It restores the drawing and Kame, and leaves variables and procedures as they are. `MAKE` and `TO` aren't undone. Mindstorms' `ERASE 1` is about the picture, and taking back a variable that a procedure since defined depends on would surprise. An ADR records this (issue 01).
- **Kame's state is saved on the interpreter's thread.** `Turtle` holds `Value`s, which hold `Rc`s and can't leave the thread (ADR 0022). Before each console entry, `Interpreter::run_console` pushes a copy of `turtle` onto a stack; a new `Job::Undo` pops one and puts it back, and ends as every job does, with a `Watch` of the globals, so the colours panel follows. `run_program` clears the stack.
- **The window saves the drawing's state.** Before each console entry it saves the number of marks, the background, and the animator's position, heading and shown flag (as `run_console` already carries them over). If the entry `CLEAN`s, the window saves the marks the first `CLEAN` threw away. Undo cuts the drawing's marks back to the saved count, or puts back the saved marks, and draws it all again, as `Drawing::set_background` does. That's a new `Drawing::truncate`.
- **The console's entries get a record of their own,** next to the Up/Down history, in `console.rs`: each entry's text, how it ended (ran, error, stopped), and whether it's been undone. The history leaves out repeats, so it can't do this job. The window keeps each entry's saved drawing state beside it.
- **Ctrl+Z only undoes an entry when the input line is empty,** so it never fights with undoing typing. The Undo button's tooltip says `Undo (Ctrl+Z)`.
- **No redo button.** Up brings the entry back, and Enter runs it again.
- **Teach Kame uses what was typed, as it was typed.** It doesn't turn four `FD 100 RT 90`s into a `REPEAT`: seeing that is the learner's next idea, not Pendown's. Each entry goes on its own line, indented two spaces; a multi-line entry keeps its lines.
- **No inputs yet.** The procedure has no `:size`. Adding an input is something to do in the editor, and a later improvement could offer it.
- **Names:** one word, starting with a letter, of letters, digits, `.`, `_` and `?`. Not a primitive's name or alias (`FD is already a Logo word`), and not a procedure already defined in the editor's program or at the console (`SQUARE is already in your program. Choose another name, or change it in the editor.`). Case doesn't matter (ADR 0004). The name is shown in capitals in the button and the code.
- **The picture is drawn as Help's are**, headless on a background thread with `pendown_cli::render`, fitted to the drawing, from a clean slate. It's redrawn when the ticks change, and the newest one wins. The procedures defined at the console and in the last run are included, so an entry that called `BOX` draws.
- **The dialog is a gpui-kit `Dialog`**, modal over the window. Escape closes it and nothing changes.
- **An entry that calls a procedure defined only at the console** is ticked as usual, with a note under it: `Uses BOX, so BOX has to be in your program too`. Copy to editor puts it there.
- **The console is still plain text.** Lines like `Undid: RT 45` can't be greyed, as the mockups show them, without making the console's output a styled list. That's left for later.
- **Teaching defines the procedure at the console** by running the `TO … END` as a console entry of its own, which isn't recorded as an entry or a console definition (so it's not offered to Copy to editor, since it's already in the editor).

## How the work is split

| Issue | What | Blocked by |
|---|---|---|
| 01 | The console's entries record, Kame's saved state and `Job::Undo`, and the ADR | |
| 02 | Undo in the window: the drawing's saved state, the button, Ctrl+Z | 01 |
| 03 | Teach Kame: the dialog, names, the preview and picture, putting it in the editor | 01 |
| 04 | Session checks and screenshots for both | 02, 03 |
