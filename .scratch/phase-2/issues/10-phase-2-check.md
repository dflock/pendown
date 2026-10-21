# Phase 2 check

Status: done
Type: task
Blocked by: 01, 02, 03, 04, 05, 06, 07, 08, 09

## What

- **The kdl test** checks that the interpreter implements exactly the phase 1 and 2 commands, and the special variables.
- **Examples** in `examples/phase-2/` use every phase 2 command between them, and are picture tests.
- **The plan and ADRs** are up to date, and the GUI's scripted session passes.

## Done when

Each command has tests of its behaviour and its errors in `pendown-logo`, or the GUI and CLI tests cover it.

## Comments

### 2026-10-21: done

The kdl test checks phases 1 and 2 and the special variables. `examples/phase-2/` has four programs that use every phase 2 command, and they're picture tests (`phase-2-*.png` and `.txt`). The scripted session has 37 checks, including filled shapes and `CATCH`. ADR 0021 records the decisions, and `pendown-logo.kdl` has `differs` notes for them.
