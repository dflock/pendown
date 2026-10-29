# The canvas draws Kame thinking

Status: needs-triage
Type: task
Blocked by: 01

## What

- In `canvas.rs`, over the drawing and under Kame: the turn's faint wedge, its old and new heading lines, and its number; the move's ruler with ticks every 10 and longer every 50, dropping to every 50 or 100 when they'd be under 4 pixels apart, and the length so far.
- Sizes in screen pixels: the arc's radius 40, the ruler 10 from the line.
- A toolbar toggle, **Show thinking**, with an icon, off at first.
- Never in Export.

## Done when

Screenshots show a turn and a move mid-way at a slow speed, as in the mockups, and an export of the same drawing has neither.
