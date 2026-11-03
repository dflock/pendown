# Picture captions from the fence

Status: resolved
Type: task

## What

A `picture` block's caption is the rest of its fence line: ```` ```picture The four lines, in order ````. Today `help_panel.rs`'s `lesson_picture` always says "What Kame meant to draw".

- `markup.rs` keeps the whole rest of the line for a picture block (its `name` is one word, for programs). A caption, or none.
- No caption: none is shown.
- The eight existing pictures (lessons 4 to 8) get ```` ```picture What Kame meant to draw ````, so they look as before.
- Markdown viewers read only the first word after the fence, so the files still show as code there.

## Done when

- A unit test in `markup.rs` parses a picture block's caption, and one without.
- The lessons with pictures look as before (the scripted session's tutorial screenshots).

## Comments

2026-11-01: Done: the rest of the fence line is the block's `name`, a program's name or a picture's caption (`markup.rs`, with a test). There were eight existing pictures, not three, all in "Help Kame fix the bug" sections; each now says "What Kame meant to draw" in its fence.
