# The editor: syntax colours and errors while typing

Status: ready-for-agent
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
