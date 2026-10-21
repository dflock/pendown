# The console

Status: ready-for-agent
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
