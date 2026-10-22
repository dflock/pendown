# Counters with names and totals, and FOR's

Status: done
Type: task

## What

The interpreter's repeat counters record which command each is (`REPEAT`, `FOREVER`, `FOR`, `FOREACH`, `MAP` and so on) and, where it's known, how many passes there will be. `FOR` gets a counter of its own, which `REPCOUNT` and `#` skip.

## Done when

Tests show `REPCOUNT` and `#` unchanged, including inside a `FOR` inside a `REPEAT`, and `perf`'s interpreter cases are no slower.

## Comments

### 2026-10-22: done

`Interpreter::counters` replaces `repcounts`: each records its command, its pass, the number of passes if known, and whether `REPCOUNT` sees it. `counting` pushes one for a whole loop and `set_count` moves it on, so a pass costs no more than before. `FOR`'s counter has its passes worked out from its start, limit and step; `REPCOUNT` and `#` skip it, and the existing tests of both pass unchanged.
