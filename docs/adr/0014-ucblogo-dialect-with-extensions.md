# 14. UCBLogo is the base dialect; keep dynamic scope; record our extensions

Status: Accepted
Date: 2026-10-18 (recorded later, from commit 5bc4b58)

## Context

There are many Logo dialects: UCBLogo, FMSLogo/MSWLogo, jsLogo, the LCSI family (LogoWriter, MicroWorlds), LibreLogo, Terrapin, PC Logo, and loosely NetLogo and Snap!. We looked for an existing Rust Logo interpreter to build on and found none suitable (see `refs/`): none track source positions or emit an event stream, which the playground's core features need, so we write our own.

## Decision

- **Implement UCBLogo.** It's the closest thing Logo has to a standard, it's the best documented (the full reference manual, Brian Harvey's books), and most later Logos follow it. The dialects mostly differ in the commands we've already made obsolete or put in late phases.
- **Keep dynamic scope:** a procedure can see the local variables of whatever called it. Every Logo works this way and Logo teaching material relies on it. How it's implemented is in ADR 0007.
- **"Our Logo" is UCBLogo's core plus a few deliberate changes and extensions,** borrowed from other dialects where they're better: colours (ADR 0015), `ARC` (ADR 0016) and shapes (ADR 0017).
- **Every change is recorded in `our-logo.kdl`:** new commands have `extension #true` and a `source` saying where the idea came from; changed commands keep the manual's `desc` and add a `differs` node.

## Consequences

- References kept in `refs/`: `ucblogo-code` (the C source, for exact behaviour), `jslogo` (the closest prior art to this playground) and `logo-rust`.
- Some UCBLogo programs will behave differently, notably ones that use `ARC` or RGB percentages. That's accepted, since running old programs isn't a goal.
