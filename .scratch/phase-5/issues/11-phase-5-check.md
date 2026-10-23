# Phase 5 check

Status: done
Type: task
Blocked by: all others

## What

README, the plan, an ADR for the decisions, and the issues closed.

## Done when

Everything passes, and the docs say what was built.

## Comments

### 2026-10-23: done

296 tests pass and clippy is clean. The scripted session passes all 70 checks, and the grammar its 7 parse tests and 16 highlight assertions. The README describes the panel, the tutorial and `--fit`; the plan marks phase 5 done and lists where it differs from the plan; ADR 0023 records the decisions.

`perf` found 5,000 lines in lockstep 25% slower than its baseline. It wasn't phase 5: main had it too, from the colours added to the variables panel after phase 4's own check, which formatted three colours for every instruction. They're now formatted only when they change, and lockstep is 4% over the baseline.
