# Turtle motion, queries and visibility

Status: ready-for-agent
Type: task
Blocked by: 01

## What

The phase-1 commands that move or ask about the turtle. The details are in `pendown-logo.kdl`.

- **Motion:** `FORWARD`/`FD`, `BACK`/`BK`, `LEFT`/`LT`, `RIGHT`/`RT`, `SETPOS`, `SETXY`, `SETX`, `SETY`, `SETHEADING`/`SETH`, `HOME`.
  - `HOME` is a move: it draws a line when the pen is down.
  - `SETHEADING` sends a `Turn` with the signed angle it turns: the shorter way round, unless the kdl entry says otherwise.
- **Queries:** `POS`, `XCOR`, `YCOR`, `HEADING`, `TOWARDS`. They're answered from the interpreter's own turtle state, never by asking the GUI (ADR 0009).
- **Visibility:** `SHOWTURTLE`/`ST`, `HIDETURTLE`/`HT`, `SHOWNP`, with `ShowTurtle` and `HideTurtle` events. The GUI and `pendown-render` hide the sprite.
- **Clearing:** `CLEAN` (clears the drawing, with a `Clean` event) and `CLEARSCREEN`/`CS` (`CLEAN` plus `HOME`, without drawing the move home).
- **Events:** `Move { from, to, pen }` and `Turn { from, by }` as in ADR 0009.
- Positions are in the turtle area's own coordinates (ADR 0003), with y up.
- **The turtle area:** 1000 × 1000 by default, from -500 to 500. It's configurable through `Limits` or a similar setting.

Turtle modes are issue 06. Until then the turtle behaves as in `WINDOW` mode.

## Done when

- Every command has tests of its events, state and errors in `pendown-logo`.
- `pendown-render` draws `Clean`, and the GUI animates `SETPOS`, `HOME` and `SETHEADING` like any other move or turn.
- A `pendown-cli` picture test covers a program using `SETXY` and `HOME`.

## Comments
