# Showcase programs, and choosing the gallery

Status: resolved
Type: task
Blocked by: 01

## What

**Showcase programs:** three to five new programs in `examples/gallery/`, written to look finished and inviting, each a picture a learner would want to make, using only what the tutorial teaches by lesson 7, with procedures and comments a learner can read. For instance:
- a garden at night (new: `examples/phase-1/garden.logo` stays as the phase 1 test it is);
- a snowflake or a fern, by recursion;
- a city skyline of filled rectangles;
- a spirograph of circles.

Each is run by the pictures test, with its picture saved.

**Choosing:** about 30–40 entries in `gallery.kdl`, in sections, from:
- the help examples that draw a picture worth a thumbnail (candidates: "A flower of squares", "A spiral", "Petals", "Rainbow", "Filled circles", "See-through", "A random colour each time", "A procedure that calls itself", "Scattered stars", "A sine wave");
- the lessons' finished programs, not the buggy ones in "Help Kame fix the bug";
- the showcase programs, first.

Choosing is the user's call: a draft list, then the user's say.

## Done when

- The showcase programs are in, with their pictures, and the pictures test runs them.
- `gallery.kdl` has the agreed list, and issue 01's test passes.

## Comments

2026-10-31: Five showcase programs in `examples/gallery/`: a snowflake (recursion), a spirograph (REPEAT, a list of colours), a tree in blossom (recursion, filled circles), a city at night (procedures with inputs, filled shapes), and a garden at night (ELLIPSE petals). All are deterministic, with no RANDOM, so their saved pictures stay the same. They use what the lessons teach, plus `REMAINDER`, `HOME` and `ELLIPSE` petals, which the help explains. `pendown-cli`'s `gallery_programs` test runs them and checks their pictures (`tests/pictures/gallery-*.png`).

Draft list: `gallery.kdl` at the root, 33 entries in 7 sections, each checked by script against `pendown-logo.kdl` and the lessons (no example that fails on purpose, no lesson picture, no buggy "Help Kame fix the bug" program). Every thumbnail is on the gallery canvas's "Draft list" board. Waiting for the user's say.

2026-10-31: The user's say: keep the spirals; FOR's "Bigger and bigger squares" now changes colour each square (`SETPC :size / 20`, in `pendown-logo.kdl`, with its saved picture updated); the house moves into "Shapes inside shapes". A Halloween section, after "Made for the gallery", with five new programs: a spider's web, a friendly ghost, a storm (lightning over a bare tree), a jack-o'-lantern and a witch's cauldron. Bats were tried and dropped: without a fill for a drawn shape (`FILLED` isn't implemented), ovals made them look like butterflies. 38 entries.

2026-10-31: The user approved the list of 38.
