# The new "Rules for growing a line"

Status: resolved
Type: task
Blocked by: 01, 02, 03

## What

Rewrite "Rules for growing a line" in `content/tutorial/08-exploring-l-systems.md` as the spec and the mockup's "Proposed" board say, and add the comments to "The Koch curve".

- **The program:** the rule as Kame's instructions, one letter on each line, with `; F`, `; -` and `; ++` comments lined up.
- **Three pictures, in Logo,** with captions:
  - The bump, its lines `orange`, `deepskyblue`, `limegreen` and `hotpink`, with grey dashes at the three corners where Kame would have gone straight on.
  - A line above the bump, with dotted guides down from the line's thirds to the bump's corners.
  - Level 1 above level 2, each level 2 bump in its level 1 line's colour.
- **A text block** laying out the level 2 rule: four copies of `F-F++F-F`, the turns between them on their own lines, and which line each came from.
- **KOCH's comments** in "The Koch curve", on both programs that define it, and the paragraph after.
- **The session and the gallery:** the new program comes first in the lesson, so the programs after it move along one. The gallery uses names, so it's fine; the scripted session's `RunLesson(7, 3)` (the gasket) needs its new index. Check `program_named` finds every gallery entry.
- **Pictures:** `UPDATE_PICTURES=1 cargo test -p pendown --test examples`, then look at the new and changed ones.

## Done when

- All tests pass, and the full scripted session reports 0 checks failed.
- Screenshots of the section in the panel match the mockup: the pictures, captions, text block.

## Comments

2026-11-01: Done, as the mockup. The session now finds the gasket by name (`program_named("gasket")`), so new programs earlier in the lesson don't move it. Lessons scroll with a handle each now (`lesson_scrolls`), so the session can scroll them: `ScrollLesson`, and four screenshots of the section. Full session: 0 checks failed.
