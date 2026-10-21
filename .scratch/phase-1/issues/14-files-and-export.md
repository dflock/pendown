# Files: New, Open, Save, and PNG export

Status: done
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

### 2026-10-21: done

- **New, Open, Save, Save As and Export PNG**, as toolbar buttons and the usual shortcuts: `secondary-n`, `-o`, `-s`, `-shift-s` and `-e`, which is Ctrl on Linux and Windows and Cmd on macOS. `secondary-enter` runs the program.
  - The file dialogs are gpui's `prompt_for_paths` and `prompt_for_new_path`.
  - Save As adds `.logo` when there's no extension.
- **The document** (`pendown::document::Document`, with 3 unit tests) knows the file and the text as last saved.
  - The window title is "name.logo — Pendown", with "● " in front when there are unsaved changes.
  - The welcome program counts as unchanged.
- **Unsaved changes:** New, Open and closing the window ask "Save the changes to …?", with Save, Don't Save and Cancel (`window.prompt`, then `files::After` says what to carry on with). Closing goes through `on_window_should_close`, and closing the last window quits the app.
- **Export PNG** saves the pixel buffer at the canvas's device resolution, without the turtle.
  - **Bug found and fixed:** the buffer only draws new marks when the canvas paints, so an export straight after a run was blank. Export now draws anything pending first.
- **The last folder** is remembered between sessions in `$XDG_CONFIG_HOME/pendown/last-folder` (or `~/.config`, or `%APPDATA%`; `PENDOWN_SETTINGS` overrides it).
- **The scripted session** checks the title before and after edits and saves, saving, opening again, and that an export is the canvas's size with something drawn. 30 checks, all passing.
- **Not checked automatically:** the platform file dialogs and the "Save the changes?" prompts. The scripted session calls `save_to`, `open_path` and `export_to` directly. Try them by hand, which issue 16 covers.
