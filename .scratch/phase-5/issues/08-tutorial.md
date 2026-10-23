# The tutorial

Status: done
Type: task
Blocked by: 05, 06

## What

Six lessons in `tutorial/`, as markdown with Kame. The Tutorial tab has a lesson picker, progress, the lesson, and Previous and Next. Code blocks have Load into editor, which asks first if there are unsaved changes, and Run.

## Done when

The lessons read well, their code runs (issue 04's test), and the tab works.

## Comments

### 2026-10-23: done

Six lessons in `tutorial/`, as markdown in the panel's markup: moving and turning, REPEAT, procedures ("Teaching Kame new words"), inputs and variables, recursion, and colours and shapes. They point at the panels, Step and F1 as they go, and avoid naming a procedure SQUARE, which is a command. `pendown::tutorial` embeds them; a lesson's title is its first heading.

The Tutorial tab has "Lesson N of 6", the title, a bar of the lessons to go to any of them (a tooltip names each), the lesson, and Previous and Next named after their lessons. Each program has Load into editor and Run. Load into editor goes through the question about unsaved changes, as New does, and the program becomes a new untitled one, so Save can't write a lesson over the user's file. Run plays it on the canvas like a help example: `Source` is now an example or a lesson's program.

The examples test runs every lesson's programs too (24, as `tutorial-L-P`), and `tests/help.rs` checks their links. The mockup's lesson picker is the bar of lessons instead of a drop-down.

The interface font draws bold as regular here, so a lesson's headings are a size larger too.
