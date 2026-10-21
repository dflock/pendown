# Files: New, Open, Save, and PNG export

Status: ready-for-agent
Type: task
Blocked by: 03

## What

- **New, Open, Save and Save As** for `.logo` files: plain UTF-8, one program per file (ADR 0006).
  - Use the platform file dialogs from gpui (`cx.prompt_for_paths`, `cx.prompt_for_new_path`).
  - Use the usual keyboard shortcuts.
- **The window title** shows the file's name, and marks unsaved changes.
- **Warn before losing unsaved changes:** on New, on Open, and on closing the window.
- **Export PNG** saves the pixel buffer as it is, at the canvas's device resolution, with the turtle sprite left out (ADR 0010).
- **Remember** the last folder used.

## Done when

- A program can be written, saved, closed, opened again and run.
- The export matches what's on the canvas.
- The unsaved-changes warning appears in all three cases.

## Comments
