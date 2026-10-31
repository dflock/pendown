# `gallery.kdl`: the index, looking it up, and the test

Status: resolved
Type: task

## What

A file at the repository root, `gallery.kdl`, beside `pendown-logo.kdl`, that lists the gallery's sections and entries. Each entry points at an example where it lives:

```kdl
section "Flowers and spirals" {
  help "repeat" "A flower of squares"
  help "repcount" "A spiral"
  lesson 6 "tree" title="A tree"
  file "examples/gallery/garden.logo" title="A garden at night"
}
```

- `help "<command key>" "<example title>"`: a help example, by its command's key in `pendown-logo.kdl` and its title.
- `lesson <n> "<name>" title="..."`: a lesson's program, by the name after its fence's language (` ```logo tree `).
- `file "<path>" title="..."`: a `.logo` file, by its path from the repository root.

In `pendown` (a `gallery` module, beside `tutorial.rs`):
- The file is built in with `include_str!`, and parsed with the `kdl` crate, as `reference.rs` parses `pendown-logo.kdl`.
- `.logo` files it names are built in too, with `include_str!`, by a `build.rs` that reads `gallery.kdl` or by a list the test checks against it. The app reads no gallery file at run time: like the help and the lessons, it's all in the binary.
- `gallery()` gives the sections, each with its entries: a title, where it comes from (a `Source`), and its code.
- `Source` is the help panel's (`help_panel.rs`), moved to the library and given a `File` variant, so the gallery, the banner over the canvas and Load share it.

## Done when

- A test resolves every entry: each help entry names an implemented command with an example of that title, which isn't one that fails on purpose; each lesson entry is in range and is a program, not a picture; each file exists and is built in.
- A test checks no example is in the gallery twice.
- A broken entry fails with a message saying which entry and why.
- The gallery works from the binary alone, run from a folder without the repository.

## Comments

2026-10-31: Done. `pendown::gallery` parses `gallery.kdl` (built in), with `gallery::FILES` holding the ten programs by `include_str!`; a test checks that list against `examples/gallery/` and that each is used. `Source` moved to `pendown::source`, with `File(usize)`. Five unit tests: every entry resolves, none is in twice, the files, a broken entry says why, and the labels.

2026-10-31: Lesson programs are now by name, not number: `lesson 6 "tree"`, from ` ```logo tree ` in the lesson (the markup reads a name after a fence's language). Adding a program to a lesson no longer moves an entry.
