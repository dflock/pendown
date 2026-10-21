# Pen and colours

Status: ready-for-agent
Type: task
Blocked by: 05

## What

- **Commands:**
  - `PENDOWN`/`PD`, `PENUP`/`PU`, `SETPENCOLOR`/`SETPC`, `SETPENSIZE`, `SETBACKGROUND`/`SETBG`;
  - `PENDOWNP`, `PENCOLOR`/`PC`, `PENSIZE`, `BACKGROUND`/`BG`.
- **Every colour form, everywhere a colour is accepted (ADR 0015):**
  - palette numbers 0 to 15;
  - `[r g b]` and `[r g b a]` from 0 to 255;
  - the 148 CSS names, in any case, with suggestions for unknown names;
  - hex `#rgb`, `#rgba`, `#rrggbb` and `#rrggbbaa`.
- **The reporters** output the colour in the form it was set, so `SETPC PC` always works.
- **`SETPENSIZE`** takes a number, or `[width height]` as in UCBLogo, with the height ignored (a square pen).
- **The background** is always solid. `SETBACKGROUND` sends a `SetBackground` event, and `pendown-render` redraws the background under the drawing.
- **The defaults are UCBLogo's:** a black background and a white pen.
- **Translucent pens blend** in the pixel buffer. Each move is its own stroke, so they're darker where moves meet (ADR 0010).

## Done when

- `pendown-logo` has tests for every colour form, for each reporter's round trip, and for the errors, including name suggestions.
- `pendown-render` has tests for the background change under an existing drawing, and for translucent blending.
- A `pendown-cli` picture test uses several colour forms and pen sizes.

## Comments
