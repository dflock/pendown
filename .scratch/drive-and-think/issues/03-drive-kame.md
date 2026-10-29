# Drive Kame

Status: done
Type: task
Blocked by: 02

## What

- A toolbar toggle, **Drive**, which focuses the canvas; Esc, Run, Pause and Step stop driving.
- The strip across the top of the canvas: what the keys do, the move step (1, 5, 10, 25, 50) and turn step (1, 5, 15, 30, 45, 90), and `Pen up` when it is.
- Keys: ↑ `FD`, ↓ `BK`, ← `LT`, → `RT`, Space `PU`/`PD`, Backspace and Ctrl+Z Undo.
- Each press is a console entry; presses of the same key in a row join into one, carried on without saving Kame for Undo, with the entry's text and console line updated.
- Its own speed, 100 steps and 150° a second; presses made while one plays wait their turn.
- Thinking always shown while driving, with the line out in front of Kame and `RT 45 · heading 135`.
- An ADR: driving is console entries; thinking is never part of the drawing.

## Done when

Driving works as in the spec and mockups, Undo and Teach Kame work on driven entries, and unit tests cover joining presses.

## Comments

### 2026-10-29: done

`pendown::drive` (no window, unit-tested): the keys, the steps, `press` (a new entry, or more of the open one with its new text), `pen`, `caption`, and `DRIVE_SPEED`. `ConsoleInput::amend_last` grows the newest entry and its history.

`driving.rs` is the window's part, an `impl App`: Drive on and off (it focuses the canvas; Esc, Run, Pause and Step stop it), presses as console entries, joining the same key with `Job::Define` (no Kame saved for Undo) and the console line changed in place, presses queued while one plays and taken as each ends (`drive_next`), Undo ending the entry and putting the pen back for `PU`/`PD`, thinking joined across an entry's presses with the last entry fading, the caption, and the strip with its step choices. The canvas draws the dashed line ahead and the caption chip below Kame, clear of the arc. `start_job` plays at `DRIVE_SPEED` while driving. ADR 0026 records it.

Toolbar: New, Open, Save and Export are icons with tooltips now, to make room for Drive (Lucide `gamepad-2`, added to `ExtraIcons`) and Thinking. Drive is disabled while a program runs.

An entry joined from ten presses is ten lines in the drawing, which look as one. Space assumes the pen is down when driving starts.

Session: "Drive Kame" sections, with steps `Drive`, `DriveKeys`, `WaitDriven` and `CheckDriving`: joined entries, Kame's place, Undo of a whole entry, the pen, Teach Kame on the result, and Esc and Run stopping driving. Screenshot `driving`.
