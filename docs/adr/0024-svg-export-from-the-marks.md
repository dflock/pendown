# 24. SVG export is drawn from the marks, over the whole turtle area

Status: Accepted
Date: 2026-10-23

## Context

PNG export saves the pixel buffer (ADR 0010). An SVG stays sharp at any size and can be edited in a drawing program, and everything Pendown draws so far is a vector shape: lines, arcs, ellipses and rectangles (ADRs 0016 and 0017). ADR 0010 noted that an SVG export couldn't show the pixel commands of phase 7: `FILL`, `PENERASE` and `PENREVERSE`.

## Decision

- **The SVG is made from the drawing's marks,** everything since the last `CLEAN`, by `Drawing::svg` in `pendown-render`. The app and the CLI share it.
- **Each mark is one element,** as each is one stroke in the PNG, so translucent pens darken where moves meet in both:
  - a line is a `<line>`, with round caps and joins;
  - an arc is a path of SVG arcs, each at most half a turn, because one SVG arc can't go all the way round;
  - an ellipse is an `<ellipse>`, and a rectangle a `<rect>` with `rx` and `ry` for rounded corners, both rotated by their heading;
  - a fill comes before its outline, as SVG paints them.
- **It shows the whole turtle area,** whatever the canvas shows, with the background as a rectangle behind everything. Its `viewBox` is in turtle steps, one step to a pixel, with y flipped to point down, as SVG's does.
- **Pens keep their width in turtle steps.** The PNG never draws a line thinner than a pixel, but an SVG has no pixels, so a size-1 pen is one step wide however the SVG is scaled.
- **Choosing it:** the app's Export button has a menu, PNG picture or SVG drawing; a name ending `.png` or `.svg` overrides it. The CLI saves an SVG when the output name ends `.svg`, and ignores `--size`.
- **Tests:** every help example and tutorial program, and a few programs of the test's own, are exported, rendered back into pixels by resvg (which gpui already uses), and compared with the PNG. They're compared at an eighth of the size, where the two renderers' different smoothing of edges averages out, with pens widened as the PNG widens them. Drawing arcs the wrong way round, or turning shapes the wrong way, fails it.

## Consequences

- A big drawing makes a big file: the 131,070-line tree is about 131,000 elements. Joining moves with the same pen into one polyline would shrink it, but would change how translucent pens look where moves meet.
- When phase 7 adds `FILLED`, `LABEL`, the pen modes and `FILL`, each needs an SVG form. `FILLED` and `LABEL` map directly; `PENERASE` can be a mask and `PENREVERSE` `mix-blend-mode: difference`; `FILL`, a flood fill of pixels, has no shape to export, and needs a decision then: a traced outline, or the filled region as an image inside the SVG.
