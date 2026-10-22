# Phase 4: watching a program run

Two panels beside the console's output, the repeat counters and the variables, that update as the turtle goes, in step with the highlighted instruction. A value flashes when it changes. See "Phase 4" in `implementation-plan.md`.

There are no new Logo commands.

## What "done" looks like

- **Repeat counters:** every `REPEAT`, `FOREVER` and `FOR` running, and the templates that number their members (`FOREACH`, `MAP` and the rest, which `#` counts), outermost first, as `REPEAT 3 of 36`. The innermost one that `REPCOUNT` would output is marked.
- **Variables:** what the running line can see. The globals, then each procedure in the call chain, outermost first, with its inputs and locals. `FOR`'s variable and a lambda template's inputs show as scopes of their own. A variable that a later scope hides is shown dimmed. Names keep the spelling of their definition (ADR 0004).
- **In step:** at animated speeds, and while paused or stepping, the panels show exactly what the highlighted instruction sees. At Instant they update about once a frame, and may be ahead of the drawing. When a run ends they show what it left: the globals.
- **Colours:** below the variables, the pen, fill and background colours, as they were set, each with a square of the colour (added after the phase, at the user's request).
- **Flashing:** a value, a colour or a counter that changed since the panel last updated flashes, and fades over 0.8 s.
- **Layout:** the console area is split into the console, the counters and the variables, resizable, and each panel scrolls.
- **Cost:** `perf` stays within its baseline, and the budget tests still pass.

## Decisions

- **The interpreter sends the panels' contents as an event,** `Event::Watch`, so they travel with the drawing and the highlight and the animator shows them when it gets there (ADR 0022). The interpreter thread formats the values, since `Value` uses `Rc` and can't cross threads.
- **When it's sent:**
  - with lockstep on (animated speeds, paused, stepping), after the `Instruction` of any instruction that sees something different from the last `Watch`: a line can be a whole loop, as in `REPEAT 36 [FD 10 RT 10]`, so line starts aren't enough;
  - at Instant, when the front end asks, which the GUI does each frame;
  - at the end of each run, with only the globals.
- **Values are shown in short form,** at most 4 lists deep and 20 members wide, so a 300,000-member list costs no more than a short one.
- **Special variables** are hidden while they have their starting values: `LOGOVERSION` always, `CASEIGNOREDP` while it's true.
- **Globals are sorted by name.** A procedure's inputs come first, in order, then its locals as `LOCAL` made them.
- **`FOR` gets a counter,** but `REPCOUNT` and `#` still ignore it, as in UCBLogo.

## How the work is split

| Issue | What | Blocked by |
|---|---|---|
| 01 | Counters with names and totals, and `FOR`'s | |
| 02 | The `Watch` snapshot, and when it's sent | 01 |
| 03 | The animator plays `Watch` in step | 02 |
| 04 | The panels, with flashing and resizing | 03 |
| 05 | Checks in the scripted session, and cost | 04 |
| 06 | Phase 4 check | all others |

## Testing

- **Interpreter tests** of the snapshot: scopes, shadowing, spellings, special variables, counters, `FOR`, templates, and when it's sent.
- **Animator tests** that the panels' contents arrive with the right highlight, stepping instruction by instruction.
- **The scripted session** checks the panels' text at a pause, and takes screenshots.
- **`perf`** and the budget tests.

## Out of scope

- Editing a variable from the panel.
- Showing a whole value too long for the panel: it's shortened, as `PRINT` does with the print limits.
- Property lists, which are phase 7.
