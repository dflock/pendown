# Spike: FILL, FILLED and the pen modes on the pixel buffer

Status: needs-triage
Deferred: to phase 7, with the commands it tests
Type: prototype

## Question

Can `FILL`, `FILLED`, `PENERASE` and `PENREVERSE` work on our anti-aliased pixel buffer? The results have to look right, and they have to stay right when the buffer is redrawn from the display list: on a resize, on `SETBACKGROUND`, and at the CLI's size rather than the window's (ADRs 0003 and 0010).

Spike 02 already answered part of this. An exact-match flood fill plus edge blending leaves no leaks and no visible halo, for every line of 1 px or more (`.scratch/phase-1-spikes/issues/02-pixel-buffer-streaming.md`, question 5). Its code, `spikes/02-pixel-buffer-streaming/src/bin/offline.rs`, is the starting point. What's new here is the real buffer, redrawing, and the pen modes.

## Build

`spikes/05-fill-and-pen-modes`, depending on `pendown-render` by path. It works mostly offline, writing PNGs and a log:

- **A `Fill { at, color }` mark**, using spike 02's fill on the buffer's bytes.
  - Drawing must be in order. Pending lines are drawn by vello_cpu before the fill reads the pixels, and the tiles the fill touches are marked dirty.
- **A `Filled { points, fill, pen }` mark**, drawn as a vello_cpu path fill, then its outline.
- **A pen mode on lines, arcs and shapes:** paint, erase and reverse.
- **Test programs,** written as lists of marks or, better, run through `pendown-logo` with stub commands:
  - a square filled from inside;
  - nested shapes filled one ring at a time;
  - a fill from the turtle sitting on a line;
  - a star from `REPEAT 5 [FD 100 RT 144]`, filled and `FILLED`;
  - erase across a filled shape;
  - reverse drawn twice over the same path;
  - reverse along a polyline of many short moves.

## Answer these

1. **The fill's seed when redrawn:** `FILL` takes its colour from the pixel under the turtle.
   - When the buffer is redrawn at another size, can that pixel fall on a line at one size and not at another, filling the line at one size and the background at the other? That's common, since the turtle often sits on the line it just drew.
   - If so, what fixes it? For example, decide the seed colour once, when the program runs, and keep it in the mark, or snap the seed off nearby line pixels.
   - The CLI and the GUI must agree.
2. **Redrawing cost:**
   - How long does a full-screen fill take at 2560 × 1600?
   - What does a program with 1,000 small fills cost, both when it first runs and when it's redrawn on a resize or `SETBACKGROUND`?
   - Each fill forces vello_cpu to draw what's pending first. Does that flush cost much?
3. **Edge blending on the real buffer:**
   - Does spike 02's result hold for every kind of mark: lines with round caps, arcs, ellipses, rectangles, and translucent pens?
   - What happens filling a region whose edge is a line from an earlier fill?
4. **`FILLED` and self-crossing shapes:** for the star, compare the nonzero fill rule (all filled) and even-odd (a hollow pentagon in the middle) with what UCBLogo's wxWidgets version does, if that can be found out. Recommend one.
5. **`PENERASE` as painting in the background colour:**
   - ADR 0010 says this breaks if the background changes later. But `set_background` now redraws everything on the new colour, so erasing with whatever the background is at redraw time may be right after all. Check it, including erase over a fill.
   - Does an exact erase of an anti-aliased line leave a faint ghost at the edges? If so, does a slightly wider erase pen fix it?
6. **`PENREVERSE`:**
   - Is vello_cpu's `Mix::Difference` with white, or another blend mode, a good inversion?
   - Does drawing a line twice restore the picture, given that anti-aliased edges are only partly inverted? How big is the ghost?
   - Where two moves meet, the joint is covered twice and so un-inverted: how bad is it? Does drawing a run of consecutive reverse moves as one path fix that, and what does it cost?
   - What do translucent pen colours do in reverse?
7. **The animated move in reverse and erase:**
   - While a move is in progress, the GUI draws it as a gpui path over the buffer (`Overlay::partial`). Erase can draw it in the background colour, but gpui can't invert, so a partial reverse line can't be drawn that way.
   - Options include drawing the partial line into a scratch copy of the affected tiles each frame, or showing the partial line in a plain colour until it finishes. Try one, and check it in the GUI.
8. **`FILL` in the animator:** does a fill just appear when the animator reaches it? Is it fast enough to do on the UI thread within a frame, or does a large fill need to run elsewhere?

## Done when

The questions are answered in a comment below, with measurements and PNGs, and ADRs 0003 and 0010 are updated if anything changes. Open choices about how things look, such as the fill rule, are listed for the user to decide.

## Comments
