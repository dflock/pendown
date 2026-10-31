# Gallery

Easy access to the good example programs, to run or load into the editor: a **Gallery** tab in the side panel, after Help and Tutorial, with a thumbnail of each picture.

The examples already live in three places, and each is already tested:
- the help examples in `pendown-logo.kdl` (213, run by `crates/pendown/tests/examples.rs`);
- the lessons' programs in `tutorial/*.md` (about 32, run by the same test);
- the `.logo` files in `examples/` (run by `crates/pendown-cli/tests/pictures.rs`).

The gallery copies none of them. It's an index, `gallery.kdl`, that points at examples where they live, and puts them in sections and an order.

## What "done" looks like

- **A Gallery tab** in the side panel, after Help and Tutorial.
- **Sections** of cards, such as "Flowers and spirals", "Colour", "Recursion" and "Chance and waves", in the order `gallery.kdl` gives. Two cards a row in the panel.
- **Each card shows:**
  - a thumbnail of the picture, drawn in the background as the help panel draws an example's now, with the same cache;
  - its title;
  - where it comes from, such as "Help: REPEAT" or "Lesson 6", which opens that page;
  - **Run**, which plays it on the canvas as a help example does, with **Back to my drawing**;
  - **Load**, with "Load into editor" as its tooltip, which makes it a new, untitled program, as a lesson's program does now, asking first if the editor has unsaved changes.
- **Hand-picked: about 30–40 examples.** Most help examples are one to three lines showing one command (`FD 100`), and they'd bury the pictures. The gallery has the ones that draw something worth looking at.
- **A few new showcase programs,** written for the gallery, in `examples/gallery/`. They're the first section, so the tab opens on something inviting.
- **Every entry is checked:** a test fails if one points at an example that's gone, or whose title has changed.
- **The tutorial and README mention it.**

## Decisions

- **An index, not copies.** Code lives in one place, so a fix to a help example fixes its gallery card too. The index is one file, so the sections and their order are decided in one place, across the three sources. Tagging examples where they live (`gallery #true` in the kdl, a marker on a tutorial code block) would keep the choice next to the code, but couldn't order examples from different sources.
- **Help examples by command and title,** such as `help "repeat" "A flower of squares"`, not by number: adding an example to a command doesn't move the gallery's. Nearly all have titles already.
- **Lesson programs by lesson and name,** such as `lesson 6 "tree"`, with a `title`, since they have none of their own. The name goes after the fence's language in the lesson: ` ```logo tree `. The test checks the lesson has a program by that name. A lesson's pictures (` ```picture `) aren't programs, and buggy programs from the "Help Kame fix the bug" sections aren't for the gallery.
- **Files by path,** built into the app as the lessons are (`include_str!`), with a `title`.
- **Showcase programs are new code, not duplicates.** They go in `examples/gallery/`, and the pictures test runs them, as it does `examples/phase-1`.
- **A grid, two cards a row,** for bigger pictures. A card that narrow has no room for "Load into editor", so the button says Load, with the full words as its tooltip.
- **Load, not Copy.** Help pages offer "Copy to editor", which adds to your program. The gallery offers "Load into editor", which replaces it, since a gallery program is a whole program.
- **No grading.** No "easy" or "hard" labels, stars or difficulty order, as the tutorial has none (`mindstorms/mindstorm-implementation-plan.md`).
- **The thumbnails are the pictures the help panel would draw,** at the help panel's size, scaled down. They're drawn in the background the first time the tab opens, and cached.

## Not in this plan

- Saving your own programs into the gallery.
- Searching or filtering the gallery.
- Sharing, or examples from the internet.
- Every help example: the help pages already show them, in context.

## How the work is split

| Issue | What | Blocked by |
|---|---|---|
| 01 | `gallery.kdl`: the index, looking it up, and the test | |
| 02 | Run and Load from any source | 01 |
| 03 | The Gallery tab | 01, 02 |
| 04 | Showcase programs, and choosing the gallery | 01 |
| 05 | The tutorial, README and the scripted session | 03, 04 |
