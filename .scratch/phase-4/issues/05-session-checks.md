# Checks in the scripted session, and cost

Status: ready-for-agent
Type: task
Blocked by: 04

## What

The scripted session pauses a program with nested loops and a procedure, checks the panels' text, steps, checks again, and takes screenshots. `perf` and the budget tests pass.

## Done when

The new checks pass in release and in debug, and `perf` is within its baseline.
