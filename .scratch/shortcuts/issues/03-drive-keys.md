# Drive keys from one table

Status: resolved
Type: task
Blocked by: 01

## What

The drive keys (`driving.rs`) are handled by a `match` on the key's name. Make them a table of key, what it does (`Input`, or stopping), and description, that `drive_key` looks up, the Shortcuts tab's "Driving Kame" group shows, and the drive strip's labels use.

## Done when

- Driving behaves as before (the scripted session's driving sections pass).
- A test checks no key is in the table twice.

## Comments

2026-11-01: Done: `DRIVE_KEYS`, read by `drive_key`, the drive strip and the tab. A key with no modifiers matches with any, as before (Shift+Up drives). The session's `DriveKeys` now goes through the table, as a real key would, and checks each is a drive key. Backspace shows as "Backspace" in the strip, not ⌫.
