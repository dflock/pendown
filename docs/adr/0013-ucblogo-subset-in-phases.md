# 13. Implement a subset of UCBLogo in phases, recorded in pendown-logo.kdl

Status: Accepted
Date: 2026-10-17 (recorded later, from commits e56b566, 8d5cc1f and a05ee3a)

## Context

Classic Logo assumes a full screen with no GUI or editor, so many of its commands exist to print procedure listings, manage a workspace, edit with an external editor, save workspaces, or deal with 1980s hardware. The goal here is a really good GUI for writing **new** Logo programs, not running the existing body of old ones.

## Decision

- **Implement a subset of UCBLogo,** in phases, starting with turtle and drawing commands and adding the more advanced parts later. The phases are described in `implementation-plan.md`: 1 turtle drawing, 2 the complete core language and filled shapes, 3 performance, 4 watching a program run, 5 help, examples and tutorial, 6 interactive programs, 7 advanced and niche, including fills, labels and pen modes.
- **Commands made unnecessary by the GUI, or tied to old hardware or the UCBLogo installation, are obsolete** and will never be supported: printing out definitions, workspace listings, `ERASE`, `EDIT`, `SAVE`/`LOAD`, `HELP`, `TRACE`/`STEP`, `PAUSE`, screen layout, console fonts, memory management, file paths, aspect-ratio correction, plus `TEST`/`IFTRUE`/`IFFALSE` and `GOTO`, which are superseded in the language.
- **`pendown-logo.kdl` is the source of truth** for what we support. Every command from the Berkeley Logo manual is in it, organised by chapter and section, with:
  - `obsolete` and `obsolete_reason`;
  - `phase`: a number, or `#null` for obsolete commands;
  - the manual's usage (`eg`), description (`desc`) and cross-references (`see`).
- **`scripts/manual_to_kdl.py`** generates `berkeley-logo.kdl` from `refs/logo/Berkeley-Logo-Reference-Manual.html`. `pendown-logo.kdl` started as a copy of that, and is now edited by hand. Never regenerate it, or the hand edits are lost.

## Consequences

- Obsolete commands can explain themselves: their `obsolete_reason` becomes the error message (ADR 0005).
- The help panel can be generated from the same file (ADR 0018).
- Our changes and extensions are recorded in the same file (ADR 0014).
