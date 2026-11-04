# When the card and underline show

Status: ready-for-agent
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
