# Breakpoints follow their lines through edits

Status: resolved
Type: task

## What

A pure function, with no GUI, that takes the old text, the new text and the breakpoints' line numbers, and gives the new line numbers:
- It compares the lines, finding the lines the two texts share before and after the change.
- A breakpoint on a line that's still there moves with it.
- A breakpoint on a line that was deleted or changed goes.
- An edit within a line keeps that line's breakpoint.
- Lines added above a breakpoint move it down.

It also says which lines can take a breakpoint: lines with code, from the tokens, so it works while the program doesn't parse. Blank lines, comment lines and `END` can't. A breakpoint left on a line that no longer has code goes.

`App` calls it on each editor change and keeps the result.

**Check gpui-kit first.** On gpui-kit's main, after 0.8.1, the editor has gutter decorations (`create_gutter_decorations_collection`, #3704). Their ranges follow edits, so they could keep the breakpoints on their lines for us, read back with `get_ranges`, instead of comparing texts. An edit that empties a range leaves it as a boundary, which would count as a deleted line. If a release has them when this starts, upgrade to it (on a branch of its own, as for 0.8.1) and use them. This function then only says which lines can take a breakpoint. Before a release has them, a git dependency pinned to a commit works:

```toml
gpui-kit = { git = "https://github.com/longbridge/gpui-kit", rev = "94480398", features = ["tree-sitter"] }
```

Pendown built against 94480398 with no changes beyond 0.8.1's, and its tests passed (2026-10-30).

## Done when

Unit tests cover:
- typing on a breakpoint's line;
- adding lines above and below one;
- deleting its line;
- pasting several lines;
- turning a line into a comment;
- undo bringing back a deleted line without its breakpoint;
- a text that doesn't parse.

## Comments

2026-10-31: Done, without gpui-kit (no release has the gutter decorations): `pendown::breakpoints::{can_take, follow, stops}`, with unit tests. The gutter calls `follow` on every change, and `check_program` and Run call it too, since `set_value` sends no change event.
