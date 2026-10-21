# Pixel drawing spikes

Status: deferred to phase 5. These were drafted for phase 2, then put off along with the commands they test.

Time-boxed experiments to take the risks out of the drawing commands that work on pixels or need text: `FILL`, `FILLED`, `LABEL`, and the pen modes. See "Phase 5" in `implementation-plan.md` for the scope, and `docs/adr/` for decisions already made. Run them before those commands' spec is written.

As in phase 1, each spike answers specific questions and ends with a short write-up under `## Comments` in its issue: what worked, what didn't, numbers measured, and anything that should change the plan or an ADR. Spike code is throwaway, but whatever works is expected to move into `pendown-render`.

| Issue | Code | Risk it takes out |
|---|---|---|
| `01-fill-and-pen-modes` | `spikes/05-fill-and-pen-modes` | Do `FILL`, `FILLED`, `PENERASE` and `PENREVERSE` work on an anti-aliased buffer that's redrawn at any size? |
| `02-labels` | `spikes/06-labels` | Can `LABEL` draw text into the pixel buffer, the same in the GUI and the CLI, without gpui? |

The two are independent and can run in parallel. Both build on `pendown-render` by path, so they test the real buffer, its tiles and its redraw on resize, rather than a copy.

The spike folders carry on from the last spike's number, so `spikes/` stays in order. 05 and 06 are free as of phase 2's start. As in phase 1, each spike keeps its own `[workspace]` and shares `spikes/target`.
