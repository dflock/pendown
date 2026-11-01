# Tooltips with keys, README and session

Status: resolved
Type: task
Blocked by: 02, 03

## What

- Run's tooltip shows its key ("Run · Ctrl+Enter"), from the keymap, and so do the file buttons that have one.
- The README mentions the Shortcuts tab, and the scripted session's list.
- The scripted session: open the Shortcuts tab, check rows for Run and Undo show their keys, screenshot it; press F5 and check the program runs; hover Run and check its tooltip.
- An ADR: shortcuts are shown from the keymap, with one table of what they do.

## Done when

- The full scripted session reports 0 checks failed.

## Comments

2026-11-01: Done: Run, New, Open, Save and Export tooltips show their keys. README, ADR 0031, and a Shortcuts section in the session: every row bound, Run, Undo, Cut/copy/paste, Undo the last move and Turn left show the right keys, F5 and Ctrl+Enter run from the editor, and a screenshot of Run's tooltip. The keys are sent to the editor: with nothing focused, the app's shortcuts don't reach it, as before.
