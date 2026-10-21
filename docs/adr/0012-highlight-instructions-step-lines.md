# 12. Highlight the running instruction; Step runs one line

Status: Accepted
Date: 2026-10-18 (recorded later, from commits 8104814 and 1ac2446). Updated 2026-10-21 after spike 04.

## Context

A headline feature is a direct visual link between what the turtle is doing and the code doing it. The plan first highlighted the running line, but `REPEAT 4 [FD 50 RT 90]` is all one line, so a line highlight can't show which `FD` or `RT` is running. Separately, the user wanted Step to go one line of code at a time. UCBLogo's own `STEP` and `TRACE` commands are obsolete here, replaced by GUI controls.

## Decision

- **Before running each instruction, the interpreter sends `Instruction { span, starts_line }`.** `span` covers just that instruction, so the highlight moves `REPEAT` → `FD 50` → `RT 90` and round again. `starts_line` marks the first instruction run from each source line.
- **Running normally,** `Instruction` is an ordinary event that only moves the highlight.
- **Step** runs one source line while paused. Everything that line draws plays at the current speed, with each instruction highlighted, then the program pauses at the start of the next line.
- **Lockstep:** while the GUI sets a shared `lockstep` flag, the interpreter sends a `Sync` after each `Instruction` with `starts_line`, and waits for the animator before running the line. Otherwise the interpreter could already have run lines ahead of the highlight, including effects that aren't events, such as `MAKE`.
  - The flag is set while paused or stepping, **and at every animated speed**, so pausing is exact: the interpreter is already waiting at the next line. Setting it only on Pause would leave the interpreter as far ahead as the channel allows, about 1,000 events.
  - Only Instant, running, leaves it off. Pausing from Instant can leave the interpreter a few hundred lines ahead; the first Step plays those lines, which have already run, and stepping is exact after that. That's accepted, because lockstep at Instant would slow programs with many short lines several times.
- **How Step works:** the animator holds the `Sync` it reaches while paused, instead of replying. Step releases it, plays events at the current speed, and holds the next `Sync`. A `Sync` that follows the line-start `Instruction` the highlight was already on belongs to that line, so Step releases it rather than stopping there.
- **Step instruction** is a second button: it runs to the next instruction, including inside brackets. One-line loops need it, because a one-line `FOREVER [FD 1 RT 1]` never reaches another line.
- **The Pause button only stops the animator.** It isn't Logo's `PAUSE` command, which is obsolete, along with `CONTINUE`. A later "Pause and inspect" could replace it in the GUI.
- **Every parsed instruction carries a source span** from the start, recording which source it came from (ADRs 0001 and 0006).

## Consequences

- Spike 04 answered the open question: a second "Step instruction" button is needed as well as Step.
- Spike 04's tests check after every Step that the interpreter has started exactly as many lines as the highlight has reached.
