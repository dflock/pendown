# Lists, words and printing

Status: ready-for-agent
Type: task
Blocked by: 01

## What

- **Lists:**
  - `LIST`, `SENTENCE`/`SE`, `FIRST`, `LAST`, `BUTFIRST`/`BF`, `BUTLAST`/`BL`, `ITEM`, `COUNT`, `EMPTYP`/`EMPTY?`;
  - the variable-input forms, such as `(SENTENCE a b c)` and `(LIST a b c)`.
- **Words:** the same selectors work on words, and numbers are words (ADR 0002), so `FIRST 345` is `3` and `COUNT 345` is `3`.
- **Printing:** `PRINT`/`PR`, `TYPE` and `SHOW`, with their variable-input forms.
  - `PRINT` drops a list's outer brackets, and `SHOW` keeps them.
  - Output goes to the console as `Print` events, in step with the drawing.
- **Errors** for empty inputs (`FIRST []`) and an `ITEM` index out of range.
- `POS` and `SETPOS` use lists (issue 05), so check the two work together.

## Done when

`pendown-logo` has tests for each command on lists, words and numbers, and for the errors, including printing nested lists.

## Comments
