# 18. Help, examples and tutorial come from pendown-logo.kdl, and examples are tests

Status: Accepted
Date: 2026-10-18 (recorded later, from commit 1ac2446)

## Context

Two of the headline features are extensive help, with a reference panel, and a built-in tutorial with example code for each command. `pendown-logo.kdl` already has every command with its usage and the manual's description (ADR 0013). But the manual's descriptions describe UCBLogo, not our dialect: they mention wxWidgets, percentage RGB values, `SETSCRUNCH` and so on.

## Decision

These are phase 5, after phase 2, so they cover the complete core language once its commands have settled.

- **The help panel is generated from `pendown-logo.kdl`,** embedded in the GUI at build time and parsed with the `kdl` crate. It's browsable by chapter and section, searchable by name and alias, and shows help for the command under the editor's cursor.
- **Each supported command gets a `help` node** with our own description, written for our dialect with the `differs` notes folded in. `desc` stays as the manual's text, for whoever implements the command.
- **Each supported command gets one or more `example` nodes:** short runnable programs, separate from `eg`, which only shows how to call the command. The help panel offers Run and Copy to editor for each.
- **Examples are tests:** a test runs every example through the interpreter and the CLI renderer and compares the result with a saved PNG. Examples that use `RANDOM` use a fixed seed.
- **The tutorial** is a sequence of lessons with text and code that loads into the editor, following the phase 1 commands.

## Consequences

- Writing `help` text and examples is real work: about 165 commands in phases 1 and 2.
- The documentation and the interpreter can't drift apart without a test failing.
- Open questions in `implementation-plan.md`: the markup format, the tutorial file format, where examples run, and whether to show pictures of example output.

## Note

2026-11-01: `pendown-logo.kdl` moved to `content/reference.kdl`, beside `content/tutorial/` and `content/gallery.kdl`, so everything the panel shows is in one folder. `berkeley-logo.kdl`, the manual converted to KDL, which `reference.kdl` started from, moved to `content/ucblogo/`, with the manual it's made from, which was only in the ignored `refs/` before.
