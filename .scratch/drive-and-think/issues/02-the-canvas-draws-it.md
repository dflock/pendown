# The canvas draws Kame thinking

Status: done
Type: task
Blocked by: 01

## What

- In `canvas.rs`, over the drawing and under Kame: the turn's faint wedge, its old and new heading lines, and its number; the move's ruler with ticks every 10 and longer every 50, dropping to every 50 or 100 when they'd be under 4 pixels apart, and the length so far.
- Sizes in screen pixels: the arc's radius 40, the ruler 10 from the line.
- A toolbar toggle, **Show thinking**, with an icon, off at first.
- Never in Export.

## Done when

Screenshots show a turn and a move mid-way at a slow speed, as in the mockups, and an export of the same drawing has neither.

## Comments

### 2026-10-29: done

`canvas.rs` draws `Overlay::thinking` over the drawing and under Kame: a turn's two heading lines, its wedge (shaded, with its edge) and its number; a move's ruler on Kame's left (by `Thought::Move`'s new `heading`, so `BK` keeps it on the same side), with ticks every 10, 50 or 100 steps so they're at least 4 pixels apart, longer every fifth, and its length. Numbers are in the editor's font, bold. The fading thought is drawn the same, fainter.

Changes from the plan: numbers are whole while turning or moving, and exact (to one place) once done, which needed a `done` flag on each `Thought`; `59.8°` mid-turn was noise. The colour is pale yellow on a dark background and dark amber on a light one. The toolbar button says **Thinking**, with the lightbulb icon (added to `ExtraIcons`): "Show thinking" pushed Help off the toolbar at the default window width. Issue 03's Drive button will need room too.

Export draws from the drawing's marks, which thinking never touches, so it's never in an export.

Session: "Show thinking" section, with `SetThinking`; screenshots `thinking-turn`, `thinking-move` and `thinking-light`.
