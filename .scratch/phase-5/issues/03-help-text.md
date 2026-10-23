# Markup, and help and examples for every command

Status: done
Type: task
Blocked by: 01

## What

`pendown::markup`: paragraphs, code, bold, italics, lists, fenced code and `[NAME]` links. A `help` node and at least one `example` for each of the 166 phase 1 and 2 commands, and the syntax entries. Written for our dialect, mentioning Kame, with the `differs` notes folded in.

## Done when

Tests: every implemented command and syntax entry has help and an example, the markup parses, and every link resolves.

## Comments

### 2026-10-23: done

Every one of the 166 implemented commands and the 9 syntax entries has a `help` node and at least one `example`: 175 pages and 213 examples, with `see` lists where the rest of the section isn't enough. They're written for our dialect: Kame for the turtle, the colour forms, ARC's walk, WAIT and SETSPEED in the GUI, `TRUE` and `FALSE`, and the `differs` notes folded in.

`pendown::markup` parses the markup, and `crates/pendown/tests/help.rs` checks every link resolves. A link can name a syntax entry by its key, as `[parentheses]` does. `ERRACT`'s example stops with an error on purpose, as it must to show anything, so examples can say `fails=#true`.

Writing them turned up two things to know: Pendown doesn't support `|...|` in words yet, so an example can't TYPE a space; and a number prints as it was typed until it's worked out, so `PRINT 1e3` prints `1e3`.
