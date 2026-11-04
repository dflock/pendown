# ADR, README and the scripted session

Status: ready-for-agent
Type: task
Blocked by: 03, 07, 08

## What

- **ADR 0033:** errors explain themselves. Why an overlay and not a gap between lines, the pause, a run's error being pinned, messages kept apart from syntax and example, and leaving UCBLogo's wording. A dated note on ADR 0005 points to it.
- **`GLOSSARY.md`:** add the card's term, if it's needed to talk about it.
- **README:** whatever it says about errors.
- **Tutorial and help:** search `content/` for text that quotes an old message or says to hover over an error, and update it.
- **The scripted session** (`PENDOWN_AUTOTEST=1`): a step that makes an error, waits, checks the card shows with the expected syntax and example, moves away, and presses Esc.

## Done when

- The session covers the card, and passes.
- `cargo fmt --all`, the mingw check and `cargo test --workspace` pass.
- The maintainer has tried it, since whether it helps or gets in the way only shows in use. The decision to open the card by itself gets another look at this point.

## Comments
