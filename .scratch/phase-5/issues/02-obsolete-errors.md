# Obsolete commands' errors, and the command an error is about

Status: ready-for-agent
Type: task
Blocked by: 01

## What

Running an obsolete command says "EDIT isn't in Pendown: ..." with its reason, not "I don't know how to EDIT". The ~100 reasons are reworded to read after "X isn't in Pendown: ". `Error::command` names the command an error is about, from the primitives' helpers, the arity errors and unknown commands.

## Done when

Tests show the message for a few obsolete commands, and `command` for the common errors. A test checks every reason reads as a sentence.

## Comments
