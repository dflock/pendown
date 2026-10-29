# The console's entries, Kame's saved state and Job::Undo

Status: done
Type: task

## What

- `console.rs`: a record of the entries since the last Run, beside the Up/Down history: each entry's text, how it ended (ran, error, stopped), and whether it's been undone. `undo()` marks the newest entry that isn't undone and returns it; `clear()` for Run; `teachable()` lists the entries Teach Kame offers.
- `pendown-logo`: `Interpreter::run_console` pushes a copy of `turtle` onto a stack before it runs; `Interpreter::undo_console` pops it and puts it back; `run_program` clears it.
- `run.rs`: `Job::Undo`, which calls `undo_console` and ends as other jobs do, with a `Watch` of the globals.
- An ADR: "Console undo takes back the drawing and Kame, not variables or procedures", with the reasons in the spec.

## Done when

Unit tests cover the entries record (undo order, undone entries left out of `teachable`, clearing on Run) and the interpreter's saved turtles (pen, colours, heading and position come back; variables and procedures don't change). The ADR is written.

## Comments

### 2026-10-29: done

`console.rs` has `Entry` and `Ending` (`Ran`, `Error(message)`, `Stopped`). `ConsoleInput::submit` records each entry it returns to run, repeats included; `finish` records how the newest ended; `to_undo` is the entry the button will name; `undo` marks it and returns it; `teachable` is the entries not undone; `clear_entries` is for Run, and leaves the history. Nothing in the window calls these yet: that's issue 02.

`Interpreter::run_console` saves the turtle and the background before every entry, even one that doesn't parse, so they stay one-to-one with the entries. `undo_console` puts the newest back (or does nothing if there isn't one), then sends a `Watch` and flushes, as a job does. `run_program` clears them. `Turtle` is now `Clone`. `Job::Undo` runs it on the interpreter's thread.

Tests: `console.rs`'s unit tests; `pendown-logo/tests/console_undo.rs` (position, heading, pen, colours, fill, shown, background and turtle mode come back; one entry at a time; failed and unparsed entries; variables and procedures stay; nothing to undo; the `Watch`); and `undo_runs_on_the_interpreter_thread` in `pendown/tests/session.rs`. ADR 0025 records the decision.
