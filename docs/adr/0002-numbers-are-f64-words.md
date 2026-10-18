# 2. Numbers are `f64`, and numbers are words

Status: Accepted
Date: 2026-10-18

## Context

UCBLogo has no separate number type in the language: a number is a word that looks like a number, so `"3` and `3` are the same thing, and `FIRST 345` is `3`. Internally it uses integers and floats. We need one rule for storage, arithmetic and printing.

## Decision

- **Storage:** every number is an `f64`. There's no separate integer type.
- **Numbers are words:** a word that looks like a number can be used as one, and a number can be used as a word. `FIRST 345` is `3`, `COUNT 345` is `3`, and `WORD 1 2` is `12`, which is also a number.
- **Printing:** whole numbers print without a decimal point, so `PRINT 4/2` shows `2`. Other numbers print with up to 15 significant digits and no trailing zeros, so `PRINT 3/2` shows `1.5` and `PRINT 0.1 + 0.2` shows `0.3`, not `0.30000000000000004`. Very large and very small numbers use exponent notation.
- **Equality:** `EQUALP` and `=` compare numbers numerically, so `2 = 2.0` is true.
- **Integer-only commands** (`REMAINDER`, `MODULO`, `ITEM`, `REPEAT`, `RANDOM`, palette numbers…) accept a number with no fractional part and give an error for anything else.
- **Errors:** dividing by zero, and any result that isn't finite, is an error rather than producing infinity or NaN.

## Consequences

- Integers are exact up to 2^53, which is plenty for drawing programs.
- A value may be held as a number or as a word internally. Converting between them must give the same printed form, so `WORD 1.50 ""` is the word `1.50` as typed, but `1.50 + 0` prints `1.5`.
