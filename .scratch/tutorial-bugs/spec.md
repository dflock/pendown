# Tutorial: no grading, and Help Kame fix the bug

Two ideas from `mindstorms/mindstorm-implementation-plan.md`:

- **No grading.** The tutorial has no scores, ticks, stars or "correct!" for its Try this tasks, and no "easy" or "hard" labels on lessons or examples. Those feed the self-image Papert describes: "I'm bad at maths". Lessons are ordered, but not rated. The tutorial has none of this today; an ADR keeps it that way.
- **Help Kame fix the bug.** A lesson built on Mindstorms Figure 1's sequence (plan, bug, compromise, house): Kame tries to draw a HOUSE from a BOX and a TRIANGLE, but the triangle turns right and the roof ends up inside the box. The lesson shows what Kame meant to draw and the program with the bug, and asks "What went wrong? Can you help Kame fix it?" More buggy programs follow, and each later lesson gets one bug of its own.

Bugs aren't mistakes to be ashamed of: they're how programs get made. The lesson says so, and treats the buggy picture as interesting in its own right (the "compromise": a box with a tent inside is a picture too).

The UI design is on a canvas of mockups: https://claude.ai/artifact/LR591Fduhev9mhaw67PwCZ (private to the owner unless shared).

## What "done" looks like

- **ADR 0027:** the tutorial doesn't grade. No scores, ticks, stars, "correct!", "well done" or "wrong" for anything the learner does; no difficulty labels on lessons, examples or tasks; no answer kept hidden until you've tried. Lessons are in an order, and that's all.
- **Pictures in lessons:** a fenced block marked `picture` shows only the picture of its program, drawn as Help draws example pictures, with a caption, and no code and no Load or Run. It's how a lesson shows what Kame meant to draw without giving the program away.
- **A new lesson 4, "Help Kame fix the bug",** after "Teaching Kame new words", so the lessons after it move up one (7 in all):
  - Kame's plan: a house is a box with a triangle on top. The picture of what Kame meant to draw.
  - The program with the bug: `BOX`, `TRIANGLE` (which turns right), and `HOUSE`, with Load into editor and Run.
  - "What went wrong? Can you help Kame fix it?", with ways to look: Step through it and watch where Kame is when `TRIANGLE` starts; be Kame, and walk it on the floor; turn on Thinking.
  - The compromise: the bug drew something too, a box with a tent inside. Keep it as a word of its own, then fix `HOUSE`.
  - "One way to fix it", at the end, saying there are others: move to the top of the box and turn before the triangle.
  - **More bugs to fix,** each with the picture Kame meant and the program that doesn't draw it:
    - a square that won't close: `REPEAT 4 [FD 100 RT 80]`, and the idea that a shape that closes turns 360 degrees altogether;
    - a star that won't close: `REPEAT 5 [FD 200 RT 140]` instead of 144, turning twice round altogether;
    - a flower with a gap: `REPEAT 12 [BOX RT 20]`, which turns 240 degrees, not 360.
- **One bug in each later lesson,** under "Help Kame fix the bug": inputs (`BOX :size` that always draws 100), recursion (a `TREE` that leans, because it forgets to turn back before it ends), colours (`SETPC` after the line it was meant for).
- The lesson bar, "Lesson N of 7", and Previous and Next follow.

## Decisions

- **Fixes are shown, not hidden.** A hidden answer that appears after trying is a quiz. "One way to fix it" comes after the bug's explanation, as one way among others, so it's there for whoever wants to compare.
- **Words:** "bug" and "fix", as Papert uses them, never "mistake", "wrong" or "error" for what the learner does. The console's error messages are the interpreter's, and stay as they are.
- **`picture` blocks are programs too:** the examples test runs them and keeps their pictures, as `tutorial-L-picture-N`. `Lesson::programs` stays the code blocks with Load and Run, so their numbers don't change.
- **The pictures are 160 pixels,** bigger than Help's 84, since they're the point of the paragraph, with a caption under: "What Kame meant to draw".
- **Renumbering:** the lessons after 3 move up one; their saved pictures are renamed, and the scripted session's lesson numbers change where they need to.
- **Every lesson's program still runs without errors,** so the bug programs are ones that run and draw the wrong thing, not ones that stop with an error or run forever.

## How the work is split

| Issue | What | Blocked by |
|---|---|---|
| 01 | ADR 0027: the tutorial doesn't grade, and the words for bugs | |
| 02 | `picture` blocks in lessons: the markup, the panel, the examples test | |
| 03 | The new lesson 4, "Help Kame fix the bug", and renumbering | 01, 02 |
| 04 | A bug in each later lesson | 03 |
