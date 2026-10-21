# Arithmetic and logic

Status: done
Type: task
Blocked by: 01

## What

- **Arithmetic:**
  - `SUM`, `DIFFERENCE`, `MINUS`, `PRODUCT`, `QUOTIENT`, `REMAINDER`, `MODULO`, `INT`, `ROUND`, `SQRT`, `POWER`;
  - `SIN`, `COS` and `ARCTAN`, in degrees, including the two-input `(ARCTAN x y)`;
  - `RANDOM`.
- **Comparisons:** `LESSP`, `GREATERP`, `LESSEQUALP`, `GREATEREQUALP`, `EQUALP`, `NOTEQUALP`, and the infix forms. Numbers compare rounded to 15 significant digits (ADR 0002).
- **Logic:** `AND`, `OR` and `NOT`, with the variable-input forms `(AND a b c)`.
  - Their inputs must be `TRUE` or `FALSE`.
  - In UCBLogo they can also take instruction lists, evaluated only when needed. Check the manual, and decide whether that's in phase 1.
- **Numbers (ADR 0002):**
  - numbers are `f64`;
  - integer-only inputs (`REMAINDER`, `MODULO`, `RANDOM`) give an error for fractions;
  - results that aren't finite are errors;
  - numbers print as `%.15g`.
- **`RANDOM`:**
  - Each Run picks a new seed (ADR 0006).
  - The seed is settable for tests and the CLI, through `Limits` or a similar setting. A `--seed` CLI option is optional.
  - Check UCBLogo's `RANDOM` with two inputs (`(RANDOM start end)`), and match it if it's in the manual.

## Done when

`pendown-logo` has tests for each command's results and errors. They include `REMAINDER` and `MODULO` with negative numbers, matching UCBLogo's signs; trig at exact angles (`SIN 30` prints `0.5`); and `RANDOM` with a fixed seed.

## Comments

### 2026-10-21: done

- **The commands** are in `prims/arithmetic.rs`, along with the existing ones:
  - `REMAINDER` (the sign of the first input) and `MODULO` (the sign of the second), on whole numbers;
  - `INT` (towards zero), `ROUND` (halves away from zero), `SQRT` and `POWER`;
  - `SIN`, `COS` and `ARCTAN` in degrees, with `(ARCTAN x y)` from -180 to 180;
  - `(QUOTIENT x)`, the reciprocal;
  - `RANDOM n` and `(RANDOM start end)`;
  - `AND`, `OR` and `NOT`, with their variable-input forms.
- **Decided:**
  - `AND`, `OR` and `NOT` take instruction lists as in UCBLogo, run left to right only until the answer is known (`Interpreter::run_expression`). It was cheap: a list literal evaluates to itself, so it's never run unless needed.
  - `SIN` and `COS` are exact at multiples of 90, so `COS 90` prints `0`.
- **`RANDOM`:**
  - Its generator is in `src/random.rs`: a PCG32, unbiased, with no dependency.
  - Each Run gets a fresh seed (ADR 0006), and `Interpreter::set_seed` fixes it.
  - `pendown-cli` has a `--seed` option, and the plan's CLI section lists it. The picture tests use seed 1.
- **Tests:** `tests/arithmetic.rs` (7 tests: results, signs, trig, comparisons and logic, lazy lists, errors, and seeds); 2 unit tests of the generator; a CLI seed test; and a `random` picture.
