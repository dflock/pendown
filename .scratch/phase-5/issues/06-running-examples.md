# Running examples, and Copy to editor

Status: done
Type: task
Blocked by: 05

## What

Run plays an example on the main canvas on a session of its own, highlighted in the panel, with a banner. The user's run, drawing and panels come back with Back to my drawing, or Run. Copy to editor adds the example to the end of the program, undoably.

## Done when

An example runs, steps and stops like a program, and the user's drawing comes back.

## Comments

### 2026-10-23: done

Run on an example sets aside the user's animator, run, text, drawing and variables (`Saved`), and plays the example from `run::start`, an interpreter of its own, so the user's procedures and variables are untouched. The highlight goes to the example's code in the panel, and Run becomes Stop. A banner over the canvas names the example, with Back to my drawing. The picture stays when the example ends; Back to my drawing puts the user's things back, Run replaces them as it would anyway, and a console entry puts them back first, since it works on what the user's program left. An error in an example shows in the console as "In the example, line N", and not on the editor's code.

Copy to editor moves the cursor to the end and inserts the example after a blank line, as one edit, so Undo takes it out.

The scripted session runs FORWARD's first example at 20 steps a second, checks the highlight, checks the user's 48 marks come back, and copies the second example to the editor.
