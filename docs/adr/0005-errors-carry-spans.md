# 5. Errors carry source spans and show as editor diagnostics

Status: Accepted
Date: 2026-10-18

## Context

Logo's traditional error messages are just text printed at the prompt, such as "I don't know how to FOO in BAR". A GUI can point at the code that went wrong. gpui-kit's editor has built-in diagnostics (`crates/base/src/input/editor/diagnostics.rs`).

## Decision

- **Every error carries a span:** the source it came from and the byte range within it, using the same spans as instructions (ADR 0001).
- **Parse errors** come from the editor's static parse and show as diagnostics while the user types.
- **Runtime errors** stop the program, show a diagnostic on the instruction that failed, and print the message in the console.
- **Errors inside procedures** also record the chain of calls, such as "in SQUARE, called from TRIANGLE, called from line 3", and each step can be clicked to jump to it.
- **Messages** are in plain English and say what to do where possible. Errors for obsolete commands use their `obsolete_reason` from `pendown-logo.kdl`.

## Consequences

- The error type lives in the logo crate, with no GUI code. The GUI turns it into a gpui-kit `Diagnostic`.
- A tail call replaces its caller, so it leaves no entry in the chain, as in UCBLogo.
- An error from a deep recursion has a chain of thousands of identical entries. The GUI should collapse repeats, such as "called from DOWN, 10,000 times".
- `CATCH "ERROR` and `ERROR` (phase 2) must expose the same information to Logo programs, as UCBLogo's error list.
- **How errors show on the code has moved on (2026-11-04).** A card over the editor explains them, and an error on the line being typed waits for a pause: see ADR 0033.
