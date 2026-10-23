# Checks in the scripted session

Status: done
Type: task
Blocked by: 06, 07, 08

## What

The scripted session opens help with F1, searches, runs an example and goes back, opens Help on an error, and loads and runs a lesson, with checks and screenshots.

## Done when

The new checks pass.

## Comments

### 2026-10-23: done

Added with each feature rather than at the end. The scripted session's phase 5 sections: F1 on `FD`, following the cursor to `RT` and to `:size`, search, contents, running FORWARD's example and Back to my drawing, Copy to editor, Help on an error and on an obsolete command, and a lesson loaded and run, with screenshots (`help`, `help-search`, `help-contents`, `example`, `error-help`, `tutorial`). The session has 70 checks. `PENDOWN_AUTOTEST_FROM="Phase 5"` starts it at the first section with that in its title, for working on one part.
