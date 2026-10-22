# Interpreter metrics

Status: ready-for-agent
Type: task
Blocked by: 01

## What

In `Control`, which the interpreter and the GUI share: instructions run, time running, time waiting in lockstep for the animator's reply, and time waiting on a full channel. Instructions are published as the program goes, so the overlay can show a live rate even for programs that draw nothing.

## Done when

Tests show the counts for a fixed program, and waits measured when the receiver is slow.
