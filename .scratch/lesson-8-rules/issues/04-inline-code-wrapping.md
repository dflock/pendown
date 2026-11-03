# Inline code doesn't break inside itself

Status: wontfix
Type: task

## What

Inline code in the help and lessons wraps at punctuation: in lesson 8, `F-F++F-F` comes out as `F-F+` at the end of one line and `+F-F` at the start of the next. A code span should break only at its spaces, so `F-F++F-F` moves to the next line whole. A long span with spaces, like `KOCH :size / 3 :level - 1`, can still break at them.

Find where gpui's line wrapper breaks after `+` and `-`, and stop it inside code spans, for example with word joiners (U+2060) between their characters if the font draws them as nothing, or by giving the wrapper the span as one word.

## Done when

- `F-F++F-F` never breaks, at any panel width (a screenshot of lesson 8 at the panel's narrowest).
- Copying text from the panel, if it can, copies no invisible characters.

## Comments

2026-11-01: Dropped: it only shows when a span lands at the end of a line at narrow panel widths, and the rewrite removes the worst case, the run-on level 2 rule.
