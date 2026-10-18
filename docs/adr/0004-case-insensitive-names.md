# 4. Names are case-insensitive, but keep their spelling

Status: Accepted
Date: 2026-10-18

## Context

Logo has always ignored case in names: `FORWARD`, `forward` and `Forward` are the same command. UCBLogo's `CASEIGNOREDP` variable controls whether word comparisons also ignore case.

## Decision

- **Procedure and variable names are case-insensitive.** They're looked up by their Unicode lowercase form.
- **The spelling from the definition is kept** for display: the first `TO` or `MAKE` that creates a name decides how the GUI shows it in the help panel, errors, and a later variable inspector.
- **Word comparisons** (`EQUALP`, `MEMBERP` and the rest) ignore case while `CASEIGNOREDP` is true, which is the default, as in UCBLogo.

## Consequences

- Built-in commands are documented in capitals, as in the manual, but users can type them in any case.
