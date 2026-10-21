# The palette and filled shapes

Status: ready-for-agent
Type: task

## What

- **`SETPALETTE` and `PALETTE`:** colour numbers 8 to 15 can be redefined, in any colour form but a number. Run puts the palette back.
- **`SETFILLCOLOR`/`SETFC` and `FILLCOLOR`/`FC`** (ADR 0017): shapes are filled with the fill colour, even with the pen up. `"none` turns it off.
- **Drawing:** `pendown-render` fills shapes, then draws their outline. The CLI and PNG export include fills.
- **Animating:** a filled shape sweeps its outline round, as before, and the fill appears when it's done. A filled shape with the pen up takes the time of its outline.

## Done when

Each command has tests of its behaviour and its errors in `pendown-logo`, or the GUI and CLI tests cover it.

## Comments
