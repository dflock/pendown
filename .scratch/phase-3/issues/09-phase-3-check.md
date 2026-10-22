# Phase 3 check

Status: done
Type: task
Blocked by: 01, 02, 03, 04, 05, 06, 07, 08

## What

- Every test, clippy, the scripted GUI session and `perf` pass.
- The README says how to read the metrics, use the overlay and run `perf`.
- The plan marks phase 3 done, and ADR 0020 is updated with what was learned.

## Comments

### 2026-10-22: done

- 245 tests pass and clippy is clean.
- The scripted session passes all 41 checks, in release and in a debug build. The debug build printed a summary for each of its 21 runs.
- `perf` passes against the baseline, every case within 6%.
- The README has a Performance section: the summary, F12, budgets and `perf`.
- The plan marks phase 3 done, and ADR 0020 has a section on what phase 3 built and learned.
- Fixed on the way: a summary of a run with no renders showed the UI's whole CPU time as the cost of a frame.
