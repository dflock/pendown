# Text blocks

Status: resolved
Type: task

## What

A ```` ```text ```` block shows its lines as code, in the code font and on the code background, with no Load into editor or Run. It isn't a program: `Lesson::programs` and `program_named` skip it, and the examples test doesn't run it.

Today every code block that isn't a `picture` is a program (`tutorial.rs`, `code`), so a block of letters like `F-F++F-F` would get a Run button and fail.

## Done when

- A unit test: a lesson with a text block has the programs it had without it.
- It looks right in the panel (a screenshot, once lesson 8 uses one).

## Comments

2026-11-01: Done: `tutorial::Kind` (program, picture or text), and `kind`, which `programs`, `pictures`, `program_named` and the panel use. A text block is plain, in the code font, with no highlighting. Unit test: `text_is_neither_a_program_nor_a_picture`.
