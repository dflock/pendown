# 30. The gallery is an index, not copies

Status: Accepted
Date: 2026-10-31

## Context

Learners want easy access to example programs to run or load. The good ones already live in three places, each tested: the help examples in `pendown-logo.kdl`, the lessons' programs in `tutorial/`, and `.logo` files. Copying them into a gallery would mean two copies of each to keep the same.

## Decision

- **A Gallery tab** in the side panel, after Help and Tutorial, with sections of cards: a picture, a title, where it's from, **Load** and **Run**.
- **`gallery.kdl` is an index.** Each entry points at an example where it lives: `help "<command>" "<example title>"`, `lesson <n> "<name>" title="..."`, or `file "<path>" title="..."`. One file decides what's in the gallery, its sections and its order, across the three sources. Tagging examples where they live would keep the choice beside the code, but couldn't put examples from different places in one order.
- **Help examples by title, not number,** so adding an example to a command doesn't move the gallery's.
- **Lesson programs by name, not number.** They have no titles, so a program the gallery uses is named after its fence's language: ` ```logo tree `. Readers never see the name, and Markdown viewers read only the language, so the lesson still shows as Logo on GitHub. A comment in the code was the other choice, but readers would see it, and Load would copy it into their program.
- **Programs written for the gallery** are new code, in `examples/gallery/`, run by `pendown-cli`'s pictures test.
- **Everything is built in.** `gallery.kdl` and the gallery's programs are compiled into the app, as the help and lessons are. The app reads no gallery file at run time.
- **A test checks every entry,** and fails with which and why if one points at nothing: an example renamed or removed, a lesson program's name gone, or a file not built in. At run time, an entry that points at nothing is left out.
- **Load, not Copy.** A gallery program is a whole program, so Load makes it a new, untitled program, as a lesson's Load does, asking first about unsaved changes. Help's Copy to editor adds to yours.
- **No grading:** no "easy" or "hard" labels or stars (ADR 0027).
- **The pictures are drawn as the help panel's are,** in the background, fitted to the drawing, and cached, the first time the tab opens.

## Consequences

- Changing a help example or a lesson's program changes its gallery card too.
- Renaming or removing a help example the gallery uses breaks the test until `gallery.kdl` is updated.
- Adding, removing or reordering a lesson's programs doesn't move the gallery's entries. Renaming or removing a named program breaks the test until `gallery.kdl` is updated, and a name after a lesson's fence says the gallery uses that program.
- `Source`, which names a program by where it lives, moved from the help panel to the library, with a `File` kind, and Load takes any `Source`.
