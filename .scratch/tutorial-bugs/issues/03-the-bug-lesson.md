# The lesson "Help Kame fix the bug"

Status: done
Type: task
Blocked by: 01, 02

## What

`tutorial/04-help-kame-fix-the-bug.md`, as the spec describes: the plan, the HOUSE bug, ways to look at it, the compromise, one way to fix it, and three more bugs (a square, a star, a flower), each with a picture of what Kame meant. The later lessons move up one: their files, `SOURCES`, the saved pictures, and the scripted session's lesson numbers.

A picture's program is never shown, so it can start by moving Kame, pen up, so the drawing's middle is at the centre: pictures are fitted around where Kame starts (`Drawing::fit`), and a drawing that goes up and right from there sits in a corner.

## Done when

The lesson reads well against ADR 0027, its programs run without errors and draw what the lesson says, and screenshots show it.

## Comments

### 2026-10-29: done

`tutorial/04-help-kame-fix-the-bug.md`: the plan with the house Kame meant, the HOUSE bug (Load into editor and Run), ways to look (Step, Thinking, being Kame), the tent, one way to fix it, and three more bugs, a square, a star and a flower, each with its picture. The lessons after it are now 05 to 07, with `SOURCES`, their saved pictures and `tests/help.rs` (7 lessons) following. The session's lesson numbers didn't need to change: it only uses lessons 1 and 2.

Pictures centre themselves with a pen-up move first. "One way to fix it" is the whole program, not only `HOUSE` as the mockup shows, so its Load and Run work on their own. The lesson's words were read against ADR 0027: "bug", "fix" and "debugging", no grading, and the fix offered as one way among others.

Checked by drawing every picture and bug to a contact sheet: the house, square and star centred; the roof inside the box; the square and star not closing; the flower with a gap. Session: "Tutorial: Help Kame fix the bug", screenshots `tutorial-bug` and `tutorial-bug-run`.
