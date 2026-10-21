# Phase 1 spikes

Time-boxed experiments to take the biggest risks out of phase 1 before building it for real. See `implementation-plan.md` for the overall design, and `docs/adr/` for decisions already made.

Each spike answers specific questions and ends with a short write-up under `## Comments` in its issue: what worked, what didn't, numbers measured, and anything that should change the plan or an ADR. Spike code is throwaway unless the issue says otherwise.

| Issue | Risk it takes out |
|---|---|
| `01-gpui-editor-and-canvas` | Does gpui-kit build here, and can its editor do the highlighting and diagnostics we need? |
| `02-pixel-buffer-streaming` | Can we show a full-resolution pixel buffer in gpui every frame, fast enough? |
| `03-interpreter-core` | Do the parser, spans, tail calls and deep recursion work, and how fast is it? |
| `04-end-to-end-animator` | Does the headline feature (speed, Pause, Step, Stop, highlighting) work as designed? |

Order: 01 and 02 together (both gpui), 03 alongside them (independent, pure Rust), then 04, which needs all three.
