# Arithmetic

Status: done
Type: task

## What

- `EXP`, `LOG10`, `LN`, `RADSIN`, `RADCOS`, `RADARCTAN`, `ISEQ`, `RSEQ`, `RERANDOM` and `FORM`.
- Errors for logarithms of zero or less, and for `FORM`'s printf format, which isn't supported.

## Done when

Each command has tests of its behaviour and its errors in `pendown-logo`, or the GUI and CLI tests cover it.

## Comments

### 2026-10-21: done

In `prims/arithmetic.rs`. `ISEQ`, `RSEQ`, `ARRAY` and `MDARRAY` refuse to make more than 10 million members. Tests: in `tests/arithmetic.rs`.
