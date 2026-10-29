# ADR 0027: the tutorial doesn't grade

Status: done
Type: task

## What

`docs/adr/0027-the-tutorial-doesnt-grade.md`: no scores, ticks, stars or "correct!"; no difficulty labels; no answers hidden until tried; lessons ordered, not rated. Bugs are "bugs" to be "fixed", never "mistakes" or "wrong". Why, from Mindstorms. A check of the lessons as they are (none today).

## Done when

The ADR is written, and the lessons have been read against it.

## Comments

### 2026-10-29: done

`docs/adr/0027-the-tutorial-doesnt-grade.md`. The six lessons have none of the words it rules out (correct, well done, wrong, mistake, easy, hard, advanced, beginners, score, try again), and no checked tasks or hidden answers.

One thing didn't fit: the lesson bar coloured earlier lessons with the theme's success green, which reads as "passed". They're grey now (`muted_foreground`), with the current lesson dark and later ones light, so the bar shows where a lesson is in the order, not what's been done.
