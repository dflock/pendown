# 29. Breakpoints

Status: Accepted
Date: 2026-10-31

## Context

Testing with real users (`.scratch/pause-and-edit/feedback-from-user-testing.md`) asked for "run to here": getting quickly to the part of a program you care about, then watching or stepping from there. That's a breakpoint. ADR 0028's replay is "run quickly to a point", and this builds on it. gpui-kit's editor (0.8.1) has no event for a click on its line numbers and no way to mark them.

## Decision

- **Click a line number to set a breakpoint;** it shows as a white number on a red pill, and the line's code is faintly tinted. Click again to take it away. There can be as many as you like. A line number that takes one has a tooltip saying what a click does: "Stop here: Kame waits before this line", "Stop each time TILE is called" on a `TO` line, or "Take the breakpoint away".
- **Pendown draws the line numbers itself,** in a column beside the editor (`gutter.rs`), and turns off gpui-kit's numbers and its folding. Each row follows the editor's scroll offset and line height, below its 8 px top padding. Soft wrap and folding are off, so each row is one line. Drawing our own numbers makes the pills, the hover and the clicks ordinary elements, rather than working out where gpui-kit draws its numbers from private widths. When gpui-kit's gutter decorations are released, they can replace the tint, but the click still needs our own numbers.
- **Only lines with code take one:** not a blank line, a comment, `END`, or a line of just brackets. A breakpoint on a `TO` line stops each time the procedure is called, at its first instruction.
- **The interpreter knows the breakpoints,** through `Control`, as the editor's lines counting from 0. When an instruction starts a breakpoint's line, the interpreter sends a `Breakpoint` event before the `Instruction`, sends what the panels show, and waits with a `Sync`, even out of lockstep. So Instant stops exactly, and stays fast: only breakpoint lines wait. The animator pauses once the instruction is shown, as Single Step does, with the highlight on it.
- **A breakpoint stops where a line starts,** as `starts_line` does (ADR 0012): when the program comes to that line from another one. `REPEAT 4 [FD 50 RT 90]`, all on one line, stops once, before the `REPEAT`. Spread over several lines, each line is a place to stop, each time round.
- **Breakpoints can be set at any time:** before a run, while it runs, and while paused. Set while running, a breakpoint stops the run from its next line.
- **Replays pass breakpoints** until they get back to where you paused (ADR 0028).
- **Only the editor's program stops at them,** because only its jobs are given them. A help example or console entry never stops, even in the editor's procedures.
- **Breakpoints keep to their lines as you edit:** lines added above move them down, an edit within the line keeps its breakpoint, and deleting the line deletes it. Undo brings back the line, but not the breakpoint.
- **They aren't saved with the program.** Like the cursor, they're for now. New, Open and loading a lesson's program clear them.
- **No message when one stops Kame.** The highlight, the pill and the Resume button show what happened.
- **The line the highlight is on has a green arrow behind its number** (#15803d, with a white number), rounded on the left and pointed on the right, whenever the editor shows the highlight. With a breakpoint on that line too, the red pill sits on the arrow with a white ring round it. The red and green are about as dark as each other, so to red/green colour blind eyes they'd merge: the ring and the arrow's point keep the two apart.

## Consequences

- A breakpoint set while an example is on the canvas applies to the user's program from its next Run, or a replay.
- Not in this ADR: conditional breakpoints, breakpoints in examples, and a list of breakpoints.
