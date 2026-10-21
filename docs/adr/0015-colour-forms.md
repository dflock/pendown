# 15. Colours: palette numbers, RGB(A) 0–255, CSS names and hex

Status: Accepted
Date: 2026-10-18 (recorded later, from commit 5bc4b58)

## Context

UCBLogo accepts a palette number or an RGB list, with RGB values as percentages from 0 to 99. Modern users know colours from CSS and image editors, where values run from 0 to 255 and colours have names. jsLogo accepts CSS names, and FMSLogo uses 0–255.

## Decision

Everywhere a colour is accepted (`SETPENCOLOR`, `SETBACKGROUND`, `SETFILLCOLOR`, `SETPALETTE`, `FILLED`), all of these forms work:

- **Palette number:** 0 to 15, with UCBLogo's default colours. `SETPALETTE` (phase 2) can redefine numbers from 8 up.
- **RGB list** `[r g b]`, each value **from 0 to 255**: a deliberate change from UCBLogo's percentages.
- **RGBA list** `[r g b a]`, where the opacity also runs from 0 (invisible) to 255 (solid).
- **CSS colour name,** any case: `"red`, `"cornflowerblue`, from the 148 CSS named colours.
- **Hex word:** `"#ff8800`, the short form `"#f80`, and with opacity `"#ff880080` and `"#f808`.

The forms can't be confused: a number is a palette index, a list is RGB or RGBA, a word starting with `#` is hex, and any other word is a name. Unknown names and bad hex are errors, and an unknown name suggests the closest known ones.

Reporters (`PENCOLOR`, `BACKGROUND`, `FILLCOLOR`) output the colour in the form it was set, so `SETPENCOLOR PENCOLOR` always works. `PALETTE` outputs RGB, or RGBA if the colour isn't solid. The background is always solid, so its opacity is ignored. Events carry the resolved RGBA (ADR 0009).

## Consequences

- UCBLogo programs that use RGB lists will come out darker, since their values are read as 0–255 instead of percentages.
- The pixel buffer blends colours with opacity. Spike 02 found it matches gpui's blending, but a translucent line is darker where two moves meet, since each move is its own stroke (ADR 0010).
