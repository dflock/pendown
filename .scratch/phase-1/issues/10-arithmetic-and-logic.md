# Arithmetic and logic

Status: ready-for-agent
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
