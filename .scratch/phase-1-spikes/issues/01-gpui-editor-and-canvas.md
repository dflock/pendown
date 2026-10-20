# Spike: gpui-kit editor and canvas in one window

Status: done
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

### 2026-10-18: spike results

Code: `spikes/01-gpui-editor-and-canvas/` on branch `spike/01-gpui-editor-and-canvas`. Run it with `cargo run --release`. `SPIKE_AUTOTEST=1` runs a timed, scripted session that prints measurements and saves screenshots; the results of the final run are in `autotest.log` and `screenshot-*.png` next to the code.

The app has a gpui-kit layout: toolbar, then the editor and a raw `gpui::canvas` in resizable panels, then a console pane. A tiny Logo lexer drives syntax colours, diagnostics and approximate instruction spans. The canvas draws a 400-segment spiral and a turtle that moves every frame.

**Summary: yes, gpui-kit's editor does everything we need for phase 1, with two setup gotchas.** Every scenario held a steady 60 fps on a 500-line program.

#### 1. Does it build and run?

Yes, on Ubuntu 24.04 under X11, with `gpui-kit = "=0.7.3"` from crates.io and Rust 1.98.

- **System packages:** linking fails without the development packages for xkbcommon. Install `libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libx11-xcb-dev`, the Linux subset of Zed's `script/linux`. These should go in the README.
- **First build** takes around 10 minutes. Later release builds take 5–50 seconds.
- **Turn on gpui-kit's `tree-sitter` feature.** Without it, gpui-kit uses a stub highlight theme whose diagnostic colours are fully transparent, so diagnostics are stored but invisible. The feature adds the tree-sitter core and a JSON grammar, not every language.
- Wayland wasn't tested, and isn't a concern for now.

#### 2. Moving highlight

Works well. A `RangeDecoration` with `RangeDecorationStyle::Fill`, moved to the next instruction every frame with `RangeDecorationCollection::set`:

| Phase (500-line program, 1200×800 window) | Frame interval p50 / p95 / max | Our work per frame |
|---|---|---|
| Idle, requesting frames | 16.67 / 16.74 / 17.27 ms | — |
| Canvas only, turtle moving | 16.66 / 16.72 / 16.76 ms | — |
| Canvas + highlight | 16.67 / 16.73 / 16.95 ms | 0.01 ms |
| Canvas + highlight, following it | 16.67 / 16.73 / 17.67 ms | 0.04 ms p50, 0.15 ms max |

Earlier runs had occasional 33 ms frames, but in every phase including idle, so that was background noise on the machine, not our work.

#### 3. Syntax colours while typing

Works well, but use the editor's highlighter interface instead of decorations. I tried two approaches:

- **`TextDecoration`s**, recomputed for the whole buffer on every change: this works, but it's a layer over the top of the text.
- **An `InputHighlighter`** (`LogoHighlighter` in the spike), installed with `EditorState::set_highlighter_factory` for a `"logo"` language: this is the native route. Token names like `"keyword"`, `"number"`, `"comment"` and `"string"` resolve through the theme's `HighlightStyleResolver`, so the colours follow the theme. **Use this one.**

Typing one character per frame into the 500-line program, re-lexing the whole buffer each time and recomputing diagnostics and spans, the frame interval stayed at 16.67 ms p50 and 17.05 ms max. The highlighter's re-lex took 0.24 ms p50 (1.08 ms max), and diagnostics plus instruction spans took 0.34 ms p50 (1.19 ms max). Lexing the whole buffer on every keystroke is fine at this size; incremental lexing isn't needed.

#### 4. Diagnostics

Work, with the red wavy underline from the theme (`screenshot-1-sample.png`, under `FOOBAR`). Two gotchas:

- The `tree-sitter` feature is needed, as above.
- `DiagnosticSet` converts line/column positions to byte offsets using its own copy of the text. Call `set.reset(state.text())` before `set.extend(...)`, or diagnostics added before the first edit end up zero-length.

**Hover message:** works. A script can't move the mouse, so this was checked by hand: hovering over `FOOBAR` shows gpui-kit's diagnostic popover (`present_diagnostic` in `editor/lsp/overlay.rs`) with the message.

#### 5. Scrolling to keep the highlight visible

Works (`screenshot-2-canvas.png`, scrolled to line 319). There's no "scroll to range" API, but this works:

1. Read `visible_row_range()` and `line_height()`.
2. If the highlighted line is within 3 lines of either edge, call `set_scroll_offset(point(x, -line_height * (line - 3)))`. Negative y scrolls down.

This assumes no soft wrap; with soft wrap, display rows and buffer lines differ.

#### 6. Missing features

Built in:
- line numbers;
- indent guides;
- undo and redo;
- search (Ctrl-F);
- multiple cursors;
- auto-indent and auto-closing brackets, configured through a `LanguageConfig`;
- folding, if our highlighter returns fold ranges;
- read-only mode;
- LSP-style provider traits for completion, hover, go-to-definition and code actions. These could serve the help panel's "help for the command under the cursor" and completion of command names from `pendown-logo.kdl`.

Not found, so we'd build them ourselves:
- **Toggle comment:** add a `;` at the start of selected lines.
- **Matching-bracket highlight:** we could do this with a `RangeDecoration` on cursor move.

#### Changes this suggests

- **ADR 0008 / README:** list the Linux packages, and use `gpui-kit` with `features = ["tree-sitter"]`. Done.
- **Plan:** syntax colours come from our own `InputHighlighter`, not decorations, and the running-instruction highlight is a `RangeDecoration` fill. Done. No decisions change.
- **Not retested:** in the very first run, typing into one line that grew to about 230 characters dropped frames to about 30 fps. Ordinary line lengths didn't. Worth a look if long lines turn out to matter.

### 2026-10-20: closed

Hover popover checked by hand and works. Wayland is out of scope for now. The suggested changes are made in ADR 0008, `README.md` and `plan-1.md`.
