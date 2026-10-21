# The console

Status: done
Type: task
Blocked by: 03

## What

The console from ADR 0006: output, and an input line that runs instructions against whatever the last run left.

- **Output:** `PRINT` output, and errors with their call chain.
  - It scrolls to the newest line, and it can be cleared.
  - Text can be selected and copied.
- **The input line:**
  - Enter runs it with `Interpreter::run_console`, using the last run's procedures, variables, turtle and drawing.
  - It's disabled while a program runs.
  - Up and down recall earlier entries.
- **Console entries** are numbered sources (`SourceId::Console(n)`). A highlight or error in a console entry shows in the console, not the editor.
- **`TO` at the console** works, including multi-line definitions: decide how the input line takes several lines.
  - The next Run clears these definitions, and the GUI says so.
  - The console offers "Copy to editor" for them.
- **The interpreter** must live between runs for this, so the thread keeps it, and waits for the next command, instead of exiting after each run. Run starts a fresh one, or clears it (ADR 0006).

## Done when

- After running the Flower example, `SQUARE 50` at the console draws a square on top.
- An error at the console shows there.
- A `TO` typed at the console works, and is gone after the next Run.
- The scripted session covers these.

## Comments

### 2026-10-21: done

- **One interpreter for the session:** `run::Session` is a long-lived interpreter thread that takes `Job::Program` (Run) and `Job::Console` entries one at a time.
  - Each job gets its own channel and `Control`, through a new `Interpreter::set_output`, so Stop ends only that job.
  - Run starts from a clean slate, and console entries use what the last run left (ADR 0006).
  - `run::start` still runs a single program on its own session, for the tests.
- **The console:**
  - Output goes in a read-only text area, so it can be selected and copied. It keeps the newest 2,000 lines and scrolls to the newest, with a Clear button.
  - The input line runs an entry on Enter, and is disabled while anything runs. Up and Down go through its history (`MoveUp`/`MoveDown` are captured around the input).
  - Entries are echoed with `>`.
- **Decided:** a `TO` typed at the console continues over several lines, prompted with "…", until a line with `END` (`console::ConsoleInput`). The one-line form works too.
- **Definitions:**
  - Procedures defined at the console are listed for **Copy to editor**, which appends them to the program.
  - The next Run says "Run cleared the procedures you defined in the console".
- **Console sources:** highlights in a console entry aren't shown in the editor. Errors in a console entry show only in the console, as "In the console: …". An error inside an editor procedure called from the console still marks the code.
- **Tests:**
  - `tests/session.rs` (2 tests): the console using the last run's procedures and variables, `TO` at the console, Run clearing it, and a stopped job not stopping the next;
  - `console` unit tests (4): multi-line entry, history, spotting definitions;
  - the scripted session's console checks, now 25 checks in all, all passing.
