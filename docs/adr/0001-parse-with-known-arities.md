# 1. Parse with known arities, following UCBLogo's minus-sign rules

Status: Accepted
Date: 2026-10-18

## Context

Logo can't be parsed on its own: how `FOO 1 2` groups depends on how many inputs `FOO` takes, and user procedures can be defined further down the file than where they're called. The minus sign is also ambiguous: `FD -5`, `:a - 5` and `:a -5` all look alike.

We also need source spans on every instruction, for highlighting the running instruction and stepping (see `implementation-plan.md`), and for errors (ADR 0005).

## Decision

**Arities come from the procedures defined when a line is parsed.** Running a program defines every `TO` in it before running any top-level instruction (see ADR 0006), so a call can come before its definition in the file.

**Tokens keep their spans.** Lists written in the source keep the spans of their members. Brackets are data in Logo: `[FD 50 RT 90]` is a list of words until something like `REPEAT` runs it. So an instruction list is parsed into instructions when it's first run, using the spans its words carry, and the parse is cached. Lists built at run time (`RUN`, phase 2) have no spans; while they run, the highlight stays on the instruction that ran them.

**The editor also parses statically**, with the arities of every `TO` in the buffer, so it can show errors while the user types.

**Minus sign:** follow UCBLogo exactly, as described under `DIFFERENCE` and `MINUS` in the manual:

- `-` is unary minus if the previous token is an infix operator or an open parenthesis, or if it's preceded by a space and followed by a non-space. Otherwise it's infix difference.
- So `FD -5` and `:a -5` (two inputs: `:a` and `-5`) use unary minus, and `:a - 5` and `:a-5` are differences.
- Unary minus binds tighter than infix operators: `- 3 + 4` is `(-3) + 4`, whereas `MINUS 3 + 4` is `-(3 + 4)`.

## Consequences

- Redefining a procedure with a different number of inputs while a program runs only affects lines parsed after that. Cached parses of lists keep the old arity. This is an edge case that new programs are unlikely to hit.
- The parser needs a table of arities for the built-in commands, including the ones that take a variable number of inputs in parentheses, such as `(SENTENCE a b c)`, `(SQUARE side corner)`.
- The tokenizer has to remember whether there was whitespace before and after each `-`.
