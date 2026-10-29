# Help: a Colours page, with a colour picker

A Help page about colours: every form a colour can take (ADR 0015), the 16 palette colours and the 148 colour names as swatches, and a colour picker with a wheel. Any colour on the page, picked or clicked, goes into the editor at the cursor, in the form it was chosen in. Every colour command's help links to the page.

There are no new Logo commands. The UI design is on a canvas of mockups: https://claude.ai/artifact/RRPY9vUABvy4hU4XiTdiSu (private to the owner unless shared).

(Help search matching anywhere in a name, asked for at the same time, is done already, outside this plan.)

## What "done" looks like

- **The page:** a syntax page, **Colours**, in Contents with the other syntax pages, found by searching `colour` or `color`. From the top:
  - what a colour is for, and a table of the forms: a palette number `4`, a name `"tomato`, RGB `[255 99 71]`, RGBA `[255 99 71 128]`, hex `"#ff6347`, short hex `"#f64`, and hex with opacity `"#ff634780`, with a swatch beside each;
  - **Find a colour:** the picker (below);
  - **Palette:** the 16 palette colours, 0 to 15, as swatches with their numbers, and a note that `SETPALETTE` can change 8 to 15;
  - **Names:** the 148 names as swatches with their names, in colour order (greys first, then by hue), with a filter box that matches anywhere in a name, and says how many match;
  - examples and See also, as other pages have.
- **The picker:**
  - a wheel of hue and saturation, with a marker where the colour is; drag or click on it;
  - a brightness bar and an opacity bar under it, each showing its range for the current colour; drag or click on them;
  - fields for R, G and B (0 to 255), A (0 to 255) and hex, which follow the wheel and bars, and move them when typed in. A field that doesn't make sense is outlined, with the reason under it, and the colour doesn't change;
  - it starts at Kame's pen colour.
- **The current colour bar,** at the bottom of the page and always in view while it's open, headed **Current colour**: a swatch of the colour (over a checkerboard, so you can see how see-through it is), what will go in the editor, the form, and **Copy to editor**.
  - The form is a choice of RGB, RGBA and Hex, plus Name and Palette when the colour is exactly one of those.
  - The picker sets the colour in the form last chosen for it, Hex at first. Clicking a palette swatch sets it as Palette, `4`; clicking a name sets it as Name, `"tomato`.
- **Copy to editor** puts the colour at the editor's cursor, replacing any selection, with a space before it if the character before the cursor isn't a space, a `[` or the start of a line. The editor's Undo takes it out. The console says nothing.
- **Links:** every colour command's help links to the page where it talks about colour forms, with `[colours]`: `SETPENCOLOR`, `SETBACKGROUND`, `SETFILLCOLOR`, `SETPALETTE`, `PALETTE`, `PENCOLOR`, `BACKGROUND`, `FILLCOLOR`, and the quoted-word syntax page. On the commands' pages, Colours comes first in See also.

## Decisions

- **The page is a syntax entry, `colours`, in the kdl,** with `help`, `example` and `names "COLOURS" "COLORS" "COLOUR" "COLOR"`, so search finds it either way and `[colours]` links to it. Its swatches, picker and bar are drawn by the help panel when it shows that entry; the kdl holds only the words.
- **The picker is our own,** not gpui-kit's `ColorPicker`, which is a dropdown of preset swatches with sliders, with no wheel.
- **The colour is kept as hue, saturation, brightness and opacity** while picking, so dragging the brightness to black and back keeps the hue. The fields show it as RGB 0–255 and hex, and typing in them sets it from those.
- **The wheel is drawn as a picture,** made once for each brightness and kept, as Help's example pictures are, with the marker drawn over it. Moving the brightness bar redraws it.
- **The arithmetic is in a module without a window,** `pendown::colours`, with unit tests: HSV to RGB and back, the wheel's position for a colour and the colour at a position, the forms as text, and which name or palette number a colour is exactly, if any.
- **The names are sorted by colour,** greys by lightness first, then by hue. `gray` and `grey`, and the other spellings that are the same colour, are both shown, side by side.
- **The palette swatches show the default palette,** not what `SETPALETTE` has changed it to, since Run puts the palette back.
- **The text that goes in the editor is a Logo value,** as `SETPC` takes it: `4`, `"tomato`, `[255 99 71]`, `[255 99 71 128]`, `"#ff6347`, `"#ff634780`. Hex is lower case, and has two more digits only when it isn't solid.
- **Copy to editor works while a program runs?** No: the editor is read-only while running (ADR 0006), so the button is disabled then, with the tooltip "Stop the program to change it".
- **The Colours page doesn't follow the editor's cursor,** as other pages do, and has no "Following the cursor" strip. Placing the cursor to copy a colour there would otherwise take the page away. F1 still opens help for the word under the cursor.
- **A syntax entry's badges are only its symbol names,** such as `:`. Colours' names, `COLOURS`, `COLORS`, `COLOUR` and `COLOR`, are only for search.
- **Later, not now:** Help following the cursor onto a colour in the editor (`"tomato`, `"#ff6347`) and opening the picker at it.

## How the work is split

| Issue | What | Blocked by |
|---|---|---|
| 01 | `pendown::colours`: HSV, RGB and hex, the wheel's geometry, the forms as text, exact names and palette numbers | |
| 02 | The page: the kdl entry and its words, the forms table, palette and names swatches with the filter, and the links from the colour commands | |
| 03 | The picker: the wheel, the bars, the fields | 01, 02 |
| 04 | The current colour bar and Copy to editor, and session checks and screenshots | 03 |
