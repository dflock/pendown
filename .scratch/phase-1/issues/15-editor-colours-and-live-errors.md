# The editor: syntax colours and errors while typing

Status: done
Type: task
Blocked by: 03

## What

- **Syntax colours** from `pendown-logo`'s own reader and tokens, not spike 01's separate lexer, so the editor and the interpreter agree on what's a word, a number, a variable or a comment (ADR 0008).
  - Known commands and procedures defined in the buffer get their own colours.
  - The whole buffer is re-coloured on every edit; spike 01 measured that as fast enough.
- **Parse errors while typing** (ADRs 0001 and 0005): the editor parses the whole buffer statically, with the arities of every `TO` in it, and shows each error as a diagnostic.
  - Errors include unknown procedures, not enough inputs, unmatched brackets and a missing `END`.
  - **Lists:** lists are only parsed when they run (ADR 0001), so also try parsing list literals that look like instructions, such as the input to `REPEAT`, `IF`, `IFELSE` and `FOREVER`, to catch `REPEAT 4 [FD]`.
  - Debounce re-parsing if it's slow on big buffers. Spike 01's 500-line program is a good test.
- **Hovering** a diagnostic shows its message (spike 01 confirmed this works).

## Done when

- The colours follow the theme.
- Typing `FOOBAR 3` or `REPEAT 4 [FD]` shows a diagnostic without running anything.
- The parse takes under a few milliseconds on a 500-line program.

## Comments

### 2026-10-21: done

- **`pendown_logo::syntax`:**
  - **`highlight(text)`** scans like the reader (words, brackets, comments), and splits each word with the parser's own `split_word`. So the editor and the interpreter agree on what's a word, a number, a variable or a comment.
    - The kinds are comment, bracket, number, quoted word, variable, operator, `TO`/`END`, built-in command, procedure defined in the text, and unknown word.
    - In the editor, built-ins and `TO`/`END` use the theme's "keyword" colour, and the program's own procedures "function". Unknown words get no colour, since they get a diagnostic.
  - **`check(text)`** parses statically, with the arities of every `TO` in the text, through a new `parse::parse_program_partly` that keeps what parsed alongside the errors. It then also parses list literals given to `REPEAT`, `FOREVER`, `IF` and `IFELSE`, so `REPEAT 4 [FD]` shows its error without running. Other lists, such as `MAKE "x [FD]`, aren't data the checker can judge, so they're left alone.
  - **Speed:** both together take under 1 ms for spike 01's 500-line program, in debug and release, so they run on every edit without a debounce.
- **The app:**
  - `highlighter.rs` uses `highlight`, replacing spike 01's lexer.
  - Every edit runs `check` and replaces the diagnostics, which also clears the last run's highlight.
  - Open, New and Copy to editor check too, because gpui-kit's `set_value` sends no change event.
  - Diagnostics count columns in UTF-16, as LSP positions do.
- **Tests:**
  - `syntax` unit tests: colours, errors while typing and a timing test;
  - the scripted session types `FOOBAR 3` and `REPEAT 4 [FD]` and checks both diagnostics, then checks they go away. It has 36 checks, all passing.
- **Also fixed:** "Run cleared the procedures you defined in the console" now shows once, not on every Run.
