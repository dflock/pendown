# Minsdstorms Improvement Ideas

## Tutorial & Lessons

- Write an ADR, for the tutorial style: Keep the tutorial free of grading. No scores, ticks, stars or “correct!” for the Try this tasks. The tutorial today has none; don't add any. No “easy” or “hard” labels on lessons or examples. They feed exactly the self-image Papert describes. Order the lessons, but don’t rate them.
- Show the HOUSE bug in the tutorial, at an appropriate point. Mindstorms Figure 1’s sequence (plan, bug, compromise, house). A tutorial page could load HOUSE built from BOX and a TRIANGLE that turns right, so the roof is drawn inside the box, and ask “What went wrong? Can you help Kame fix it”. Show the desired picture and the code with the bug. Expand this "Help Kame fix the bug" section with other examples of buggy code to fix.

# Rendering Overlay: Show Kame Thinking

- An option to switch on a "helpful overlay" (maybe "Show Kame thinking"?), to visually build intuition for angles & lengths:
  - Visually show turns. When RT or LT animates, draw a faint arc of the angle with its number, like a protractor, and fade it. Seeing 90 as a quarter, 120 as a third and 45 as half of 90 is how the numbers get their meaning.
  - Visually show lengths. When FD animates, draw a faint ruler with the length so far alongside, and fade it when the next command happens.

## Drive Kame with the keyboard.

- A mode where the arrow keys move and turn Kame by small fixed steps (Up = FD 10, Left = LT 15, …), with each move typed into the console as the instruction it stands for. This is the floor turtle’s button box. Learners find shapes with their hands, see the Logo for each move, and then use Make a procedure on it. While driving, when pressing left/right we need to show a kind of compass heading - maybe a line extending out in front of Kame, with the angle shown visually (a faint arc of the angle with its number, like a protractor), so you can see what angle you're pointing at.

# Console Improvements

- Undo at the console. An “undo last instruction” for the console (Ctrl+Z when it has focus) that takes back the last thing typed: its marks and Kame’s move. This is Minstorms Figure 3’s ERASE 1, and it makes trial and error cheap. It needs the console to keep Kame’s state and a mark count before each entry, which fits ADR 0006’s console model.
- Teach Kame what you just did. After fiddling at the console, select some history lines (or “everything since the last Run”) and choose Make a procedure: Pendown asks for a name and puts TO name … END in the editor. This is the book’s central metaphor, made into a button: you do it, then teach Kame the word.
