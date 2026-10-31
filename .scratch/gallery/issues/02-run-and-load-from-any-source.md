# Run and Load from any source

Status: resolved
Type: task
Blocked by: 01

## What

The help panel can run a help example or a lesson's program on the canvas (`run_example`, `ExampleRun`), and load a lesson's program into the editor (`After::Load { lesson, program }`). The gallery needs both for any `Source`, files included.

- **Run:** `ExampleRun` takes any `Source`. The banner over the canvas says what's running, such as "A garden at night, from the gallery", with **Back to my drawing** as now.
- **Load:** `After::Load` takes a `Source`, not a lesson and program. It still makes a new, untitled program (so Save can't write over the user's file), clears the breakpoints, and asks first if the editor has unsaved changes, as now.
- The lessons' Load and Run go through the same code, unchanged to the user.

## Done when

- The existing scripted-session sections for lessons and help examples still pass.
- A unit test, or the scripted session, loads a file source into the editor and runs one on the canvas.

## Comments

2026-10-31: Done. `run_source` already took any `Source`; `After::Load` now takes a `Source`. The banner says “A snowflake”, from the gallery. The scripted session runs and loads gallery entries from all three sources.
