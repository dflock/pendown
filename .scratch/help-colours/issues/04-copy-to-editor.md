# The current colour bar and Copy to editor

Status: done
Type: task
Blocked by: 03

## What

- The current colour bar at the bottom of the Colours page, always in view, headed **Current colour**: a swatch over a checkerboard, the text that will go in the editor, the form (RGB, RGBA, Hex, and Name or Palette when the colour is exactly one), and **Copy to editor**.
- The picker sets the colour in the form last chosen for it, Hex at first; a palette swatch sets it as Palette, a name as Name.
- Copy to editor inserts at the editor's cursor, replacing a selection, with a space before it unless the character before is a space, `[` or the start of a line; one step for the editor's Undo. Disabled while running, with the tooltip "Stop the program to change it".
- Session checks: pick a colour with the fields, copy it in each form; click a palette swatch and a name; Undo in the editor; screenshots of the page, the picker and the result.

## Done when

It works as in the spec and the mockups, and the scripted session passes with its screenshots.

## Comments

### 2026-10-29: done

The bar, headed "Current colour", has the swatch, the text, Copy to editor beside it, and the form buttons under it, the chosen one solid. Copy to editor uses the editor's `replace`, so it takes the place of a selection and is one step for Undo, then focuses the editor. While a program runs it's disabled, with its tooltip. The Colours page no longer follows the editor's cursor, which would have taken it away as the cursor is placed.

Session: "Colours" sections, with steps `ShowPanel`, `ScrollHelp`, `ColourField`, `WheelAt`, `PickPalette`, `PickName`, `ChooseForm`, `CopyColour`, `CheckCurrentColour`, `CheckColourProblem` and `FilterNames`. Screenshots `colours`, `colours-problem`, `colours-names` and `colours-copied`. The whole session passes.
