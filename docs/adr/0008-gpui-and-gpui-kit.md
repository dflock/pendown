# 8. Build the GUI with GPUI and gpui-kit, drawing the turtle on a raw canvas

Status: Accepted
Date: 2026-10-17 (recorded later, from commit 8104814 and `plan-1.md`)

## Context

The playground needs a code editor, a console and a large canvas for the turtle, in a fast native Rust GUI. We chose GPUI (Zed's UI framework) with the gpui-kit component library. The question was whether either provides a canvas the turtle can draw on.

## Decision

- **GUI framework:** GPUI, with gpui-kit for the layout and components: editor, console, buttons, sliders, panels.
- **Turtle canvas:** gpui-kit has no canvas component of its own, so the turtle draws on gpui's low-level `gpui::canvas`, using `PathBuilder` + `window.paint_path` for vector drawing and `window.paint_image` for pixel buffers. gpui-kit's `brush` example uses this same pattern.
- **Editor:** gpui-kit's editor. Syntax colours come from our own `InputHighlighter` for a `"logo"` language, since there's no tree-sitter grammar for Logo. The running-instruction highlight is a `RangeDecoration` fill, and errors are `Diagnostic`s (ADR 0005).
- **Features:** use `gpui-kit` with `features = ["tree-sitter"]`. Without it, gpui-kit's stub highlight theme makes diagnostics fully transparent.
- **Versions:** gpui-kit depends on `gpui = { package = "gpui-pre", version = "=0.3.9" }`, so we use the same. For reading source, `refs/gpui-kit` is checked out at tag `v0.7.3`, and `refs/zed` at commit `93d7456`, which match the crates.

## Consequences

- gpui has no blend modes, which drives the pixel buffer decision (ADR 0010).
- We're tied to a pre-release gpui pinned by gpui-kit. Upgrading means upgrading both together.
- Building on Linux needs `libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libx11-xcb-dev`. Tested on X11 only.
- Spike 01 (`.scratch/phase-1-spikes/issues/01-gpui-editor-and-canvas.md`) confirmed that this builds and runs here, and that the editor holds 60 fps on a 500-line program while highlighting, re-colouring and showing diagnostics.
