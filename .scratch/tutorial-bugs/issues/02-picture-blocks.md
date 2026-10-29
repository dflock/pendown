# Picture blocks in lessons

Status: done
Type: task

## What

- A fenced block marked `picture` in a lesson shows only its program's picture, 160 pixels, drawn as Help's are, with the caption "What Kame meant to draw", and no code, Load or Run.
- `Lesson::programs` stays the code blocks with Load and Run; a new `Lesson::pictures` gives the picture blocks.
- The examples test runs picture blocks and keeps their pictures as `tutorial-L-picture-N`.

## Done when

A picture block shows as in the mockups, and the examples test covers it.

## Comments

### 2026-10-29: done

`pendown::tutorial::PICTURE` marks a fenced block as a picture. `Lesson::programs` is now only the other code blocks, so Load into editor and Run keep their numbers, and `Lesson::pictures` gives the picture blocks; a unit test checks both. The Tutorial tab draws a picture block with Help's `picture` and `svg_image`, 160 pixels square, captioned "What Kame meant to draw", with no code or buttons. The examples test runs pictures as `tutorial-L-picture-N`.

Checked by adding a picture block to lesson 2 for a screenshot, then putting the lesson back. Pictures are fitted around where Kame starts, so a drawing that goes up and right sits in a corner; issue 03's picture programs centre themselves with a pen-up move first, which the learner never sees.
