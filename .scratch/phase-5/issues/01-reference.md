# The reference model

Status: done
Type: task

## What

`pendown_logo::reference`: embeds `pendown-logo.kdl`, parses it once with `kdl`, and gives chapters, sections and commands, with names, usage (`eg`), `help`, examples, `see`, phase, whether it's an extension, and obsolete reasons. Syntax entries from a top-level `syntax` node. Lookup by any name or alias, case-insensitively, and search by prefix. `kdl` becomes a normal dependency.

## Done when

Tests cover lookup by every alias, search, the implemented commands, and obsolete ones.

## Comments

### 2026-10-23: done

`pendown_logo::reference` parses the embedded kdl once (`OnceLock`) into chapters, sections and commands. A command is implemented if the interpreter has its kdl key, not any name in its usage: `USUAL`'s usage shows `TO` and `END`. Syntax entries are a `syntax` node beside `logo`, found only by their `names`, so `+` still finds `SUM`. `REPCOUNT` gets `names "#"` and `TO` gets `names "END"`. `kdl` is now a normal dependency of `pendown-logo`.
