# Pen and colours

Status: done
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

### 2026-10-21: done

- **Colours** are in `src/colour.rs` (ADR 0015):
  - UCBLogo's wxWidgets palette;
  - the 148 CSS names, in sorted order and searched by binary search;
  - `[r g b]` and `[r g b a]`, and `#rgb`, `#rgba`, `#rrggbb` and `#rrggbbaa`.
  
  An unknown name suggests up to three close names (Levenshtein distance, at least 2, or names that contain it). A bad hex word gets its own message.
- **The commands** are in `prims/pen.rs`: `PENUP`/`PU`, `PENDOWN`/`PD`, `PENDOWNP`/`PENDOWN?`, `SETPENCOLOR`/`SETPC`, `PENCOLOR`/`PC`, `SETPENSIZE`, `PENSIZE`, `SETBACKGROUND`/`SETBG` and `BACKGROUND`/`BG`.
  - The reporters output the form the colour was set in.
  - `PENSIZE` outputs `[w w]`, because the pen is square.
  - Run resets the pen, its colour and the background.
- **The background:** `SetBackground { color }` is always opaque.
  - `Drawing::set_background` redraws everything over the new colour, and `Drawing::reset` goes back to black, for Run.
  - The animator reports the background in its frame, and the app and CLI pass it on.
- **Tests:**
  - `tests/pen.rs` (8 tests), and 4 colour unit tests;
  - 2 new render tests: the background going under the drawing, and translucent blending;
  - a `colours` picture.
