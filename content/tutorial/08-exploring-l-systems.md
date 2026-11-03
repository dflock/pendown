# Exploring L-systems

Recursion can be hard to think about. Following every call of a procedure that calls itself, which calls itself, which calls itself, is too much to hold in your head. This lesson gives you another way to think about it, and some beautiful pictures to practise on.

## Rules for growing a line

An **L-system** is a rule for rewriting a line, again and again. It's written in a tiny language of three letters, and each letter is something Kame can do:

- `F` means draw a line: `FD`
- `-` means turn left 60 degrees: `LT 60`
- `+` means turn right 60 degrees: `RT 60`

Here's a rule:

> `F` becomes `F-F++F-F`

### Reading the rule

Read it one letter at a time, as Kame would. Here it is as Kame's instructions, with each letter beside the instruction it stands for:

```logo
PU LT 90 FD 150 RT 180 PD

FD 100        ; F
LT 60         ; -
FD 100        ; F
RT 60 RT 60   ; ++
FD 100        ; F
LT 60         ; -
FD 100        ; F
```

```picture The four lines, in the order Kame draws them: orange, blue, green, pink. The grey dashes are where Kame would have gone without turning.
TO AHEAD
  SETPC "gray SETPENSIZE 2
  REPEAT 4 [FD 6 PU FD 4 PD]
  PU BK 40 PD
  SETPENSIZE 3
END

SETPENSIZE 3
RT 90
SETPC "orange FD 100 AHEAD
LT 60
SETPC "deepskyblue FD 100 AHEAD
RT 120
SETPC "limegreen FD 100 AHEAD
LT 60
SETPC "hotpink FD 100
```

`++` is two right turns together, 120 degrees: the sharp turn at the top of the bump.

Now add up the turns: left 60, right 120, left 60. They cancel out, so Kame finishes facing the way it started, on the same straight line. That's what makes the trick work: any line can be swapped for a bump, and Kame still ends up in the right place, facing the right way.

> **Try this:** run it with **Single Step**, and say each letter as Kame does it.

### The same width

```picture A line, and the rule drawn with lines a third as long.
TO GUIDE
  RT 90 SETPC "gray SETPENSIZE 1.5
  REPEAT 12 [FD 3 PU FD 7 PD]
  PU BK 120 PD LT 90
END

RT 90
PU LT 90 FD 120 RT 90 PD
SETPENSIZE 3 SETPC "white FD 300
PU BK 300 PD
REPEAT 4 [GUIDE PU FD 100 PD]
PU BK 400 RT 90 FD 120 LT 90 PD
SETPENSIZE 3
SETPC "orange FD 100 LT 60
SETPC "deepskyblue FD 100 RT 120
SETPC "limegreen FD 100 LT 60
SETPC "hotpink FD 100
```

Each `F` in the rule is a third as long as the line it replaces. The first and last lines cover the outside thirds. The two in the middle lean in towards each other, so together they cover the middle third. The bump fits in exactly the same space as the line.

### Rewriting, again and again

Now do it again. Every `F` in `F-F++F-F` becomes the whole rule, all at once. The turns stay as they were: only the `F`s change.

```text
F-F++F-F    the orange line
-
F-F++F-F    the blue line
++
F-F++F-F    the green line
-
F-F++F-F    the pink line
```

```picture Level 1, and level 2: each line has grown its own bump, in the same colour.
TO KOCH :size :level
  IF :level = 0 [FD :size STOP]
  KOCH :size / 3 :level - 1
  LT 60
  KOCH :size / 3 :level - 1
  RT 120
  KOCH :size / 3 :level - 1
  LT 60
  KOCH :size / 3 :level - 1
END

TO COLOURED :level
  SETPC "orange KOCH 100 :level LT 60
  SETPC "deepskyblue KOCH 100 :level RT 120
  SETPC "limegreen KOCH 100 :level LT 60
  SETPC "hotpink KOCH 100 :level
END

SETPENSIZE 3
RT 90
PU LT 90 FD 135 RT 90 PD
COLOURED 0
PU BK 300 RT 90 FD 135 LT 90 PD
COLOURED 1
```

Each time you rewrite is a **level**. Level 0 is just `F`, level 1 is the rule once, and level 2 is the rule twice. Every level has four times as many lines as the one before, each a third as long: 4 lines, then 16, then 64.

Writing all those letters out by hand would take a long time. A procedure that calls itself can do the rewriting for you.

## The Koch curve

Here's the rule as a procedure. `:level` says how many times to rewrite. At level 0, Kame draws a line. At any other level, it draws the rule, with four smaller curves, a level lower, where the rule says `F`. Each line has the letter of the rule it stands for beside it.

```logo
TO KOCH :size :level
  IF :level = 0 [FD :size STOP]
  KOCH :size / 3 :level - 1   ; F
  LT 60                       ; -
  KOCH :size / 3 :level - 1   ; F
  RT 120                      ; ++
  KOCH :size / 3 :level - 1   ; F
  LT 60                       ; -
  KOCH :size / 3 :level - 1   ; F
END

PU LT 90 FD 240 RT 180 PD
KOCH 480 2
```

Read down the comments and you get `F-F++F-F`, the rule. Each `F` is a smaller `KOCH`, a third the size and a level lower. Each `-` is `LT 60`, and `++` is `RT 120`, the two turns in one. The first line, `IF :level = 0`, is where the rewriting stops: at level 0, an `F` is just a line.

That is the whole trick, and it's worth saying slowly:

- **Level 0 is just a line.**
- **Level 1 is the rule, drawn with level-0 lines.**
- **Level 2 is the rule, drawn with level-1 pieces.**

When you read `KOCH :size / 3 :level - 1`, you don't have to work out everything it does inside. Trust it: it draws a smaller curve, one level lower, and leaves Kame at its end, facing the same way. Then you only have to think about one level at a time.

This program draws levels 0, 1, 2 and 3, one under another, so you can see that each is made of four copies of the one above. [REPCOUNT] is 1, 2, 3 and 4 as it goes round, so `REPCOUNT - 1` is the level.

```logo
TO KOCH :size :level
  IF :level = 0 [FD :size STOP]
  KOCH :size / 3 :level - 1   ; F
  LT 60                       ; -
  KOCH :size / 3 :level - 1   ; F
  RT 120                      ; ++
  KOCH :size / 3 :level - 1   ; F
  LT 60                       ; -
  KOCH :size / 3 :level - 1   ; F
END

PU FD 330 LT 90 FD 240 RT 180 PD
REPEAT 4 [
  KOCH 480 REPCOUNT - 1
  PU BK 480 RT 90 FD 200 LT 90 PD
]
```

Step through the level 2 curve and watch the variables panel. Each call of `KOCH` has its own `:size` and `:level`, and you can see a level 1 call inside a level 2 call, with level 0 calls inside that.

> **Try this:** the bigger levels draw a lot of lines, so turn the speed up to 9 or 10 to see them sooner. Try level 4 or 5. Can you see the level 3 curves inside?

## The Koch snowflake

Three Koch curves round a triangle make a snowflake.

```logo
TO KOCH :size :level
  IF :level = 0 [FD :size STOP]
  KOCH :size / 3 :level - 1
  LT 60
  KOCH :size / 3 :level - 1
  RT 120
  KOCH :size / 3 :level - 1
  LT 60
  KOCH :size / 3 :level - 1
END

SETBG "midnightblue
SETPC "lightcyan
PU LT 90 FD 210 RT 90 FD 120 RT 90 PD
REPEAT 3 [
  KOCH 420 3
  RT 120
]
```

> **Try this:** change `RT 120` to `LT 120`, so Kame goes round the other way. The bumps point inwards.

## The Sierpinski gasket

An L-system can be about shapes, not only lines. Here's a rule: a triangle becomes three half-size triangles, one in each corner. Level 0 is a triangle, level 1 is three triangles, and level 2 is three level-1s: nine triangles.

```logo gasket
TO GASKET :size :level
  IF :level = 0 [
    REPEAT 3 [FD :size RT 120]
    STOP
  ]
  REPEAT 3 [
    GASKET :size / 2 :level - 1
    FD :size
    RT 120
  ]
END

PU LT 90 FD 240 RT 90 BK 200 PD
RT 30
GASKET 480 4
```

It's like the squares at the corners in the recursion lesson. Each smaller gasket leaves Kame where it started, facing the same way, so `FD :size` and `RT 120` take it to the next corner.

> **Try this:** level 0 is a triangle. What if it drew something else, like a circle, `CIRCLE :size / 3`?

## The same gasket, another way

Here's an L-system with two rules, for two kinds of line, `A` and `B`:

> `A` becomes `B+A+B`, and `B` becomes `A-B-A`

Each rule uses the other, so it's two procedures that call each other: `A` calls `B`, and `B` calls `A`. That's fine: it's still one level at a time.

```logo arrowhead
TO A :size :level
  IF :level = 0 [FD :size STOP]
  B :size / 2 :level - 1
  RT 60
  A :size / 2 :level - 1
  RT 60
  B :size / 2 :level - 1
END

TO B :size :level
  IF :level = 0 [FD :size STOP]
  A :size / 2 :level - 1
  LT 60
  B :size / 2 :level - 1
  LT 60
  A :size / 2 :level - 1
END

SETPC "orange
PU LT 90 FD 240 RT 90 BK 200 PD
RT 90
A 480 4
```

The same gasket appears, but drawn as one wiggly line, without lifting the pen. It's called the **Sierpinski arrowhead**.

> **Try this:** level 5 comes out upside down, and level 6 the right way up again. At level 5, try starting with `B 480 5` instead of `A`.

## The Gosper curve

The Gosper curve has two longer rules, with turns of 60 and 120 degrees. Each level's lines are about 2.65 times shorter than the level above. Kame draws `A`'s lines in gold and `B`'s in blue, so you can see which rule drew each one.

```logo gosper
TO A :size :level
  IF :level = 0 [
    SETPC "gold
    FD :size
    STOP
  ]
  A :size / 2.65 :level - 1
  LT 60
  B :size / 2.65 :level - 1
  LT 120
  B :size / 2.65 :level - 1
  RT 60
  A :size / 2.65 :level - 1
  RT 120
  A :size / 2.65 :level - 1
  A :size / 2.65 :level - 1
  RT 60
  B :size / 2.65 :level - 1
  LT 60
END

TO B :size :level
  IF :level = 0 [
    SETPC "deepskyblue
    FD :size
    STOP
  ]
  RT 60
  A :size / 2.65 :level - 1
  LT 60
  B :size / 2.65 :level - 1
  B :size / 2.65 :level - 1
  LT 120
  B :size / 2.65 :level - 1
  LT 60
  A :size / 2.65 :level - 1
  RT 120
  A :size / 2.65 :level - 1
  RT 60
  B :size / 2.65 :level - 1
END

SETBG "midnightblue
SETPENSIZE 2
PU RT 90 FD 270 LT 90 BK 40 PD
A 460 3
```

Look closely: the shape is made of seven smaller copies of itself, each made of seven smaller ones.

> **Try this:** level 4 draws seven times as many lines as level 3. Turn the speed up to Instant and try it.

## Help Kame fix the bug

Kame meant to draw the level 2 Koch curve from the start of the lesson.

```picture What Kame meant to draw
TO KOCH :size :level
  IF :level = 0 [FD :size STOP]
  KOCH :size / 3 :level - 1
  LT 60
  KOCH :size / 3 :level - 1
  RT 120
  KOCH :size / 3 :level - 1
  LT 60
  KOCH :size / 3 :level - 1
END

PU LT 90 FD 240 RT 180 PD
KOCH 480 2
```

```logo
TO KOCH :size :level
  IF :level = 0 [FD :size STOP]
  KOCH :size / 3 :level - 1
  LT 60
  KOCH :size / 3 :level - 1
  RT 120
  KOCH :size / 2 :level - 1
  LT 60
  KOCH :size / 3 :level - 1
END

PU LT 90 FD 240 RT 180 PD
KOCH 480 2
```

Something's wrong: the right-hand side droops, and the curve ends lower than it started. Change the `2` at the bottom to `1`, and look at the four pieces. Which piece is the wrong length? Which line of `KOCH` draws it?

One way to fix it: every piece is a third of the whole, so the third call should be `KOCH :size / 3 :level - 1`, like the others.

## What next?

That's the tutorial. You know enough to draw almost anything. The Help tab has every command Kame knows, each with examples you can run: press its **Run**, and Kame draws it right here. The Gallery tab has pictures to try, from a snowflake to a jack-o'-lantern: load one into the editor, and see how it's made.

There are many more L-systems to explore. The **dragon curve** has two rules, `X` becomes `X+YF+` and `Y` becomes `-FX-Y`, with turns of 90 degrees, where only `F` draws a line. The **Hilbert curve** fills a square with one line that never crosses itself. Try writing one as procedures, one level at a time.
