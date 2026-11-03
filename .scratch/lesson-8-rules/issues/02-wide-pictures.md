# Pictures the shape of their drawing

Status: resolved
Type: task

## What

A lesson's picture is always `LESSON_PICTURE` square, and drawn with `help::picture_options()`, which fits the drawing into a square. A wide drawing, such as a line above a bump, comes out small, with black above and below.

- A picture is the shape of its drawing's bounds, plus a margin: as wide as the panel's text, at most, and no taller than it is now. A square drawing looks as before.
- The saved pictures in `crates/pendown/tests/help-pictures/` are drawn the same way, so the test checks what the panel shows. Regenerate any that change with `UPDATE_PICTURES=1`, and look at them.
- Help examples' pictures stay as they are, unless they use the same code and it's simpler to change both. Say which in a comment.

## Done when

- A wide picture fills the panel's width, at its narrowest and widest.
- The existing lesson pictures look as before (screenshots).

## Comments

2026-11-01: Done: `Drawing::bounds` and `Drawing::svg_cropped` in `pendown-render`, and a `crop` option in `pendown-cli`, used by `help::lesson_picture_options`. The panel reads the SVG's shape and draws the picture 160 px tall, or as wide as the panel if that's less. Help examples' pictures stay square. Square pictures look as before, but tall or off-centre ones are now cropped to the drawing, so the house in lesson 4 is bigger and narrower than it was.
