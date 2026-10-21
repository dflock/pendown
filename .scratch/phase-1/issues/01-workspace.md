# Workspace, with `pendown-logo` and `pendown-render` from the spikes

Status: ready-for-agent
Type: task

## What

Set up the Cargo workspace from `spec.md`, and move the spikes' code into it.

- A root `Cargo.toml` workspace with `crates/pendown-logo`, `crates/pendown-render`, `crates/pendown` and `crates/pendown-cli`. `pendown` and `pendown-cli` can start as stubs; issues 03 and 04 fill them in.
- **`pendown-logo`:** copy `spikes/03-interpreter-core`, with spike 04's additions (`Control`, `Sync`, `PU`, `PD`, `SETPC`). Keep its tests and `bench` binary.
  - Fix ADR 0004's spelling rule while moving it: the first `TO` or `MAKE` that creates a name decides its display spelling, not the first time the name is seen.
  - Set `[profile.dev.package.pendown-logo] opt-level = 1` at the workspace root (ADR 0007).
- **`pendown-render`:** the drawing from `spikes/04-end-to-end-animator/src/canvas.rs` without gpui:
  - the display list (ADR 0003);
  - a premultiplied pixel buffer drawn by vello_cpu with red and blue swapped (ADR 0010);
  - dirty-tile tracking, as a list of changed 256 × 256 tiles that the GUI uploads;
  - re-rendering at a new size;
  - PNG encoding, which swaps red and blue back.
  
  Its input is `pendown-logo`'s events, so it depends on `pendown-logo`.
- Pin the shared dependencies once, in `[workspace.dependencies]`: `gpui-kit = "=0.8.0"` with `tree-sitter`, `vello_cpu`, `image`, `smallvec`.
- Update ADR 0019 with the fourth crate, `pendown-render`, and why it exists (`spec.md`, "Crates").
- `README.md`: how to build, run and test the workspace.

## Done when

- `cargo test` at the root passes, including all of spike 03's tests, now in `pendown-logo`.
- `pendown-render` has tests: drawing a few segments and checking pixels, and re-rendering at a new size.
- The spikes still build on their own, untouched.

## Comments
