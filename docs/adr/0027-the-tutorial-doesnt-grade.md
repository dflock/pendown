# 27. The tutorial doesn't grade

Status: Accepted
Date: 2026-10-29

## Context

Pendown's tutorial teaches Logo with Kame, after Papert's Mindstorms. Papert argues that children learn to think of themselves as "bad at maths" from school's way of marking work right or wrong, and that the turtle helps because a program that doesn't work isn't a failure: it has a bug, and bugs can be found and fixed. Debugging is the activity, not a punishment for getting it wrong.

Tutorials elsewhere often add scores, ticks, stars, "Correct!", difficulty labels, or answers locked until the learner has tried. Each of those turns the tutorial back into marking. Pendown's lessons have none of them today, and the plan to add a lesson about bugs (`.scratch/tutorial-bugs/`) makes it worth writing down why.

## Decision

- **No grading of what the learner does:** no scores, points, ticks, stars, badges, streaks or progress percentages, and no "Correct!", "Well done", "Wrong" or "Try again". Pendown never checks a "Try this" task's answer.
- **No difficulty labels:** nothing on a lesson, example or task says "easy", "hard", "advanced" or "for beginners". Lessons are in an order, shown as "Lesson N of M", and that's all.
- **No hidden answers:** where a lesson shows a way to do something, it's there to read, not unlocked by trying. It's offered as one way among others.
- **The words for bugs:** a program that doesn't do what was meant has a **bug**, and finding it is **debugging**, as Papert uses them. Lessons don't call the learner's programs "mistakes", "wrong" or "errors". The interpreter's error messages say what the interpreter couldn't do, and stay as they are.
- **A bug's picture is a picture.** Where it makes sense, a lesson treats what a bug drew as interesting in its own right, as Mindstorms' house whose roof fell inside becomes something else worth keeping.

## Consequences

- New lessons, tasks and examples are written to these rules, and reviews of tutorial changes check them.
- The tutorial can't tell the learner they've finished a task. A lesson ends with what to try next, or the next lesson.
- The lesson bar shows where a lesson is in the order, not how much the learner has done.
