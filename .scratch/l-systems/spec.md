# Lesson 8: Exploring L-systems

## Why

Recursion is hard to think about. Lesson 6 introduces it with three programs (a spiral, squares at the corners, a tree) and a bug to fix, then moves on. Learners need more practice, with pictures that reward it.

L-systems suit this. An L-system is a rule for rewriting a line: "a line becomes a line, a left turn, a line, two right turns, a line, a left turn, a line". Do it again to every line, and again, and a snowflake grows. Each rule becomes a recursive procedure, and "do it again" becomes "call yourself". That gives a way to think about recursion that doesn't need you to follow every call:

- **Level 0 is just a line.**
- **Level 1 is the rule, drawn with level-0 lines.**
- **Level 2 is the rule, drawn with level-1 pieces.** You don't have to work out what level 1 looks like inside; it's already drawn.

## What done looks like

A new lesson, `tutorial/08-exploring-l-systems.md`, the eighth in the Tutorial tab, after "Colours and shapes". Kame teaches, in the voice of the other lessons, with programs that load and run, **Try this** notes, and a **Help Kame fix the bug** section.

1. **What an L-system is.** A start, `F`, and a rule, `F → F-F++F-F`, where `F` is a line, `+` is turn right 60 and `-` is turn left 60. The lesson rewrites it by hand, once and twice, in the text, so you see the rule applied to every `F` at once.
2. **The Koch curve.** The rule as a procedure: `KOCH :size :level`, where level 0 draws a line and any other level draws the rule with four smaller `KOCH`s, a level lower. Then one program draws levels 0 to 3 side by side (with `REPCOUNT`), so you can see each is made of the one before. Step through level 2 and watch the variables panel: each call has its own `:level`.
3. **The Koch snowflake.** Three Koch curves round a triangle. **Try this:** turn the other way, and the bumps point inwards.
4. **The Sierpinski gasket.** A triangle made of three half-size triangles, each made of three smaller ones: like lesson 6's squares at the corners. Here the rule is about shapes, not lines.
5. **The same gasket, another way.** The Sierpinski arrowhead is an L-system with two rules, `A` and `B`, that use each other. So it's two procedures that call each other. It draws the same gasket as one wiggly line, without lifting the pen.
6. **The Gosper curve.** Two rules again, longer ones, with turns of 60 and 120, that fill a shape like a snowflake made of hexagons. Each level is about 2.65 times smaller than the one above (√7, but the lesson needn't say so). Colour makes it striking, from lesson 7.
7. **Help Kame fix the bug.** A Koch curve where one of the four smaller calls is the wrong size, so one bump is lopsided. Comparing levels 1 and 2 shows which call it is.
8. **What next?** Other L-systems to try (the dragon curve, the Hilbert curve), and lesson 7's "What next?" moves here, as this is now the last lesson.

Levels are chosen so each program finishes in a few seconds at the normal speed. Gosper at level 4 draws about 2,400 lines, so its program may start at level 3 and suggest 4 with the speed turned up.

## Decisions

- **Lesson 8, after "Colours and shapes",** not straight after recursion. A lesson 7 would renumber "Colours and shapes", and the gallery refers to lessons by number. After colours, the pictures can use colour. And it reads well as a second go at recursion, once you've had time with the first.
- **Rules become procedures, with a `:level`.** That's the recursion the lesson is for. Rewriting the rule as text happens in the lesson's prose, by hand. A Logo program that rewrites words (with `MAP` and `OUTPUT`) would show the L-system literally, but the lessons don't teach `MAP`, and it would teach words, not recursion. It could go in the gallery later.
- **Only what lessons 1–7 teach.** Procedures with inputs, `IF` and `STOP`, `REPEAT` and `REPCOUNT`, `SETPOS`, colours. No `FOR`, `MAP` or `OUTPUT`.
- **The gallery gets an "L-systems" section** with the lesson's best pictures: the gasket, the arrowhead and the Gosper curve. The Koch snowflake is already in the gallery as `examples/gallery/snowflake.logo`, so the lesson's isn't added twice.
- **No new commands, and no changes to the app** beyond the lesson count.

## Checked already

Drafts of the four drawings run in Pendown (`pendown-cli`) in under 0.05 s each: the Koch snowflake at level 4, the gasket at level 5, the arrowhead at level 6, and Gosper at level 4. Two procedures that call each other work. Placing them on the canvas still needs care: the arrowhead and Gosper curves wander off and wrap if started in the wrong place.

## Out of scope

- Plants with branches (`[` and `]` in L-systems, to save and return to a place). They'd need the tree's "walk back" trick or a stack, and are a lesson of their own.
- Random L-systems.
- A word-rewriting program, for now.

## Issues

| # | Issue | Blocked by |
|---|---|---|
| 01 | The lesson's programs | |
| 02 | The lesson's text | 01 |
| 03 | An eighth lesson: tests, pictures, README and session | 02 |
| 04 | An L-systems section in the gallery | 03 |
