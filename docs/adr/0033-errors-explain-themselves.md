# 33. Errors explain themselves, on a card over the code

Status: Accepted
Date: 2026-11-04

## Context

A red underline told a learner something was wrong, but they had to know to hover to find out what. What they then read often didn't help: "Not enough inputs to REPEAT" doesn't say how many inputs, or what they look like. The reference already knows how each command is written.

## Decision

- **An overlay, not a gap between the lines.** The message shows at the end of the line, with a card under it saying how the command is written, an example that works, and a Help button. The editor's only way to open a gap is its ghost text for inline completions, which is tied to Tab-to-accept. So the card covers the lines under it until the learner moves away or presses Esc.
- **Placed from the scroll offset, the line height and measured text,** as the gutter is. `range_to_bounds` lags a frame, and it reports a line scrolled above the view as being at the top.
- **One error's message at a time, on the cursor's line.** A message on every error line would be noisy in a pane about 460 px wide. Other lines get a red dot and their underline. With several errors on a line, it's the one the cursor is in, or the first.
- **The card is the only popup.** Hovering over an underline shows that error's card, on any line, after 300 ms so that one doesn't flash up as the pointer passes. It stays while the pointer is on the line or the card, so the Help button can be reached. The editor's own hover popup would compete with it, and has no way to be turned off, so it's used as the signal for which error the pointer's on, and cleared before it's drawn.
- **It opens by itself after a pause,** so the learner doesn't have to know to go looking.
- **While typing on a line, nothing new shows there until a pause of a second,** not even the underline. Otherwise every half-typed command gets a red line under it. Moving onto an error's line shows it at once, and errors on other lines show as they always have.
- **A run's error is pinned.** The card shows on its line wherever the cursor is, and the line scrolls into view, until the next edit or run. It's the one error the learner may not be looking near. Moving the cursor onto another error shows that one, and the pinned one comes back after.
- **Esc, or the card's close button, hides the card until that line's text changes.** The underline stays. Hovering over it still shows the card, as that's asking for it, once the pointer has left and come back.
- **Messages are kept apart from the syntax and example.** `explain` in `pendown-logo` puts them together for the console and the card. Messages stay one short sentence, because programs can see them through `CATCH "ERROR`.
- **Messages can stop matching UCBLogo's wording,** which ADR 0005 already asks for by wanting plain English. "REPEAT needs 2 inputs, but got 1" replaces "Not enough inputs to REPEAT".
- **"Did you mean" only for very close matches,** with no other as close, as a wrong guess could mislead a beginner. Words under four letters get no suggestion.
- **The rules for when things show have no window.** `error_display.rs` takes edits, cursor moves, pauses, Esc and runs, and says what to show, so it's tested without timers.

## Consequences

- A program that compares `CATCH "ERROR`'s message with UCBLogo's wording, or with an earlier Pendown's, will see different text. Examples of the old wording remain in ADR 0001, as a record, and in the manual's text in `desc`.
- `FORWARD 10 20` and `REPEAT 4 FD 50` are only found when they run, after the first instruction has run. The parser can't tell yet.
- Two lines with the same text share Esc: hiding the card on one hides it on the other.
- An error on another line shows at once, even if it's only there because a block isn't finished: `SQ has no END` shows on the `TO` line while its body is being typed.
- Relying on the editor setting its popup is a seam with gpui-kit. If an upgrade changes when it's set, hovering stops working, or the red popup comes back.
- The card covers code under it. If it gets in the way in use, opening it with a key instead is an easy change, as the rules are all in one place.
- A command with no `short_example` in `content/reference.kdl` shows its syntax alone. Examples can be added a few at a time.
