# When the card and underline show

Status: resolved
Type: task
Blocked by: 06

## What

The rules in the spec's table, as a small state type with no window, like `drive.rs`, so they can be tested without timers. It takes edits, cursor moves, a pause, Esc, run start and run errors, and says what to show.

- **The pause.** About a second after the last edit, using `cx.background_executor().timer` as `app.rs` does elsewhere, and restarted by each edit. Moving onto an error's line shows it at once, as the learner isn't typing there.
- **The underline waits.** The check after each edit (`app.rs`, about line 1245) holds back errors on the cursor's line until the pause. Errors on other lines show at once, as now.
- **Other lines** get a red dot at the end of the line.
- **Several errors on a line.** The message and card show the error the cursor is in, which means anywhere in its underlined text or just after it. They switch at once when the cursor moves into another, with no pause, and the card moves to that error's column. With the cursor in none of them, they show the first. This also applies when several errors from Run are on the cursor's line. Esc hides the card for the whole line.
- **A run's error** (`show_errors`) is pinned: the card shows on its line wherever the cursor is, and the line scrolls into view. The next edit or run clears it. This includes a parse error found by Run, and an error inside a procedure, whose call chain stays in the console.
- **Esc** hides the card until that line's text changes. It has to reach the card without taking Esc from the editor's own uses, such as clearing a selection, so it shouldn't stop propagation unless a card is showing and there's no selection.
- **The hover popup.** The card hides while `diagnostic_popover()` is set.
- **Examples and console entries** show no card, as their spans aren't in the editor's text.
- **Other things that clear errors,** such as New, Open and loading an example, clear the card too.

Check that the card doesn't flicker when typing continues on a line whose error stays the same: it hides on the edit and comes back after the pause, and no more often than that.

## Done when

- Unit tests cover each row of the spec's table, the Esc and hover rules, and choosing between several errors on a line: the cursor in each, just after one, and in none.
- On the private display: typing `REPEAT 4 [FD 50]` at normal speed shows nothing, pausing part-way shows the card, and finishing the line clears it. All five states of the mockup are checked with screenshots, and so is moving the cursor between two errors on one line.
- `cargo test --workspace` and the scripted session pass. The session's `CheckDiagnostics` steps will need to wait for the pause on the line being typed.

## Comments

2026-11-04: Done. The rules are in `crates/pendown/src/error_display.rs`, with a test for each row of the spec's table, Esc and several errors on a line. The hover rule is in `error_card.rs`, as the popup is the editor's state.

Looked at on the private display, with screenshots: typing `REPEAT 4` shows nothing, pausing shows the card, moving off leaves the underline and a dot, moving back shows it at once. Esc hides it through moving away and back, and it comes back when the line changes. A run's error shows wherever the cursor is, and scrolled the program back to it. Hovering hides the card. Moving between `FOO` and `BAR` in `REPEAT 4 [FOO] BAR` switches at once.

**Different from the issue:**

- **Esc doesn't wait for a selection to clear.** The editor's Esc doesn't clear a selection, so that rule would mean Esc did nothing with text selected. It hides the card whatever's selected. With several cursors, the first Esc now hides the card and the second collapses them.
- **Pinned, with the cursor on another error.** The issue says the card shows on the pinned line "wherever the cursor is". Moving onto another error's line shows that one, as the learner went there, and the pinned one comes back after. Worth a look in use.
- **Two lines with the same text share Esc,** as the hidden lines are found by their text, which also follows them through edits to other lines.

**Worth noticing in use:** an unfinished block isn't held back. While typing a procedure's body, `SQ has no END` underlines the `TO` line, with a dot, as errors on other lines show at once. That's what the spec says, but it may turn out to be noisy.

I checked flicker by screenshots at a few moments, not frame by frame. It doesn't re-show while typing, as only the pause timer ends the hold.

2026-11-04: Changed after trying it, as the maintainer found the card and the old red popup fought each other. The card is now the only popup, and shows on hover too. This replaces the rule above that the card hides while the popup is up, and the Markdown in `Diagnostic` from issue 05 is gone, as nothing reads it now.

- The editor can't turn its popup off. `error_card.rs` reads and clears it after each mouse move, using it to learn which error the pointer's on. It's never drawn, as that all happens before the next frame.
- The card shows after 300 ms on the underline. It stays while the pointer is on the line, or the card, or crossing the gap, so Help can be reached.
- Hover wins over the cursor's error and a pinned one, which come back when the pointer leaves. Esc hides a hovered card until the pointer leaves and returns.

Tested in `error_display.rs` and with xdotool on the private display. The scripted session has no step that moves the mouse over an underline, so hover isn't in it.

2026-11-04: The card has a close button at its top right, at the maintainer's request, which does what Esc does. A message with no card has none, so Esc is the only way to hide that.

