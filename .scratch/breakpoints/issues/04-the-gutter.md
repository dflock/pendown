# The gutter: clicking sets and clears breakpoints, and the dots

Status: needs-triage
Type: task
Blocked by: 03

## What

gpui-kit's editor (`gpui-base` 0.8.0, `input/base/element.rs`) draws the line numbers, but has no event for a click on them and no way to draw a marker there. Its gutter width (`line_number_width`) is private. It does give the scroll offset (`scroll_offset()`), the line height, and range decorations (`RangeDecorationStyle::Fill`).

**Check gpui-kit first** (see issue 03). Gutter decorations draw coloured bars in a strip before the line numbers, with a width we can set (`gutter_decorations_width`). They don't take clicks: #3704 says so. gpui-kit's Diff component got clickable gutter markers the same week (#3703), so check whether the editor's gutter takes clicks by the time this starts. Then:
- **If a release has gutter decorations,** use them for the marks, with bars in place of dots and a strip wide enough to see. Only the clicks below are still ours to do.
- **If it also has gutter clicks,** use those too, and most of this issue goes.
- **Otherwise,** do it all ourselves, as below.

- **Clicks:** wrap the editor in a div that catches a left mouse-down in the gutter before the editor gets it.
  - The gutter's width is worked out from the number of digits in the last line number, the font and the editor's padding, in one function, so there's one place to fix if gpui-kit changes.
  - The line is worked out from the click's height, the scroll offset and the line height. Soft wrap is off, so one row is one line.
  - A click toggles that line's breakpoint if it can take one, and stops the event so the cursor doesn't move.
  - Clicks work while running, and while the editor is read-only.
- **Dots** (only without gutter decorations): an overlay over the gutter, positioned the same way, draws a red dot over the number of each breakpoint line, and follows scrolling.
- **The tint:** each breakpoint line gets a faint `Fill` range decoration, a separate collection from the highlight's, so the line shows when the gutter's scrolled off or the highlight's on it.
- **If this proves too fragile** (the width or rows drifting from what the editor draws), patch gpui-kit instead, as a `[patch.crates-io]` copy, with a gutter-click event and gutter markers. Offer the patch upstream. Say which in this issue's comments before doing it.

## Done when

- **Clicks:** in the scripted session, clicking line numbers sets and clears breakpoints:
  - on line 1;
  - on a line in double figures;
  - with the editor scrolled down;
  - while running.
- **Dots:** the screenshots show each dot over its number.
- **Lines without code:** clicking a blank line's number does nothing.
- **The cursor:** a click in the gutter doesn't move the cursor.

## Comments
