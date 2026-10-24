# Mindstorms and Pendown: ideas for making Pendown easier to learn

These are notes from reading Seymour Papert's *Mindstorms* (1980) a section at a time and asking, for each section, what it means for Pendown. Each section has a short summary of what Papert says, how it bears on Pendown as it is now (October 2026: the editor and console, the speed slider, Step line and Step instruction, the counters and variables panels, help, Kame's six tutorial lessons, colours and shapes, and errors that come with a call chain), and the ideas it suggests.

The ideas are gathered and ranked at the end, in [The ideas, ranked](#the-ideas-ranked).

A lot of what Papert asks for, Pendown already does. Watching the turtle think, with the running instruction highlighted, the loop counters and each procedure's variables, is a better version of what Papert meant by letting children "get into" a program and see why it did what it did. Most of the ideas below build on that.

---

## Foreword: The Gears of My Childhood

**What Papert says.** As a small child Papert fell in love with gears. He could *be* the gear and feel how it turned, and gears later carried multiplication tables and simultaneous equations into his head. His "fundamental fact about learning" is: *anything is easy if you can assimilate it to your collection of models.* Learning has feelings in it as well as thinking. Nobody told him to learn about gears.

**How it applies.** Kame is Pendown's gear: something you can be, that also carries mathematics. What matters is that a learner can connect what Kame does to something they already know, and that they come to like it, not just that they understand it.

**Ideas.**
- Help the learner bring in models they already have. The tutorial does this with walking ("Kame walks up and draws a line"). It could say so outright, and more often: "stand up and walk it", "you're Kame: which way is your right?"
- Kame should stay a character you can be fond of, not just an arrow. The SVG in the tutorial is a good start. See also *Kame's eye view* under Chapter 3.

---

## Introduction: Computers for Children

**What Papert says.** In most schools the computer programs the child, through drill and practice. In Logo the child programs the computer. Learning to program should be like learning French by living in France ("Mathland"), not like classroom French. The first experience is typing `FORWARD 100` and `RIGHT 90` and exploring what the numbers mean. Programming is introduced as *teaching the turtle a new word*. Figure 1 shows a HOUSE program going through its bugs (the triangle inside the square, then the house on its side) before it works.

**How it applies.** Pendown is on the right side of this split: it's a place to make things, not a quiz. The tutorial already says "teach Kame new words". The risk is that the tutorial turns into a course with steps to complete; Papert would say the lessons are only there to get people started.

**Ideas.**
- **Keep the tutorial free of grading.** No scores, ticks, stars or "correct!" for the *Try this* tasks. The tutorial today has none; that's worth writing down as a principle (an ADR, perhaps), so nobody adds them later.
- **Show the HOUSE bug on purpose.** Figure 1's sequence (plan, bug, compromise, house) is the best short story about programming there is. A tutorial page could load `HOUSE` built from `BOX` and a `TRIANGLE` that turns right, so the roof is drawn inside the box, and ask "what went wrong?" before fixing it. See *A lesson on bugs* under Chapter 4.

---

## Chapter 1: Computers and Computer Cultures

**What Papert says.**
- The computer can make the formal concrete. Nested loops, for example, make systematic counting ("all pairs of beads") obvious, and the double-counting bug becomes something you can name.
- The question to ask of a program is not whether it's right or wrong, but whether it's *fixable*. A culture of debugging makes people less afraid of being wrong.
- Thinking deliberately like a computer, step by step and literally, is a style you can choose, and choosing it teaches you that there are styles of thinking.
- Writing on a computer means a first draft doesn't have to be the last: rewriting is cheap, so children learn to revise.
- QWERTY: early, primitive choices dig in and get rationalised later. BASIC is his example.

**How it applies.**
- Pendown's Step line and Step instruction *are* "thinking like a computer, step by step", made visible.
- The word-processor point is about revising: in Pendown, the editor and console make revising cheap, but nothing helps people come back to an earlier draft.
- On QWERTY: Pendown speaks UCBLogo because it's the most widely used Logo, and marks obsolete commands with reasons. That's the right side of the QWERTY argument: it keeps what's good and is explicit about what it drops. Papert would also approve of Pendown's extensions (colour names, `ARC` that walks): Logo was never meant to be final.

**Ideas.**
- **Name the bug, not the mistake.** Wherever Pendown talks to the learner (errors, tutorial, help), use the language of bugs and fixing. Errors already say "I don't know how to …" in Logo's friendly first person, with a call chain. The error panel could add a line such as "Step up to here to see what Kame was doing", with a button that runs to the error at slow speed. That turns an error into the start of debugging rather than a verdict.
- **Drafts you can go back to.** A light history of runs (code and picture, kept automatically for the session) lets people try things boldly, knowing they can go back. It also helps with Papert's later point about keeping a copy of an interesting bug (Chapter 3, *Serendipity*).
- **A nested-loop example that isn't drawing.** The beads example (all pairs of colours) is a good one for the lists chapter of help: `FOREACH` inside `FOREACH`, printing pairs, with the double-counting bug and its fix.

---

## Chapter 2: Mathophobia: The Fear of Learning

**What Papert says.** Children learn to be afraid of learning. People take one bad experience as a fact about themselves: "I'm not a maths person". Jim, a very verbal child, hated arithmetic because he had no words for it. Jenny understood nouns and verbs at last when she needed them, to make the computer write poetry. Good learning environments follow three principles:
- **Continuity:** new mathematics should carry on from what the learner already knows and values.
- **Power:** it should let them do things they care about that they couldn't do without it.
- **Cultural resonance:** it should make sense in the wider world, to adults too.

**How it applies.** Pendown's whole tutorial is about drawing. That's a strong way in for people who think visually, but Jim and Jenny show that some people come in through words. Pendown has the whole core language, words and lists included, so it can offer that too.

**Ideas.**
- **A second way in: Kame writes poems.** A tutorial lesson (or a help example set) where Kame makes random sentences, Jenny-style, with `PICK`, `SENTENCE` and lists of nouns, verbs and adjectives. It needs no geometry, it gives a reason for categories of words, and it's funny. Later the poem can be written on the canvas with `LABEL` (phase 7) or along a path.
- **Use the three principles to judge new lessons and features.** For each new lesson: what does it carry on from (continuity), what can the learner now make that they couldn't before (power), and would an adult want to do it too (resonance)? A one-line note at the top of each tutorial file would keep this honest.
- **No "easy" or "hard" labels** on lessons or examples. They feed exactly the self-image Papert describes. Order them, but don't rate them.

---

## Chapter 3: Turtle Geometry: A Mathematics Made for Learning

This is the chapter closest to Pendown, so it has the most ideas.

**What Papert says.**
- The turtle is a point with a *heading*, which you can identify with. You learn turtle geometry by *playing turtle*: walking the shape yourself and describing what you did.
- Figure 3 is a real transcript of a child finding the angle for a square by trial: `RT 100`, `ERASE 1` (undo that), `LT 10`, `LT 10`, and so on. Exploring what the numbers mean is the point.
- A circle is "forward a little, turn a little, and keep doing it". That's a differential equation, in a form a child can own.
- Some procedures leave the turtle as they found it. That clean style makes them easy to reuse, and teaches the idea of *state*.
- The **Total Turtle Trip Theorem**: go round any closed shape and you turn 360° in all. It's a tool, not a fact to memorise: it tells you the turn for a triangle is 120.
- Variables come as "the amount I can't tell you now"; the `SPI` spiral needs one. Recursion excites children more than any other idea. Turning angles near 90 in `SPI` gives surprising galaxy-shaped spirals that nobody programmed.
- Then a long conversation between two children making a flower: plan, find resources (a `QCIRCLE` from last week), try, bug (it's a bird!), another bug (it's a fish!), *maths to the rescue* (the Total Turtle Trip Theorem), a working `PETAL`, a `FLOWER`, a `PLANT`, `SLIDE`, a garden with `RANDOM`, a `FLOCK` that comes out as a pinwheel by accident ("if we debug it, we should keep a copy like this"), and finally "how can we make them fly?"
- The chapter ends: the printed page can't show the motion, the bugs or the discoveries.

**How it applies.**
- Playing turtle is hard when Kame is facing down the screen: right and left look swapped. Pendown shows Kame and its heading but doesn't help you take Kame's point of view.
- Figure 3's fiddling (try a number, undo it, try another) is how people find out what 90 means. In Pendown you'd do that at the console, but there's no undo there, and each change to the editor means a full Run.
- The tutorial already uses 360 divided by the number of sides, and says Kame "ends where it started, facing up again". It doesn't name the idea, and nothing on screen shows the total turn.
- The tutorial's `TREE` stresses walking back to where it started. That's Papert's clean style; Pendown could show it.
- Lesson 5 starts recursion with `IF … [STOP]`. Papert starts with the version that never ends, which is the exciting part. Pendown's tail calls (ADR 0007) and Stop button make a never-ending `SPI` safe to run.
- Pendown has `ARC`, so the flower conversation can be followed almost exactly, with `ARC 90 :size` as `QCIRCLE`.

**Ideas.**
- **Undo at the console.** An "undo last instruction" for the console (Ctrl+Z when it has focus, or `UNDO` as an extension) that takes back the last thing typed: its marks and Kame's move. This is Figure 3's `ERASE 1`, and it makes trial and error cheap. It needs the console to keep Kame's state and a mark count before each entry, which fits ADR 0006's console model.
- **Teach Kame what you just did.** After fiddling at the console, select some history lines (or "everything since the last Run") and choose *Make a procedure*: Pendown asks for a name and puts `TO name … END` in the editor. This is the book's central metaphor, made into a button: you do it, then teach Kame the word.
- **Drive Kame with the keyboard.** A mode where the arrow keys move and turn Kame by small fixed steps (Up = `FD 10`, Left = `LT 15`, …), with each key press typed into the console as the instruction it stands for. This is the floor turtle's button box. Learners find shapes with their hands, see the Logo for each move, and then use *Make a procedure* on it.
- **Kame's eye view.** An option that turns the canvas so Kame always faces up while it's paused or stepping, so "right" is always to the right. Or, less drastically, show a small Kame in a corner that always faces up, with arrows labelled *left* and *right*. This is for playing turtle when you can't stand up and walk.
- **Show the turn.** When `RT` or `LT` animates, draw a faint arc of the angle with its number, like a protractor, and fade it. Seeing 90 as a quarter, 120 as a third and 45 as half of 90 is how the numbers get their meaning.
- **A running total of turning.** In the variables panel, beside each procedure in the call chain, show how far Kame has turned and where it is relative to where that call started, for example `BOX: turned 360°, back where it started`. That's the Total Turtle Trip Theorem on screen, and it shows at once which procedures are *state-transparent* (Papert's clean style). A procedure that doesn't come back could be marked `turned 270°, 100 steps from where it started`, which is often the bug.
- **Name the powerful ideas.** Give the tutorial a few named ideas the learner can carry about: *play Kame*, *the total trip is 360*, *leave Kame where you found it*, *bugs are for fixing*. Each could be a help page under its own chapter (*Kame's big ideas*), linked from the lessons. Papert's point is that a named idea becomes a tool you reach for.
- **Start recursion with the version that never ends.** Lesson 5 could open with `SPI` and no `STOP`, run at a slow speed, with "press Stop when you've seen enough". Then add `IF … [STOP]` when the learner wants it to stop by itself. Then the galaxies: `SPI` with angles of 89, 91, 92. Each one is surprising.
- **A project lesson: the flower and the flock.** Turn the children's conversation into a tutorial lesson, bugs and all: `QCIRCLE` (as `ARC`), the bird, the fish, working out the turn with the total trip, `PETAL`, `FLOWER`, `PLANT`, `SLIDE`, a `GARDEN` with `RANDOM`, and the accidental pinwheel flock. It shows what the six lessons don't: how a real project goes, with planning, reuse and bugs that turn out to be good. It should end open: "how could you make them fly?"
- **Keep a copy of a good bug.** A *Keep this* button that saves the current program and its picture to a gallery (a folder of `.logo` files with PNG thumbnails, say) without leaving the editor. It answers "if we debug it, we should keep a copy like this", and makes it safe to keep changing things.
- **Share the process, not just the picture.** The chapter ends by regretting that a page can't show motion. Pendown can: export a run as an animated SVG or GIF, drawn at the current speed, so a learner can share how the picture was made. Export already writes SVG from the marks (ADR 0024); an animated version could add timing from the animator.

---

## Chapter 4: Languages for Computers and for People

**What Papert says.**
- Some knowledge is said to be beyond words, like riding a bike. Papert disagrees: learning to describe a process in the right language helps you learn it.
- Keith wrote his stick figure as one long list of moves and couldn't find the bug. Broken into `VEE`, `LINE` and `HEAD`, each small enough to understand, it was easy. Robert: "all my procedures are mind-sized bites".
- Juggling can be learned in half an hour by treating it as a program with subprocedures (`TOSSLEFT`, `TOSSRIGHT`) and *when demons* (`WHEN TOPRIGHT TOSSRIGHT`), and debugging each part on its own.
- Children often resist debugging: when a program doesn't work they erase it and start again. School has taught them that errors are bad.
- When the teacher genuinely doesn't know how to fix the bug, child and teacher are doing research together.
- Deborah was frightened by the freedom of the turtle. She gave herself only one turn, `RIGHT 30`, built her own small world inside it, and grew confident there.

**How it applies.**
- Pendown already helps with Keith's problem more than 1970s Logo did: you can step through a long program and watch which line drew which line. But you can't test one procedure on its own without editing the top-level code, and you can't easily turn a run of lines into a procedure.
- Phase 6's `BUTTONACT` and `KEYACT` are when demons.
- Pendown has nothing like Deborah's constraint, and its help, with every command a click away, is the opposite of it.

**Ideas.**
- **A lesson on bugs.** A tutorial lesson that's all about fixing: the triangle inside the house, the hexagon that was meant to be a triangle (`REPEAT 3 [FD 100 RT 60]`), a `TREE` that doesn't walk back, a long Keith-style program with one wrong turn. Each comes with a method: step through it, play Kame, find the first line where Kame does something you didn't expect. Pendown's stepping makes this lesson better than anything Papert had.
- **Click a line in the drawing to find its code.** Marks could carry the span of the instruction that drew them (and the call chain, or just the outermost call in the editor). Clicking a line on the canvas then highlights the instruction in the editor. This answers "which bit of code drew *that*?", the first question in debugging a drawing, and links picture to code in the other direction from the highlight. It's the single most useful debugging aid this chapter suggests.
- **Try one procedure on its own.** A small button in the editor's gutter beside each `TO` line that runs just that procedure, with a fresh turtle, asking for its inputs if it has any. That's how you debug mind-sized bites one at a time.
- **Make a procedure from selected lines.** The editor's version of *Teach Kame what you just did*: select some lines, choose *Make a procedure*, name it, and Pendown wraps them in `TO … END` and puts a call where they were. If the lines use variables from round about, they become inputs. This is the step Keith needed, and Papert says children reach for it themselves once they're stuck ("I guess I should turn it into subprocedures?").
- **An outline of procedures.** A list of the procedures in the editor, and which ones use which, so a program can be "viewed from the top", as Robert put it.
- **Microworlds you choose.** Let a learner, or a teacher, or a tutorial page, limit Kame to a few commands, such as `FD`, `BK` and `RT 30` only. Kame politely refuses anything else ("Today I only know FD, BK and RT 30"). It's Deborah's idea, and it makes good puzzles: draw a square with only `RT 30`; draw a circle with no `RT` bigger than 5. It could be a directive at the top of a program (`; Kame knows: FD BK RT`) or a setting on a tutorial page.
- **Call them demons.** When phase 6 brings `BUTTONACT` and `KEYACT`, the help can call them demons, as Papert did, and show the juggling program as an example of how a few demons make something that keeps going by itself.

---

## Chapter 5: Microworlds: Incubators for Knowledge

**What Papert says.**
- Two rules of learning: relate the new thing to something you know, and make it your own by doing something with it.
- Newton's laws are hard because nothing in daily life behaves like them. So Papert builds a ladder of turtle microworlds: the geometry turtle (`FD` changes position, `RT` changes heading), then a *velocity turtle* (`SETVELOCITY`), then an *acceleration turtle* (it can only change its velocity), then a Newtonian one. Learners can invent their own laws of motion.
- Several turtles at once: `TELL TURTLE 4 SETVELOCITY 20`. *Mirror turtles*: two turtles where one's right is the other's left make kaleidoscopes.
- "Wrong" theories are how children learn to build theories. Microworlds are places where right and wrong aren't the point, and where the product is something a teacher can be genuinely excited by. A child who drew a circle with tiny steps will argue that a circle *is* a polygon.

**How it applies.** Pendown is one microworld: the geometry turtle. Its roadmap (phase 6, interactive programs; phase 7, advanced) doesn't yet include more turtles or turtles that move by themselves. These are the "how can we make them fly?" of Chapter 3.

**Ideas.**
- **More than one turtle.** Numbered turtles and a way to talk to one, such as `TELL` (an extension: UCBLogo has one turtle, but other Logos have several, like MSWLogo's `SETTURTLE`). The counters and variables panels already show what a program sees; they'd need to show which turtle is being told. This is a phase-7-sized feature, but it opens a lot: flocks, races, mirror turtles.
- **Mirror turtles.** A built-in pair, or a `MIRROR` mode, where every turn is copied to a second turtle the other way round. It's the easiest route to kaleidoscopes, and teaches symmetry by making it.
- **Turtles that move by themselves.** A Dynaturtle microworld as an extension, or as a library written in Logo that ships with Pendown: `SETVELOCITY`, `CHANGEVELOCITY` (a kick), and a clock that moves every turtle each tick. With `KEYACT` from phase 6 this gives the classic Logo game of steering a Dynaturtle onto a target by kicking it, which teaches Newton's first law without saying so.
- **Let people write their own laws of motion.** If the clock calls a procedure the learner writes (`TO TICK`), they can invent worlds where turtles slow down by themselves (Aristotle's), or fall, or bounce. A help page with three or four of these, side by side, is Papert's ladder.
- **Ship microworlds as files.** A microworld is a few procedures plus a page saying what's in it. Pendown could ship a handful (*RIGHT 30 world*, *mirror turtles*, *Dynaturtle*, *Kame writes poems*) to open from the Help panel, alongside the tutorial.

---

## Chapter 6: Powerful Ideas in Mind-Size Bites

**What Papert says.**
- Learning a subject is like getting to know a person or a community, not collecting facts and skills. Good teachers make introductions.
- Formal reasoning is often used to put intuitions down. What learners need is to *debug their intuitions*, not to be told they're wrong.
- The string round the earth: raised six feet, how much longer must it be? Intuition says a lot; the answer is about 40 feet. Think about a square earth, where all the extra is at the corners, then polygons with more sides, then the circle. *When two cases conflict, look for cases in between.*
- Kim hated maths but built a family-tree database, and decided at the end of the year, by herself, that maths wasn't so bad.
- Ken added 35 and 35 and got 610. His trouble was with the procedure, not with numbers: he didn't yet see procedures as things you can name, look at and fix.

**How it applies.** The polygon-to-circle step is already in lesson 2 (`REPEAT 360 [FD 2 RT 1]`). What's missing is a way to slide between the cases and watch.

**Ideas.**
- **Numbers you can drag.** In the editor, hold a modifier and drag a number to change it, with the drawing re-running as you drag (at Instant speed, for short programs). Drag the sides of a `POLYGON` from 3 to 50 and watch it become a circle; drag `SPI`'s angle from 85 to 95 and watch the galaxies appear and vanish. This is "look for cases in between" as a gesture. It's also Figure 3's fiddling, without the typing. Of everything here it's the change that would most change how Pendown feels to use. It needs care with Run's clean slate (ADR 0006) and with programs that take long to run: a time limit, or only for programs that finish quickly at Instant.
- **A "Stuck?" button with methods, not answers.** Papert's teachers didn't give answers; they offered a method. A *Stuck?* item in Help could offer Pólya's questions in Kame's words: *Play Kame: walk it yourself. Have you made something like this before? Can you break it into smaller parts? Try a simpler case: a square before an octagon. Step through it and find the first thing Kame does that you didn't expect.* Each with a button that does that thing (start stepping, open the procedure list, and so on). It doesn't need to understand the program to help.
- **Procedures as things.** Ken's problem is solved by seeing procedures as things you can name and change. A panel of *Kame's words*, listing every procedure the learner has taught Kame, with its inputs and a *try it* button, makes them objects. The console already knows them after a run; a visible list says so.

---

## Chapter 7: LOGO's Roots: Piaget and AI

**What Papert says.**
- Piaget taught that to understand how children learn number, study number. Learning to ride a bike is learning not to unbalance it.
- Minds are societies of simple agents (judge by height, judge by width, judge by history) that argue and are kept in check by other simple agents, not made logically consistent.
- We are all *bricoleurs*: we learn by tinkering, working with what we've got, not by building logically.
- Children can't do combinatorial tasks until 11 or 12, Papert suggests, because our culture is poor in materials for thinking about procedures, not because the task is harder. Procedures, in Logo, can be named, stored, retrieved, changed and combined (note 4).

**How it applies.** The tinkering point is a question for ADR 0006: today a parse error anywhere stops the whole program running. That's tidy, but a bricoleur works with half-finished things.

**Ideas.**
- **Let half-finished programs run.** When there are parse errors, offer to run anyway: define the procedures that parse, and run top-level lines up to the first line that doesn't. The errors still show. Then a learner can try their working `PETAL` while their `GARDEN` is still a mess. This changes ADR 0006, so it wants its own decision.
- **Procedures that can be stored and fetched.** Note 4's list (named, stored, retrieved, changed, combined) suggests a learner's own library: procedures saved once and available in every program, like the children's `QCIRCLE` "from last week". This could be as simple as a folder of `.logo` files whose `TO`s are defined before each Run, with the *Kame's words* panel showing which came from where. It's the "Find resources" step of the flower project.
- **Agents in the help.** Papert talks about each procedure call as an agent with its own inputs ("the SPI man"). The variables panel already shows each call in the chain with its own variables. Lesson 5 says "each call of `SPIRAL` has its own `:length`"; the panel could make that vivid by giving each call a small Kame of its own, or a name such as `SPIRAL #3`.

---

## Chapter 8: Images of the Learning Society

**What Papert says.**
- The Brazilian samba school: novices and experts dancing together, learning from each other, for a real performance. Logo environments are like this when work is shared: pictures on the walls, children changing each other's programs and passing them back.
- Logo makes it easy to talk about how you made something, not only what you made, and its debugging gives children words to ask for help, so anyone can help.
- Educators must be anthropologists: build on what's already happening in the culture.
- Learning parallel skiing became easy when skiing itself was rethought, along with short skis and new boots: content, method and technology changed together.

**How it applies.** Pendown is a desktop app for one person. The culture around it today is GitHub, the web, and screenshots. Sharing is what turns the work into something social.

**Ideas.**
- **Share a program as a link or a single file with its picture.** Export already writes PNG and SVG. A PNG or SVG that carries its own program inside it (as metadata) could be opened by Pendown to get the code back. Then posting a picture *is* sharing the program, and anyone can remix it.
- **A gallery to learn from.** Ship a gallery of programs, made by people of all levels, with *Copy to editor*, as the help panel's examples do now. Include unfinished and buggy ones, with notes on what went wrong.
- **Words for asking for help.** Papert says debugging gives children a language for asking for help. Pendown can help them describe a bug: *Copy bug report* copies the program, the error and its call chain, and the line Kame was on, as text ready to paste to a friend or teacher.
- **Rethink the subject, not just the teaching.** The skiing story is a warning against just adding tutorials. Things like dragging numbers, clicking the drawing to find the code, and keyboard-driven Kame change what learning Logo *is*, not just how it's explained. They should come before more lesson text.

---

## Epilogue: The Mathematical Unconscious

**What Papert says.** Poincaré thought mathematicians are guided by a sense of beauty, and that it's inborn. Papert agrees it's aesthetic but not that it's inborn. People feel real pleasure when an equation turns into a nicer one. Turtle geometry puts "the pretty" at the front: once a child can draw circles, a whole palette of shapes opens up.

**How it applies.** Pendown's colour forms, shapes and `ARC` are aesthetic tools; they matter to learning, not just to looks.

**Ideas.**
- **Lead with beautiful examples.** The first thing a new user sees should be lovely: an example running slowly on the canvas when the app opens with an empty editor, perhaps, with *Load into editor*. The README's flower is a good candidate.
- **Make "pretty" easy to reach.** Small conveniences that make drawings look good with little effort: the CSS colour names already do this. Others might be a `RANDOMCOLOR`, a few named palettes, or pen-width help, each with examples.

---

## Afterword and Notes

**What Papert says.** Logo was designed to be a real language with "easy entry routes", never a toy, and never final. The best learning happens when the learner takes charge. The notes add the touch-sensor turtle: a few lines that follow round any object by staying in touch with it, which children work out by acting it themselves. Note 3 on Chapter 4: children need parallel processes as soon as they try to make things move; one procedure per moving thing is the natural way.

**Ideas.**
- **Help people who are reading Mindstorms.** Many people will come to Pendown having read the book, and type its programs. They'll hit old syntax: `TO SPI DISTANCE` (no colon), `REPEAT 4` followed by an indented body with no brackets, `RIGHT TURN 1`, `PEN UP`, `ERASE 1`, `REPEAT [FORWARD 1 RIGHT 1]` with no count, and `TEST`/`IFTRUE`. Each could get a specific error that says what the book meant and how to write it in Pendown ("Mindstorms-era Logo wrote REPEAT with no count to mean for ever; here that's FOREVER [FD 1 RT 1]"). This is like the obsolete-command reasons Pendown already gives, and costs little.
- **A Mindstorms page in help.** A short help chapter that gives the book's programs (`SQUARE`, `TRIANGLE`, `HOUSE`, `SPI`, `PETAL`, `FLOWER`, `VEE`, `MAN`) in Pendown's Logo, each runnable, and each tested like the other examples.
- **A touch-sensing microworld.** Something to bump into, like walls or shapes, and a way for Kame to feel them, such as a `TOUCHING?` that reports whether Kame's front, left or right is on something already drawn (like `COLORUNDER` in some Logos). The follow-the-wall program is a lovely first taste of feedback.
- **One procedure per moving thing.** If Pendown gets several turtles (Chapter 5), let each run its own procedure at the same time, as Papert asked, rather than making the learner interleave their steps by hand.

---

## The ideas, ranked

Ranked by how much each would help people learn, for the work it takes, given what Pendown already has. *Size* is a rough guess: small (days), medium (a week or two), large (a phase).

### Do first

| Idea | From | Size | Why |
|---|---|---|---|
| **Click a line in the drawing to find its code** | Ch. 4 | medium | Links picture to code in the other direction from the highlight. It's the first question in every debugging session. Marks need to carry spans. |
| **Numbers you can drag in the editor** | Ch. 3, 6 | medium | Turns fiddling with numbers, Papert's way into angles, variables and emergent shapes, into a gesture. Needs care with long programs. |
| **Undo at the console** | Ch. 3 | small–medium | Figure 3's `ERASE 1`. Makes trial and error cheap. |
| **Teach Kame what you just did** (console history → procedure) | Intro, Ch. 3 | small | The book's central metaphor, made into a button. |
| **Running total of turning, and "back where it started"** in the variables panel | Ch. 3 | small | Puts the Total Turtle Trip Theorem and state-transparent procedures on screen. Often points straight at the bug. |
| **A lesson on bugs** and **the flower-and-flock project lesson** | Ch. 3, 4 | small | Shows how real projects go, bugs and all. The tutorial today shows only programs that work. |
| **Mindstorms-syntax hints** in errors, and a Mindstorms help page | Notes | small | People will type the book's programs. |

### Do next

| Idea | From | Size |
|---|---|---|
| Try one procedure on its own, from a button beside its `TO` | Ch. 4 | small |
| Make a procedure from selected editor lines | Ch. 4 | medium |
| Show the turn as an arc with its angle while `RT`/`LT` animates | Ch. 3 | small |
| Drive Kame with the keyboard, recording to the console | Ch. 3 | medium |
| *Stuck?* help with methods, each with a button | Ch. 3, 6 | small |
| *Kame's words* panel: procedures as things you can see and try | Ch. 6, 7 | medium |
| *Keep this*: a gallery of saved programs and pictures, and a history of runs | Ch. 1, 3 | medium |
| Start recursion with the version that never ends; then the galaxies | Ch. 3 | small |
| Kame writes poems: a way in through words | Ch. 2 | small |
| Microworlds you choose: limit Kame to a few commands | Ch. 4 | small–medium |
| Name the big ideas in help (*play Kame*, *the total trip*, *leave Kame where you found it*, *bugs are for fixing*) | Ch. 3 | small |

### Later, or for a new phase

| Idea | From | Size |
|---|---|---|
| Several turtles, `TELL`/`ASK`, each running its own procedure | Ch. 5, Notes | large |
| Mirror turtles | Ch. 5 | medium (with several turtles) |
| Dynaturtles, a clock, and laws of motion you write | Ch. 5 | large |
| Touch sensing, and the follow-the-wall program | Notes | medium |
| Pictures that carry their program; copy a bug report | Ch. 8 | small–medium |
| Animated export of a run | Ch. 3 | medium |
| Run half-finished programs (changes ADR 0006) | Ch. 7 | medium |
| A personal library of procedures available in every program | Ch. 7 | medium |
| Kame's eye view (canvas turns so Kame faces up) | Ch. 3 | medium |

### Principles to keep

These aren't features, but Papert would say they matter more than any of them. They might be worth an ADR.

- **The learner programs Kame; Pendown doesn't program the learner.** No grading, scores, badges or "easy"/"hard" labels. *Try this* stays open-ended.
- **Bugs are for fixing, not mistakes.** All of Pendown's wording (errors, tutorial, help) treats a bug as the normal next step.
- **Methods, not answers.** When Pendown helps, it offers a way to find out (step, play Kame, try a simpler case) rather than the fix.
- **Projects, not exercises.** Lessons end with something the learner can make their own, and leave the next step open.
- **Change what learning Logo is, not just how it's explained.** Prefer features that make exploring cheaper over more lesson text.
