# The lesson's text

Status: resolved
Type: task
Blocked by: 01

## What

`tutorial/08-exploring-l-systems.md`, titled "Exploring L-systems", in the voice and markup of lessons 1–7: short paragraphs, links to commands in `[BRACKETS]`, `> **Try this:**` notes, and a "Help Kame fix the bug" section.

- Explain an L-system by rewriting `F → F-F++F-F` by hand, once and twice.
- Teach the level way of thinking: level 0 is a line, and each level is the rule drawn with pieces of the level below. Say it plainly, more than once, as it's the idea that makes recursion manageable.
- Point at the tools that help: stepping, **Thinking**, the variables panel showing each call's `:level`.
- Name the programs the gallery will use after their fence's language (` ```logo gasket `).
- End with "What next?", which moves here from lesson 7.

## Done when

- Every `[LINK]` is to an implemented command (the help test checks).
- It reads well aloud to someone who found lesson 6 hard.

## Comments

2026-11-01: Done: `tutorial/08-exploring-l-systems.md`. The level way of thinking is set out as a list after the first Koch program. The gallery's programs are named `gasket`, `arrowhead` and `gosper`. Lesson 7's What next? now points at lesson 8. The rewrite at level 2 wraps in the narrow panel, which reads fine.
