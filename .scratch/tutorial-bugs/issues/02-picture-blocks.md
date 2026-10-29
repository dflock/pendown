# Picture blocks in lessons

Status: needs-triage
Type: task

## What

- A fenced block marked `picture` in a lesson shows only its program's picture, 160 pixels, drawn as Help's are, with the caption "What Kame meant to draw", and no code, Load or Run.
- `Lesson::programs` stays the code blocks with Load and Run; a new `Lesson::pictures` gives the picture blocks.
- The examples test runs picture blocks and keeps their pictures as `tutorial-L-picture-N`.

## Done when

A picture block shows as in the mockups, and the examples test covers it.
