# Spike: gpui-kit editor and canvas in one window

Status: ready-for-agent
Type: prototype

## Question

Does gpui-kit (with `gpui-pre 0.3.9`) build and run on this Linux machine, and can its code editor do what the playground needs?

## What we know

gpui-kit's editor (`refs/gpui-kit/crates/base/src/input/editor/`) has:

- `RangeDecoration`: fills or frames a UTF-8 byte range. Candidate for the running-instruction highlight.
- `TextDecoration` with a `HighlightStyle`: candidate for syntax colours from our own lexer, since there's no tree-sitter grammar for Logo.
- `Diagnostic` / `DiagnosticSet`: candidate for errors (ADR 0005).

## Build

A small app with a gpui-kit layout: the editor on one side and a `gpui::canvas` on the other, with a console pane underneath. Follow `refs/gpui-kit/examples/brush` for the canvas.

## Answer these

1. Does it build and run here? Which windowing backend does it use (Wayland or X11), and does it need any system packages?
2. **Moving highlight:** move a `RangeDecoration` along the text 60 times a second. Is it smooth, and what does it cost per frame?
3. **Syntax colours:** recolour the whole buffer with `TextDecoration`s from a simple tokenizer on every keystroke, with a 500-line program. Does typing stay responsive?
4. **Diagnostics:** can we show an error on a byte range with a message on hover?
5. Can the editor scroll to keep the highlight visible?
6. Anything missing that we'd have to build ourselves (line numbers, bracket matching, auto-indent, undo)?

## Done when

The questions are answered in a comment below, with any numbers measured, and any changes needed to `plan-1.md` noted.

## Comments
