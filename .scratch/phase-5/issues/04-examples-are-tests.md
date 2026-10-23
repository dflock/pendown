# Examples are tests

Status: ready-for-agent
Type: task
Blocked by: 03

## What

A `fit` option on `pendown-cli`'s renderer fits the turtle area to the drawing. A test runs every example and tutorial code block, fitted at 200 × 200 with seed 1, checks it runs without errors, and compares it with PNGs saved in `crates/pendown/tests/help-pictures/`.

## Done when

The test passes, and `UPDATE_PICTURES=1` saves the pictures.

## Comments
