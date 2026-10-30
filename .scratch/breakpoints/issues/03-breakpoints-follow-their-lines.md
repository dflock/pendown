# Breakpoints follow their lines through edits

Status: needs-triage
Type: task

## What

A pure function, with no GUI, that takes the old text, the new text and the breakpoints' line numbers, and gives the new line numbers:
- It compares the lines, finding the lines the two texts share before and after the change.
- A breakpoint on a line that's still there moves with it.
- A breakpoint on a line that was deleted or changed goes.
- An edit within a line keeps that line's breakpoint.
- Lines added above a breakpoint move it down.

It also says which lines can take a breakpoint: lines with code, from the tokens, so it works while the program doesn't parse. Blank lines, comment lines and `END` can't. A breakpoint left on a line that no longer has code goes.

`App` calls it on each editor change and keeps the result.

## Done when

Unit tests cover:
- typing on a breakpoint's line;
- adding lines above and below one;
- deleting its line;
- pasting several lines;
- turning a line into a comment;
- undo bringing back a deleted line without its breakpoint;
- a text that doesn't parse.

## Comments
