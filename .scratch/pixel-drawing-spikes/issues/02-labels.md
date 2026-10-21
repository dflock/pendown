# Spike: LABEL text in the pixel buffer

Status: needs-triage
Deferred: to phase 5, with the commands it tests
Type: prototype

## Question

Can `LABEL` draw text into the pixel buffer with vello_cpu, so it looks the same in the GUI and the CLI, is redrawn sharply at any size, and doesn't depend on gpui? The plan still says labels are drawn with gpui's `shape_line`, which was written before the pixel buffer. That would leave labels out of the CLI's pictures, out of PNG export, and on top of later drawing instead of under it.

The commands are `LABEL text`, which draws starting at the turtle's position in the pen colour, `SETLABELHEIGHT height` in turtle steps, and `LABELSIZE`, which outputs `[width height]`.

## Build

`spikes/06-labels`, depending on `pendown-render` by path:

- **vello_cpu's `text` feature** (glifo, on skrifa) draws glyph runs with `RenderContext::glyph_run` (`refs/vello/vello_cpu/src/render.rs:477`).
- **Shaping,** that is turning text into positioned glyphs, comes from another crate. Try harfrust, or parley if laying out lines turns out to be needed, against a plain lookup in skrifa's character map with its advances and no shaping.
- **A `Label { at, heading, text, height, pen }` mark,** drawn in order with the other marks, and redrawn on resize.
- **A test picture:**
  - labels at several heights, from 5 to 100 turtle steps;
  - one at each of several headings;
  - accented letters, a non-Latin script and an emoji;
  - a label over a line, and a line over a label;
  - a label filled around with `FILL`, if the other spike's fill is ready.

## Answer these

1. **Which font, bundled with the app?** A label must look the same on every machine and in the CLI, so the system's fonts won't do.
   - Propose two or three open-licence fonts suited to a playground for learners, with their licences and sizes.
   - How much does each add to the binary?
2. **Shaping:**
   - Is harfrust (or parley) worth its dependencies over the plain lookup?
   - What breaks without it: kerning, accents, non-Latin scripts?
   - How many crates does each bring?
3. **Emoji:** children will try them. Can vello_cpu draw a colour emoji font (COLR or bitmap)? If not, what's the fallback: an outline emoji font, or a box for a missing character?
4. **Orientation:** UCBLogo prints labels horizontally, but some Logos turn them with the turtle's heading. Render both, so the user can choose.
5. **Where the text sits:** UCBLogo starts the text at the turtle's position. Is that the baseline, or the bottom of the descenders? What does `SETLABELHEIGHT` measure: the font size, or the height including descenders, as the manual says? What do `LABELSIZE`'s width and height mean for a proportional font?
6. **Cost:**
   - How long does a label take to draw at 2560 × 1600?
   - What do 1,000 labels cost when redrawn on a resize?
   - Is shaping worth caching per label, or is glyph outline caching enough?
7. **The interpreter's side:**
   - `LABELSIZE` needs the font's measurements in `pendown-logo`, which has no drawing code (ADR 0009). Can the measurements come from the same font data, through a small crate or module both use, without pulling vello into the interpreter?
   - Which crates would `pendown-logo` then depend on?
8. **In the animator:** does a label just appear, and does it need an in-progress overlay at all?

## Done when

The questions are answered in a comment below, with measurements and PNGs, and ADR 0009 or a new ADR records the font and text decisions. The choice of font and orientation is listed for the user to decide.

## Comments
