# 2. Numbers are `f64`, and numbers are words

Status: Accepted
Date: 2026-10-18. Updated 2026-10-21 after spike 03.

## Context

UCBLogo has no separate number type in the language: a number is a word that looks like a number, so `"3` and `3` are the same thing, and `FIRST 345` is `3`. Internally it uses integers and floats. We need one rule for storage, arithmetic and printing.

## Decision

- **Storage:** every number is an `f64`. There's no separate integer type.
- **Numbers are words:** a word that looks like a number can be used as one, and a number can be used as a word. `FIRST 345` is `3`, `COUNT 345` is `3`, and `WORD 1 2` is `12`, which is also a number.
- **Printing** follows C's `%.15g`, as UCBLogo does:
  - Whole numbers print without a decimal point, so `PRINT 4/2` shows `2`.
  - Other numbers print with up to 15 significant digits and no trailing zeros, so `PRINT 3/2` shows `1.5` and `PRINT 0.1 + 0.2` shows `0.3`, not `0.30000000000000004`.
  - Numbers from 1e15 up, and below 1e-4, use exponent notation: 2^53 prints as `9.00719925474099e+15`, and 0.00001 as `1e-05`.
  - `-0` prints as `0`.
- **Equality:** `EQUALP` and `=` compare numbers numerically, so `2 = 2.0` is true.
  - **Numbers are compared rounded to 15 significant digits,** so two numbers are equal exactly when they print the same. `0.1 + 0.2 = 0.3` is true, which UCBLogo gets wrong: what you see is what's equal.
  - `<`, `>`, `<=`, `>=` and `<>` round the same way, so they always agree with `=`.
  - Whole numbers below 2^53 are compared exactly, which is also the fast path.
- **Integer-only commands** (`REMAINDER`, `MODULO`, `ITEM`, `REPEAT`, `RANDOM`, palette numbers…) accept a number with no fractional part and give an error for anything else.
- **Errors:** dividing by zero, and any result that isn't finite, is an error rather than producing infinity or NaN.

## Consequences

- Integers are exact up to 2^53, which is plenty for drawing programs.
- A value may be held as a number or as a word internally. Converting between them must give the same printed form, so `WORD 1.50 "` is the word `1.50` as typed, but `1.50 + 0` prints `1.5`. A number literal typed in its printed form (`10`, `1.5`) is stored as a number when it's parsed; others (`1.50`, `1e3`) stay words.
- Comparing numbers that aren't whole means rounding them through formatting, which is slower than a plain comparison. Spike 03's loops, which compare whole numbers, ran no slower.
- Two numbers that differ only after the 15th significant digit are equal, even though subtracting them doesn't give 0.
