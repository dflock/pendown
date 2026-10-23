# Obsolete commands' errors, and the command an error is about

Status: done
Type: task
Blocked by: 01

## What

Running an obsolete command says "EDIT isn't in Pendown: ..." with its reason, not "I don't know how to EDIT". The ~100 reasons are reworded to read after "X isn't in Pendown: ". `Error::command` names the command an error is about, from the primitives' helpers, the arity errors and unknown commands.

## Done when

Tests show the message for a few obsolete commands, and `command` for the common errors. A test checks every reason reads as a sentence.

## Comments

### 2026-10-23: done

`Error::unknown` gives "EDIT isn't in Pendown: it opens an external text editor, and Pendown has its own." for an obsolete command, named as it was called (ED for ED), and "I don't know how to" otherwise. Parsing and running both use it, so the message shows while typing too. The 39 distinct reasons, shared by 103 commands, are reworded to follow "X isn't in Pendown: ", lower case, ending with a full stop, saying Pendown rather than the GUI; a test checks that.

`Error::command` is set by `Args::error` and `doesnt_like`, colour errors, the arity errors, STOP and OUTPUT outside a procedure, and obsolete commands. A procedure the user wrote can be named too; the GUI only offers help when the reference has a page.
