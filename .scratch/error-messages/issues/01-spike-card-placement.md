# Spike: placing a card over the editor

Status: resolved
Type: prototype

## What

The card has to sit exactly under a line of code and stay there through scrolling and resizing. `range_to_bounds` on the editor's state looks like the way to find the line, but it hasn't been tried, and it returns nothing for text that isn't laid out.

Throwaway code: a fixed box over the editor under a chosen line, in the `h_flex` that holds the gutter and `editor_pane` (`app.rs`, about line 1786). `editor_pane` is a cached view, so the box has to be a sibling over it, and it has to redraw when the editor scrolls, as `gutter.rs` does.

Answer these, on the private Xvfb display:

- Is `range_to_bounds` callable from Pendown, and in what coordinates? `Gutter` works from `line_height`, `scroll_offset` and `TOP` instead, which may be enough with soft wrap and folding off.
- Does the box stay on its line when scrolling with the wheel and the scrollbar, typing, and dragging the panes? Any lag of a frame or more?
- What does it do when its line scrolls out of view?
- With `.occlude()` on the box, do clicks stay out of the code under it, and does the wheel still scroll the editor? The gutter forwards the wheel with `on_scroll`.
- Does a box with no handlers let clicks through to the editor? The message at the end of a line needs this.

## Done when

- The questions are answered in the comments below, with screenshots checked.
- If placement can't work well, the spec is changed with the maintainer before issue 06 starts.
- The spike's code is not kept.

## Comments

2026-11-04: Placement works, but not with `range_to_bounds`. The spike's code is gone. Tested on X11 only, in Xvfb with xfwm4 at 1x scale, with a 60-line program, logging what each frame rendered and what the editor then painted.

**Use this:**

- **y:** `8 + scroll.y + line_height * line`, as `gutter.rs` does. It was right in every section of the test.
- **x of the end of a line:** `12 + scroll.x + width`, with `width` from `window.text_system().shape_line` in the theme's mono font and size. The 12 is the editor's padding, like the gutter's `TOP`, read from `range_to_bounds` at scroll 0. The marker I checked used 13, and sat 1 px past the end of the text at 8 and 112 characters, scrolled sideways, and with a `ü`. So 12 fits, but hasn't been looked at itself. Wide characters, such as CJK from a fallback font, weren't tried.
- **The parent:** a `relative` wrapper with `overflow_hidden` around `editor_pane` and the overlay, keeping `.cached(size_full)`. It sets the coordinates, and clips the card when its line scrolls out of view, at either edge.
- **Redrawing:** `cx.observe(&editor, ...)`, as the gutter does.

**Why not `range_to_bounds`:**

- For a line scrolled above the view, it doesn't return nothing, whatever its docs say. It returns the first visible line, as though the line had stuck to the top of the pane (8 px, with line 6 of 60 far out of view).
- It reads the last layout, so it's a frame behind while scrolling (69 of 629 frames in the first run, against 33 for the sum above, nearly all from typing). Its width was also 0 after typing a long line, until something else redrew it, about half a second later by the look of the log, though I didn't time it.
- `visible_row_range` also lags a frame, so it shouldn't decide whether the card shows.

**The answers:**

- **Stays on its line?** With the wheel, the scrollbar and pane drags, the sum was never behind. When the editor scrolls itself, as Ctrl+Home, Ctrl+End, Page Down and typing past the edge do, the card is one frame behind, then right on the next. That's about 16 ms and I couldn't see it in screenshots, so I don't think it matters. Check by eye in issue 06.
- **Line out of view?** The wrapper clips it, with nothing special needed.
- **`.occlude()`?** A click on the box hit its handler and left the cursor where it was, and the keyboard still worked after. The box forwarded the wheel with its own handler calling `set_scroll_offset`, which worked (two clicks moved it 120 px). I didn't test it without that handler, so it may not be needed.
- **A box with no handlers?** Clicks go through to the editor, and the cursor moved to that line.

**For issue 06:**

- The card needs clamping to the pane: at a narrow pane it was cut off on the right.
- Text needs `whitespace_nowrap`, as a box near the right edge wrapped into a tall thin column.
- At the end of a long line, the message landed off the pane's right edge, so the "no room, move it into the card" rule is needed.
