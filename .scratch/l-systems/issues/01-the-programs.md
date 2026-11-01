# The lesson's programs

Status: resolved
Type: task

## What

Write and check every program in lesson 8 (see the spec's "What done looks like"), before the text, so the text can describe what they really draw:

- the Koch curve at one level, and levels 0 to 3 side by side;
- the Koch snowflake;
- the Sierpinski gasket;
- the Sierpinski arrowhead, as two procedures, `A` and `B`, that call each other;
- the Gosper curve, two procedures, with colour;
- the bug: a Koch curve with one call the wrong size, and a `picture` block of what Kame meant.

Each is centred in the turtle area, doesn't wrap, uses only what lessons 1–7 teach, and finishes in a few seconds at the normal speed. Upper-case commands and names, two-space indents, as the other lessons.

## Done when

- Each renders with `pendown-cli` to a picture that looks right.
- They're short enough to read: no procedure longer than the Gosper rules need.

## Comments

2026-11-01: Done: the Koch curve at level 2, levels 0–3 one under another, the snowflake at level 3, the gasket at level 4, the arrowhead at level 4, Gosper at level 3 (A's lines gold, B's blue, so you can see which rule drew each), and the bug (the third call is `:size / 2`). All placed with PU moves, as `SETPOS` isn't taught. The arrowhead flips at odd levels, so it's at 4 and a Try this explains.
