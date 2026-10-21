# Lists, words and printing

Status: done
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

### 2026-10-21: done

- **The commands,** in `prims/data.rs`:
  - `LIST` and `SENTENCE`/`SE`, with their variable-input forms;
  - `FIRST`, `LAST`, `BUTFIRST`/`BF`, `BUTLAST`/`BL` and `ITEM`;
  - `COUNT` and `EMPTYP`/`EMPTY?`.
- **Words:** the selectors work on words letter by letter, counting Unicode characters, and numbers are words, so `FIRST 345` is `3` and `(BF 345) + 1` is `46`.
- **Printing:** `PRINT`, `TYPE` and `SHOW` are unchanged from spike 03. `PRINT` drops a list's outer brackets, `SHOW` keeps them, and `TYPE` adds no spaces or newline.
- **Errors:**
  - An empty input or an `ITEM` index out of range is "X doesn't like ... as input", using the name as typed.
  - An empty word now reads as "the empty word" in every such message.
- **Tests:** `tests/lists.rs` (6 tests): constructors, selectors on lists, words and numbers, printing nested lists, `POS` and `SETPOS` with lists, and errors.
