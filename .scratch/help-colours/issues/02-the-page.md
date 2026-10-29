# The Colours page

Status: done
Type: task

## What

- A `colours` syntax entry in `pendown-logo.kdl`: title "Colours", `names "COLOURS" "COLORS" "COLOUR" "COLOR"`, `help` with what a colour is for and the forms, and `example`s.
- In the help panel, for that entry: the forms table with a swatch beside each, a "Palette" section of 16 swatches with their numbers and the `SETPALETTE` note, and a "Names" section of the 148 names as swatches, in colour order, with a filter box that matches anywhere in a name. A swatch is a button; issue 04 makes clicking it set the current colour.
- `[colours]` links in the help of `SETPENCOLOR`, `SETBACKGROUND`, `SETFILLCOLOR`, `SETPALETTE`, `PALETTE`, `PENCOLOR`, `BACKGROUND`, `FILLCOLOR` and the quoted-word syntax page, where they talk about colour forms.

## Done when

The page shows as in the mockups, search finds it by `colour` and `color`, the links work (the markup test checks every link), its examples pass the examples test, and screenshots show it.

## Comments

### 2026-10-29: done

The `colours` syntax entry is in `pendown-logo.kdl` after "Quoted words", with an example (its picture is `crates/pendown/tests/help-pictures/colours-1.svg`). The colour commands and the quoted-word page link to `[Colours]`, and the commands have Colours first in See also. `colours_page.rs` draws the forms table (with short hex as well), the palette and the names with their filter and count; `help_panel.rs` puts them after the page's help. Search finds it by `color` first. A syntax entry's badges are now only its symbol names, so Colours shows none.
