# Errors that explain themselves

## Why

A red underline tells a learner something is wrong, but they have to know to hover to find out what. The message they then get often doesn't help: "Not enough inputs to REPEAT" doesn't say how many inputs, or what they look like.

Mockup and proposal: https://claude.ai/artifact/LiTVwPjibe2Yt3P8fFmcRn

## What done looks like

### In the editor

When the cursor rests on a line with an error, the message shows at the end of that line. A card under the line shows how the command is written and an example that works, with a "Help on REPEAT" button.

| Situation | What shows |
|---|---|
| Typing on the line | Nothing new until a pause of about a second, not even the underline. Other lines' errors stay as they are |
| Cursor resting on the line | Underline, message at the end of the line, and the card |
| Cursor on another line | Underline, and a red dot at the end of the line. Hovering shows the same content as the card |
| A run stops on an error | The card shows on that line wherever the cursor is, and the line scrolls into view, until the next edit or run |
| Esc | Hides the card until that line is edited. The underline stays |
| An error in a console entry or a help example | The console only, as now |

- The card flips above the line when there isn't room below.
- When the message doesn't fit beside the line, it moves into the card as its first line.
- With several errors on a line, the message and card show the error the cursor is in, and switch at once when the cursor moves into another. With the cursor in none of them, they show the first. The cursor counts as in an error when it's anywhere in the underlined text, or just after it.
- The card hides while the editor's own hover popup is up, so the two never show together.

### Messages

The same message goes in the console, the card and the hover. Messages stay one short sentence, because programs can see them through `CATCH "ERROR` (ADR 0005). The syntax and example are kept apart from the message.

| Program | Now | New |
|---|---|---|
| `REPEAT 4` | Not enough inputs to REPEAT | REPEAT needs 2 inputs, but got 1 |
| `REPEAT [FD 50]` | Not enough inputs to REPEAT | REPEAT needs 2 inputs, but got 1 |
| `SQ` (from `TO SQ :n`) | Not enough inputs to SQ | SQ needs 1 input, but got none |
| `FD "x` | FD doesn't like x as input | FD needs a number, but got the word x |
| `RT [90]` | RT doesn't like [90] as input | RT needs a number, but got the list [90] |
| `REPEAT 4 FD 50` | FD didn't output to REPEAT | REPEAT needs a list of instructions here. Put them in [ ] |
| `FORWARD 10 20` | You don't say what to do with 20 | Nothing uses 20. FORWARD takes 1 input |
| `FD100` | I don't know how to FD100 | I don't know how to FD100. Did you mean FD 100? |
| `FORWRD 10` | I don't know how to FORWRD | I don't know how to FORWRD. Did you mean FORWARD? |
| `FD :size` | :size has no value | :size has no value yet. Give it one with MAKE "size 100 |
| `TO SQ` with no `END` | SQ has no END | SQ has no END. Add END after its last line |
| `REPEAT 4 [FD 50]]` | Unexpected ] | This ] has no matching [ |

### What the card says

- **Syntax:** the `eg` line for the name the learner typed, so `FD distance`, not `FORWARD dist`, with the manual's input names turned into plain ones (`num` to `number`, `instructionlist` to `[instructions]`).
- **Example:** the command's new `try` field in `content/reference.kdl`. A command with no `try` shows its syntax alone.
- **A learner's own procedure:** the syntax from its `TO` line, `SQ :n`, and "Defined on line 1", with no Help button.

The console prints the syntax and example under the error too, as the "After Run" state of the mockup shows.

## Decisions

Agreed with the maintainer on 2026-11-04:

- **The card opens by itself after a pause.** The learner shouldn't have to know to go looking. If it gets in the way in use, opening it with a key is an easy change.
- **The underline waits for the pause on the line being typed.** Otherwise every half-typed command gets a red line under it.
- **Messages can stop matching UCBLogo's wording.** ADR 0005 already asks for plain English.
- **Claude drafts the `try` examples and the maintainer skims them,** starting with the commands the tutorial teaches.

Settled by the proposal:

- **An overlay, not a gap between lines.** The editor's only way to open a gap is the ghost text for inline completions, which is tied to Tab-to-accept. So the card covers the lines under it until the learner moves away or presses Esc.
- **Not a message on every error line.** It's noisy with several errors, and the default editor pane is only about 460 px wide.
- **"Did you mean" only for very close matches,** as a wrong guess could mislead a beginner.

## Out of scope

- A button that inserts the example or fixes the line. Learners type it themselves, in keeping with ADR 0027.
- Errors in the console's input line getting a card.
- Warnings or style hints. Only errors, as now.
- `try` examples for every command, in this work.

## Issues

| # | Issue | Blocked by |
|---|---|---|
| 01 | Spike: placing a card over the editor | |
| 02 | How many inputs were needed and given | |
| 03 | What kind of input was wanted, and clearer wording | |
| 04 | The `try` field, with first examples | |
| 05 | Explaining an error, in the console and the hover | 02, 04 |
| 06 | The error card | 01, 05 |
| 07 | When the card and underline show | 06 |
| 08 | Smarter hints | 05 |
| 09 | ADR, README and the scripted session | 03, 07, 08 |

Issues 02, 03, 04 and 08 are each useful without the card, so they can ship first if the spike goes badly.
