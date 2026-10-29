# Teach Kame

Status: needs-triage
Type: task
Blocked by: 01

## What

- The **Teach Kame** button in the console's title bar, with Lucide's `message-square-plus` icon (`IconName::MessageSquarePlus`: add it to `ExtraIcons` in `icons.rs` if gpui-kit's default set doesn't embed it), enabled when an entry can be taught. The dialog's Teach Kame button has it too.
- A modal dialog, with the tutorial's picture of Kame at the top left beside the title, about 72 × 48 px (`tutorial/kame-turtle.svg`, embedded already as `pendown::tutorial::KAME`); then the entries with tick boxes (All, None, Shift-click for a range); errors shown unticked with their message; definitions shown but not tickable; the name field and its reasons; the highlighted preview; the picture, drawn headless and redrawn when the ticks change.
- Name checks, in `console.rs` so they're unit-tested: one word, not a primitive or alias (from `pendown_logo::reference`), not a procedure already in the program or defined at the console.
- **Teach Kame NAME** appends the `TO … END` to the editor (one undo step there), defines it at the console without recording it as an entry, and prints `Kame knows NAME now. It's at the end of your program.`.

## Done when

The dialog works as in the mockups, the name checks and the generated text have unit tests, and screenshots show it.

## Comments

### 2026-10-29: from issue 01

`run_console` saves a turtle for every console job, and Undo relies on there being one per entry. Defining the taught procedure at the console mustn't add one, since it isn't an entry: give it a job of its own (say `Job::Define`) that doesn't save a turtle.
