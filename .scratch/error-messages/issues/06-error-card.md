# The error card

Status: resolved
Type: task
Blocked by: 01, 05

## What

A new view, `crates/pendown/src/error_card.rs`, over the editor, placed as issue 01's comments say: from the scroll offset, line height and measured text width, not `range_to_bounds`. It draws an error's `Explanation`, as in the mockup's "Paused on the line" and "Narrow pane" states:

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

2026-11-04: Done, in `crates/pendown/src/error_card.rs`. The app's `set_code_errors` sets the underlines and the card together.

Looked at on the private display (X11, Xvfb), with screenshots:

- The "Paused on the line" and "Narrow pane" states match the mockup. So do a procedure ("Defined on line 1"), a bare message with no card, and an obsolete command with the message in the card.
- A long line, a line scrolled sideways, and a card flipped above the last line.
- The wheel over the card scrolls the editor and the card stays on its line, until the line is out of view and both go. Dragging the pane narrower and wider keeps it right.
- A click on the card leaves the cursor where it was. Help on REPEAT opens REPEAT's page.

**Different from the issue:**

- A card shows only with something to say. An error with no command, such as "This ] has no matching [", shows just the message, unless that doesn't fit beside the line. Then a card holds it.
- An obsolete command keeps its Help button, so it gets a card.
- The card is a fixed 350 px, as in the mockup, or the pane's width less a margin if that's narrower. Code that's too long is cut off.
- The card's height is worked out from constants and the message's wrapped lines, as placement needs it before layout.

**For issue 07:**

- "Esc hides this" shows, but Esc doesn't do anything yet.
- The hover popup can show at the same time as the card, which I saw. It's on 07's list.
- It shows on every edit, and picks the first error on the line.

**Not tried:** the one-frame lag when the editor scrolls itself wasn't looked at frame by frame, only in screenshots. Wide characters, and scales other than 1x, also weren't tried.

`cargo test --workspace` and the mingw check pass. In the scripted session the only failure is "frame intervals" on the 131,070-move tree, as in issues 02 to 05.
