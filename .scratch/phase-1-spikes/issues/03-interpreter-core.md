# Spike: interpreter core

Status: ready-for-agent
Type: prototype

## Question

Do the parsing and evaluation decisions (ADRs 0001, 0002, 0004, 0005, 0006 and 0007) work in practice, and is the interpreter fast enough for Instant mode?

## Build

A `pendown-logo` library crate (ADR 0019), with no GUI code, that implements about ten commands: `FD`, `RT`, `REPEAT`, `TO`/`END`, `IF`, `STOP`, `OUTPUT`, `MAKE`, `PRINT`, plus infix arithmetic. It sends drawing events to a channel, as in `plan-1.md`. This spike may become the start of the real `pendown-logo` crate, so write it with that in mind.

## Answer these, with tests

1. **Endless tail recursion:** `TO SPIRAL :n FD :n RT 90 SPIRAL :n + 1 END` runs for a long time with memory staying flat (ADR 0007). Also test tail calls inside `IF` and `IFELSE`, and mutual recursion.
2. **Deep recursion that isn't a tail call:** a tree fractal 12 levels deep, and recursive Fibonacci of 25, work without overflowing. Measure how many nested calls a 256 MB stack allows, and recommend the depth limit (ADR 0007). Going past the limit gives a friendly error, not a crash.
3. **Calls before definitions:** a call above its `TO` in the same file parses correctly (ADRs 0001 and 0006).
4. **Minus sign:** `FD -5`, `:a - 5`, `:a -5`, `:a-5`, `- 3 + 4` and `MINUS 3 + 4` all parse as ADR 0001 says.
5. **Spans:** every instruction carries the right span, including inside `REPEAT` lists and procedure bodies.
6. **Numbers:** printing and equality follow ADR 0002.
7. **Stop:** setting the cancel flag stops a running `FOREVER` promptly.
8. **Speed:** how many instructions per second for a simple drawing loop, in release mode?

## Done when

The tests pass, the questions are answered in a comment below with the measured speed, and any ADR changes are noted.

## Comments
