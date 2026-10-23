# Examples are tests

Status: done
Type: task
Blocked by: 03

## What

A `fit` option on `pendown-cli`'s renderer fits the turtle area to the drawing. A test runs every example and tutorial code block, fitted at 200 × 200 with seed 1, checks it runs without errors, and compares it with PNGs saved in `crates/pendown/tests/help-pictures/`.

## Done when

The test passes, and `UPDATE_PICTURES=1` saves the pictures.

## Comments

### 2026-10-23: done

`Drawing::fit` zooms the turtle area to the smallest square centred on `[0 0]` that holds the drawing, with a margin, from 100 steps up to the usual 1000. `pendown-cli` has it as `Options::fit` and `--fit`. `pendown::help::picture_options` is how the panel and the test draw an example: 200 × 200, fitted, seed 1.

`crates/pendown/tests/examples.rs` runs all 213 examples, checks each runs cleanly (or fails, if it says `fails=#true`), and compares its picture and what it printed with `tests/help-pictures/NAME-N.png` and `.txt`; it also fails on pictures left from examples that have gone. The 349 files are 750 KB. The tutorial's code blocks join the test in issue 08.

Fitting is centred on `[0 0]`, so a drawing off to one side, like FORWARD's staircase, comes out smaller than it could. Panning the picture would mean a centre in `Drawing`'s transform; it isn't worth it for thumbnails.
