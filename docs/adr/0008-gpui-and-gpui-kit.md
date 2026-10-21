# 8. Build the GUI with GPUI and gpui-kit, drawing the turtle on a raw canvas

Status: Accepted
Date: 2026-10-17 (recorded later, from commit 8104814 and `implementation-plan.md`). Updated 2026-10-20 for gpui-kit 0.8.0.

## Context

The playground needs a code editor, a console and a large canvas for the turtle, in a fast native Rust GUI. We chose GPUI (Zed's UI framework) with the gpui-kit component library. The question was whether either provides a canvas the turtle can draw on.

## Decision

- **GUI framework:** GPUI, with gpui-kit for the layout and components: editor, console, buttons, sliders, panels.
- **No Webviews:** Native components only.
- **Turtle canvas:** gpui-kit has no canvas component of its own, so the turtle draws on gpui's low-level `gpui::canvas`, using `PathBuilder` + `window.paint_path` for vector drawing and `window.paint_image` for pixel buffers. gpui-kit's `brush` example uses this same pattern.
- **Editor:** gpui-kit's editor. Syntax colours come from our own `InputHighlighter` for a `"logo"` language, since there's no tree-sitter grammar for Logo. The running-instruction highlight is a `RangeDecoration` fill, and errors are `Diagnostic`s (ADR 0005).
- **Features:** use `gpui-kit` with `features = ["tree-sitter"]`. Without it, gpui-kit's stub highlight theme makes diagnostics fully transparent. The lighter `syntect` feature, new in 0.8.0, uses the same stub theme, so it doesn't help. Also, `syntect` highlighting is outdated and not great, so we'll stick to tree-sitter.
- **Versions:** we use `gpui-kit = "=0.8.0"`. It pins `gpui = { package = "gpui-pre", version = "=0.3.9" }`, and through it re-exports GPUI, so we don't list GPUI ourselves. For reading source, `refs/gpui-kit` is checked out at tag `v0.8.0`, and `refs/zed` at commit `93d7456`, which match the crates.
- **Finding the Zed commit:** each `gpui-pre` release is a snapshot of Zed. Its `src/action.rs` starts with a comment naming the commit, for example "snapshot of zed@93d7456". When a gpui-kit upgrade moves to a new `gpui-pre`, move `refs/zed` to the commit that comment names.

## Consequences

- gpui has no blend modes, which drives the pixel buffer decision (ADR 0010).
- We're tied to a pre-release gpui pinned by gpui-kit. Upgrading means upgrading both together, and moving `refs/zed` to match.
- Building on Linux needs `libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libx11-xcb-dev`. Tested on X11 only.
- Spike 01 (`.scratch/phase-1-spikes/issues/01-gpui-editor-and-canvas.md`) confirmed that this builds and runs here, and that the editor holds 60 fps on a 500-line program while highlighting, re-colouring and showing diagnostics.
