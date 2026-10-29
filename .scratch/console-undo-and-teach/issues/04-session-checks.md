# Session checks for Undo and Teach Kame

Status: needs-triage
Type: task
Blocked by: 02, 03

## What

Add sections to the scripted session (`autotest.rs`): type entries, undo some (including one that `CLEAN`s and one that errors), check the drawing's marks and Kame's position and heading after each; then teach Kame the rest, check the editor's text, and type the new word at the console.

## Done when

The session passes, and its screenshots are saved with the others.
