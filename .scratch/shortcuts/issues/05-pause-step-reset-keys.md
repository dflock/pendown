# Keys and tooltips for Pause/Resume, Single Step and Reset

Status: resolved
Type: task
Blocked by: 04

## What

Shortcuts and tooltips with their keys for the run controls, as in VS Code's debugger, since F5 already runs:

- **F6:** Pause, or Resume when paused (one button, one action).
- **F10:** Single Step.
- **Shift+F5:** Reset, which stops and puts everything back. There's no separate Stop button.

Each key does nothing when its button is greyed out. The three are rows in the Shortcuts tab's "Your program" group.

## Done when

- The scripted session checks the tab shows the three keys, and that pressing F6, F10, F6 and Shift+F5 in the editor pauses, steps, carries on and resets.

## Comments

2026-11-01: Done: `PauseOrResume`, `SingleStep` and `ResetProgram` actions, in the table and the tooltips. The Pause button's tooltip follows its label: "Pause before the next instruction" or "Carry on from here". The session takes a screenshot of the Resume tooltip.
