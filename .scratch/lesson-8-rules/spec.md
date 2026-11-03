# Lesson 8: explaining F-F++F-F

## Why

Lesson 8's "Rules for growing a line" names the three letters and then jumps straight to rewriting. It never shows what `F-F++F-F` looks like, or why that bump works: that its turns cancel out, so Kame ends where a line would have, facing the same way, and that its lines are a third as long, so it fits the same space. Those two facts are what make recursion work in the rest of the lesson. The level 2 rule is a run-on string that's hard to read.

Mockup: https://claude.ai/artifact/TR83PzzpXp4NsqFGQnAg9v ("Now", "Proposed", and the Koch curve).

## What done looks like

"Rules for growing a line" explains the rule in three steps:

- **The letters are things Kame does:** a list of `F` is `FD`, `-` is `LT 60`, `+` is `RT 60`, then the rule.
- **Reading the rule.** The rule written out as Kame's instructions, a letter on each line, with the letter beside each as a comment (`FD 100 ; F`, `RT 60 RT 60 ; ++`). It's a program, with Load into editor and Run, so you can Single Step through it. A picture of the bump in four colours, in the order Kame draws them, with grey dashes at each corner where Kame would have gone straight on. Then: `++` is the sharp 120 degree turn; the turns add up to nothing, so Kame finishes facing the way it started, which is why any line can be swapped for a bump. A "Try this" to step through it, saying each letter.
- **The same width.** A picture of a line above the bump, with dotted guides showing the bump fits the same width, and why: each `F` is a third as long, and the middle two lean in to cover the middle third.
- **Rewriting, again and again.** The level 2 rule laid out as four copies of the rule, one per line, with the turns between them kept, and each copy labelled with the line it came from ("the orange line"). A picture of level 1 above level 2 in the same colours: each line has grown its own bump. Then levels, and that each has four times the lines of the last, a third as long.

In "The Koch curve", `KOCH`'s lines carry the same comments (`; F`, `; -`, `; ++`), and a paragraph says that reading down the comments gives the rule.

The pictures are Logo `picture` blocks, drawn with `SETPC` and pen-up dashes, so they're tested like every other picture. They have no labels (the renderer draws no text): colours and captions do that job.

## What the app needs

- **A picture's caption from its fence:** ```` ```picture The four lines, in order ````. Today every picture says "What Kame meant to draw".
- **Wide pictures:** a picture is the shape of its drawing, up to the panel's width, not always a 160 px square.
- **Text blocks:** ```` ```text ```` shows as code, with no Load or Run, and isn't a program.

## Decisions

- **Pictures in Logo, not SVG files.** They stay in the lesson's own language, can't drift from what Logo draws, and are tested. The markup needs no images.
- **Colour instead of labels.** The four lines of the rule are orange, blue, green and pink in every picture and in the laid-out level 2 rule, so a reader can follow one line through the levels.
- **The step-by-step is a program, not a strip of pictures.** Single Step already shows Kame reading the rule a letter at a time.
- **Captions are written in each fence.** The eight existing pictures, all in "Help Kame fix the bug" sections, say "What Kame meant to draw" in theirs, so nothing changes for them.

## Out of scope

- Other lessons' explanations, or diagrams for the gasket, arrowhead and Gosper sections. The same pictures and captions could help there later.
- Labels drawn in pictures (LABEL in the renderer).

## Issues

| # | Issue | Blocked by |
|---|---|---|
| 01 | Picture captions from the fence | |
| 02 | Pictures the shape of their drawing | |
| 03 | Text blocks | |
| 04 | ~~Inline code doesn't break inside itself~~ (wontfix) | |
| 05 | The new "Rules for growing a line" | 01, 02, 03 |
