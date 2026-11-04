# ADR, README and the scripted session

Status: resolved
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

2026-11-04: Everything's done but the maintainer trying it, so this is `ready-for-human`.

- **ADR 0033** is written, with a dated note on ADR 0005. ADR 0001 keeps the old wording as a record, as 0033 says.
- **`GLOSSARY.md`** doesn't exist, and "error card" reads fine without one, so nothing's added.
- **README:** the features list and the session's list.
- **`content/`:** nothing quotes an old message or says to hover. The one match, "not enough inputs to max" in `reference.kdl`, is the manual's own example of `THROW "ERROR`.
- **The scripted session** covers the pinned card, the wait, the card's syntax and example, moving away and back, Esc, an edit bringing it back, and a suggestion. On the private Xvfb display the only failure is "frame intervals" on the 131,070-move tree, as in the earlier issues.

`cargo fmt --all`, the mingw check and `cargo test --workspace` pass.

**Left for the maintainer:** try it, and take another look at the decision to open the card by itself. Issue 07's comment lists things worth noticing in use.

2026-11-04: The maintainer tried it, after the hover popup was replaced by the card and a close button added, and says it looks good. The card keeps opening by itself.

