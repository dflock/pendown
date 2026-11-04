# Spike: placing a card over the editor

Status: ready-for-agent
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
