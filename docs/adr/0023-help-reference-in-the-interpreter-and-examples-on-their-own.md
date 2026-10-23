# 23. The help reference lives in the interpreter, and examples run on their own

Status: Accepted
Date: 2026-10-23

## Context

Phase 5 adds a help panel generated from `pendown-logo.kdl` (ADR 0018), a tutorial, and errors for obsolete commands that say why they aren't here. The interpreter needs the obsolete commands' reasons, and the GUI needs everything else: names, usage, our descriptions, examples and the chapters.

Each example has Run, which plays it on the main canvas with the usual animation, highlight, Step and panels. The user may be in the middle of their own program, with procedures and variables the console can use (ADR 0006).

The plan asked for pictures of each example's output, made from the PNGs the example tests save.

## Decision

- **The reference is `pendown_logo::reference`.** It embeds the kdl and parses it with the `kdl` crate the first time it's needed. The interpreter uses it for "EDIT isn't in Pendown: …", and the GUI for the panel, so the reasons have one source. `kdl` is now a normal dependency of `pendown-logo`.
- **A command is implemented when the interpreter has its kdl key,** not any name in its usage: `USUAL`'s usage shows `TO` and `END`.
- **Syntax entries** are a `syntax` node beside `logo` in the kdl, found by their `names` (`:`, `[`, `?REST`) and by key, as links name them (`[parentheses]`).
- **Help text is a small markup,** `pendown::markup`: paragraphs, code, bold, italics, lists, notes, headings, fenced code and `[NAME]` links. The tutorial is written in it too, as markdown files in `tutorial/`.
- **Pictures are drawn when shown,** on a background task with `pendown-cli`'s renderer, fitted to the drawing and with `RANDOM`'s seed fixed, and kept for the session. The examples test draws them the same way and compares them with the saved PNGs, so the panel shows what the test checked without embedding 200 pictures in the app.
- **An example runs on an interpreter of its own** (`run::start`), so the user's procedures and variables are untouched. The window sets aside the user's animator, run, text, drawing and variables, and puts them back with **Back to my drawing**. The example's picture stays when it ends, until then or until Run; the plan had the user's drawing come back as soon as it ended, which would hide what the example drew. A console entry puts the user's things back first, since it works on what their program left.
- **Errors carry the command they're about,** `Error::command`. The console is a text area, so instead of links in its lines, its title bar has **Help on X** for the last error.
- **Following the cursor** watches the editor, which notifies when its cursor moves, and finds the page with the highlighter's tokens: a variable shows Variables, a number Numbers, and the program's own procedures nothing.

## Consequences

- The kdl is parsed once per process, the first time anything asks: in the CLI only if a program uses an unknown name.
- An example's picture takes a moment the first time its page shows; the panel shows a blank square until it's drawn.
- Every example, and every program in the tutorial, is a test: it must run (or fail, if it says `fails=#true`), print what's saved, and draw the saved picture. Changing an example means looking at its new picture and saving it with `UPDATE_PICTURES=1`.
- Running an example starts an interpreter thread for it, with the usual large stack, which ends when the example does.
