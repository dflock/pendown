# A bug in each later lesson

Status: done
Type: task
Blocked by: 03

## What

A short "Help Kame fix the bug" at the end of the inputs, recursion, and colours lessons: `BOX :size` that always draws 100; a `TREE` that comes out in a tangle, because it forgets to turn back before it ends; `SETPC` after the line it was for. Each with the picture Kame meant, the program, a question, and one way to fix it.

## Done when

The examples test passes: every bug runs without errors, and draws the wrong picture.

## Comments

### 2026-10-29: done

A "Help Kame fix the bug" at the end of lessons 5 and 6, and before "What next?" in lesson 7, each with the picture Kame meant (centred with a pen-up move), the program with the bug, a question pointing at where to look, and one way to fix it in a sentence:

- inputs: `BOX :size` with `FD 100`, so three boxes come out the same;
- recursion: `TREE` without its last `LT 30`, which comes out in a tangle, not leaning as the plan guessed, so the lesson says tangle;
- colours: each `SETPC` after the square it was meant for, so the first square is white and there's no sky blue.

Checked by drawing each picture and bug to a contact sheet. Every program runs without errors.
