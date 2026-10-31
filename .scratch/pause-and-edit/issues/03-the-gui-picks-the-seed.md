# The GUI picks each Run's seed

Status: resolved
Type: task

## What

`Job::Program` carries the seed for `RANDOM`, and the interpreter uses it for that run (`run_program` with a seed, or `set_seed` per job). Run picks a fresh seed, as the interpreter does now (`Random::fresh_seed`), and the app keeps it with the run, so a replay can send it again. The CLI and tests still choose their own seed as now.

## Done when

A test runs a program that draws with `RANDOM` twice with the same seed and gets the same marks, and Run still gives a different picture each time.

## Comments

2026-10-31: Done. `Job::Program { text, seed }`, and the seed is kept on `Run` as `seed`, so the help panel's saved run keeps it too. `pendown_logo::fresh_seed()` picks one. Help examples still get a fresh seed each, through `run::start`.
