# Help Kame fix the bug

Kame wants to draw a house. Its plan: a house is a box with a triangle on top. Kame already knows `BOX`, so it teaches itself `TRIANGLE`, and then `HOUSE`, which draws one and then the other.

```picture
PU LT 90 FD 50 RT 90 BK 93 PD
REPEAT 4 [FD 100 RT 90]
FD 100 RT 30
REPEAT 3 [FD 100 RT 120]
```

```logo
TO BOX
  REPEAT 4 [FD 100 RT 90]
END

TO TRIANGLE
  REPEAT 3 [FD 100 RT 120]
END

TO HOUSE
  BOX
  TRIANGLE
END

HOUSE
```

Run it. That isn't a house: the roof is inside the box. Something in the program isn't doing what Kame meant. Programmers call that a **bug**, and finding it and fixing it is called **debugging**. Everyone's programs have bugs, every day: it's how programs get made.

## What went wrong?

Can you help Kame fix it? Some ways to look:

- Step through it, and watch where Kame is, and which way it faces, when `TRIANGLE` starts.
- Turn on **Thinking**, and watch which way each turn goes.
- Be Kame: stand up, and walk the program on the floor. `FD 100` is a big step, and `RT 90` is a quarter turn.

## A tent, too

Kame's bug drew something else instead: a box with a tent inside. That's a picture too, so don't lose it.

> **Try this:** teach Kame `TENT`, which draws the picture with the bug in, then fix `HOUSE`.

## One way to fix it

Here's one way. There are others, and yours might be different.

```logo
TO BOX
  REPEAT 4 [FD 100 RT 90]
END

TO TRIANGLE
  REPEAT 3 [FD 100 RT 120]
END

TO HOUSE
  BOX
  FD 100
  RT 30
  TRIANGLE
END

HOUSE
```

After `BOX`, Kame is back where it started, facing up. `FD 100` takes it to the top of the box, and `RT 30` points it along the roof, so the triangle goes up and over instead of into the box. The last lesson's `ROOF` did the same.

## More bugs to fix

### A square that won't close

```picture
PU BK 50 LT 90 FD 50 RT 90 PD
REPEAT 4 [FD 100 RT 90]
```

```logo
REPEAT 4 [FD 100 RT 80]
```

How far does Kame turn altogether, going round? A shape that closes up turns Kame all the way round once: 360 degrees. Four turns of 80 make 320.

### A star that won't close

```picture
PU LT 90 FD 22 RT 90 BK 100 PD
REPEAT 5 [FD 200 RT 144]
```

```logo
REPEAT 5 [FD 200 RT 140]
```

This star turns Kame round twice as it draws, not once. How far does each turn have to be, for five of them to make two whole turns?

### A flower with a gap

```picture
TO BOX
  REPEAT 4 [FD 100 RT 90]
END

REPEAT 12 [BOX RT 30]
```

```logo
TO BOX
  REPEAT 4 [FD 100 RT 90]
END

REPEAT 12 [BOX RT 20]
```

Kame's flower stops before it gets back round. Twelve turns of 20 make 240 degrees. There's more than one way to fix it: change the turn, or change how many boxes.
