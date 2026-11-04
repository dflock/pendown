# The error card

Status: ready-for-agent
Type: task
Blocked by: 01, 05

## What

A new view, `crates/pendown/src/error_card.rs`, over the editor, built on what the spike found. It draws an error's `Explanation`, as in the mockup's "Paused on the line" and "Narrow pane" states:

- **The message,** at the end of the line, in red on a pale background. It takes no clicks.
- **The card,** under the line and starting at the error's column: "Written like this", "Example", "Esc hides this" and the Help button. Syntax and example use the editor's font and code colours, and it takes clicks so they don't reach the hidden code. The wheel over it still scrolls the editor.
- **Help on REPEAT** does what the console's button does (`open_help`). A procedure shows "Defined on line 1" instead.

Placement goes in a pure function with tests, taking the line's bounds, the card's size and the pane's size:

- It flips above the line when there's no room below, and stays inside the pane sideways.
- The message moves into the card when it doesn't fit beside the line.
- Nothing shows when the line is out of view.

For this issue, show the card whenever the cursor is on an error's line. Issue 07 adds the timing and the other states.

The red must not be the only signal. The wavy underline and the words do that job, and text on the pale red needs good contrast. The gutter's red pill and green arrow are already kept apart for colour-blind eyes.

## Done when

- Placement's unit tests pass, including a pane too narrow for the message, and the first and last lines.
- On the private display, screenshots match the mockup's two states, including a long line and a line scrolled sideways.
- Help opens the right page, and the card stays right through scrolling and pane drags.
- `cargo fmt --all`, the mingw check, tests and the scripted session pass.

## Comments
