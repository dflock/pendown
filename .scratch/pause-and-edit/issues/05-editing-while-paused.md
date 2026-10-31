# Editing while paused, and Resume and Single Step replaying

Status: resolved
Type: task
Blocked by: 01, 04

## What

- The editor is read-only while running, and editable while paused (`set_readonly` follows "running and not paused").
- While the editor's text differs from the paused run's, the highlight is hidden. Undoing back to the same text brings it back.
- Resume and Single Step, when the text differs: parse the new text. If it doesn't parse, show the errors (ADR 0005) and stay paused. Otherwise end the paused run, clear the drawing, and replay the new program with the old run's seed to the old run's `instructions_shown`, then resume or take the step. The new program is now the run, so a later change replays from it.
- When the text is the same, they work as now.
- If the replay ends first, the run just finishes, as any run does, with no message. An error is shown as at any other time.
- Only the editor's program replays. A paused help example or console entry carries on as now, whatever's in the editor.

## Done when

The scripted session pauses a program, changes a `REPEAT` count and an `FD` in the procedure that's running, resumes, and checks the drawing is the changed program's; checks the highlight goes on a change and comes back on undo; checks a change that doesn't parse leaves it paused with the error shown; and checks a change that makes the program shorter finishes as normal, with nothing new in the console and the toolbar back to Run. Screenshots of each.

## Comments

2026-10-31: Done. `App::changed_while_paused` decides, and `App::replay` replays (it clears the console too, as Run does, since the replay prints again). `show_highlight` won't highlight editor text that differs from the run's, which handles hiding and bringing it back on undo in one place. The scripted session has four ADR 0028 sections. The script's `Type` and undo key presses take effect a moment later, so they need a short `Wait` before checking.
