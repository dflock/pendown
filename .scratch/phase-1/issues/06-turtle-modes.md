# Turtle modes: `WRAP`, `WINDOW`, `FENCE`

Status: done
Type: task
Blocked by: 05

## What

`WRAP`, `WINDOW`, `FENCE` and `TURTLEMODE`, with the interpreter doing all the geometry (ADR 0009). The GUI only scales.

- **`WRAP`** is the default. A move that crosses an edge is split into several `Move` events, and the turtle reappears on the opposite edge. Moves that wrap more than once, and moves along an edge or through a corner, need care.
- **`FENCE`** stops the turtle at the edge with an error, as the manual says. Decide whether the move is drawn up to the edge before the error, and record it in the kdl's `differs` if it differs from UCBLogo.
- **`WINDOW`** lets the turtle move off the area, which then shows nothing of it.
- Switching to `WRAP` or `FENCE` with the turtle outside the area: follow UCBLogo (check `refs/ucblogo-code`).
- `ARC` (issue 08) must wrap and fence like any other move. Design the splitting so arcs can use it too.
- The GUI shows the space outside the turtle area in a slightly different shade (ADR 0003), so the edges are visible.

## Done when

- `pendown-logo` has tests for:
  - a move that wraps once and more than once;
  - moves through corners;
  - `FENCE` errors;
  - `WINDOW` moves off the area;
  - `TURTLEMODE`'s output.
- A `pendown-cli` picture test shows a wrapping spiral.

## Comments

### 2026-10-21: done

- **The geometry** is `src/geometry.rs`: `Area` and `TurtleMode`, and `Area::line`, which splits a straight move for the mode. It's pure and has 11 tests:
  - wrapping once, several times, diagonally, through a corner, and along an edge;
  - a huge move;
  - `FENCE`, `WINDOW`, wrapping a point back in, and other area sizes.
- **The commands:** `WRAP`, `WINDOW`, `FENCE` and `TURTLEMODE`. `WRAP` is the default, and Run resets it.
- **Every move goes through `move_to`:** `FD`, `BK`, `SETPOS`, `SETXY`, `SETX`, `SETY` and `HOME`.
- **Decisions:**
  - **`FENCE`** draws up to the edge, then stops with an error. That includes `SETPOS`, which in UCBLogo gives the error without moving, and the kdl's `FENCE` entry now has a `differs` note saying so.
  - **Switching to `WRAP` or `FENCE`** with the turtle outside the area wraps its position back in, as UCBLogo does. The jump is a pen-up move, so it's never drawn.
  - **A move that wraps more than 10,000 times** stops drawing there, and jumps to where it would end (`MAX_WRAPS`).
- **The area** is configurable with `Interpreter::set_area`, plus `Drawing::set_area` to match. Both default to 1000 × 1000.
- **For issue 08:** `Area::line` handles straight lines only. Arcs can wrap by being split into short chords first.
- **Tests:** `tests/modes.rs` (7 tests), and a `wrapping-spiral` picture. Two animator tests now start with `WINDOW`, since they move past the edge.
