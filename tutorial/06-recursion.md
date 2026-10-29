# Shapes inside shapes: recursion

A procedure can use any word Kame knows, and that includes itself. A procedure that calls itself is recursive.

This one draws a line, turns, and then calls itself with a longer line. Without anything to stop it, it would go on for ever, so [IF] stops it when the line gets long enough. [STOP] ends the procedure.

```logo
TO SPIRAL :length
  IF :length > 200 [STOP]
  FD :length
  RT 90
  SPIRAL :length + 10
END

SPIRAL 10
```

Step through it and look at the variables panel. Each call of `SPIRAL` has its own `:length`.

> **Try this:** turn `RT 91` instead of `RT 90`, and start at `SPIRAL 1` going up by 2 each time.

## Shapes inside shapes

Recursion can draw something that contains smaller copies of itself. This square draws a smaller square at each of its corners, which draws smaller ones at its corners, until they're too small to bother with.

```logo
TO CORNERS :size
  IF :size < 10 [STOP]
  REPEAT 4 [
    FD :size
    CORNERS :size / 2.5
    RT 90
  ]
END

CORNERS 200
```

## A tree

A tree is a trunk with two smaller trees on top. `TREE` draws its trunk, turns left and draws a smaller tree, turns right and draws another, then walks back down its trunk to where it started.

```logo
TO TREE :size
  IF :size < 5 [STOP]
  FD :size
  LT 30
  TREE :size * 0.7
  RT 60
  TREE :size * 0.7
  LT 30
  BK :size
END

PU BK 200 PD
TREE 120
```

Walking back to where it started matters: each smaller tree leaves Kame just where it found it, so the next one can start in the right place.

> **Try this:** change the angles, `30` and `60`, or `0.7`. A tree that turns 20 degrees and shrinks by 0.8 looks quite different.

## Help Kame fix the bug

Kame meant to draw the tree above.

```picture
PU BK 200 PD
TO TREE :size
  IF :size < 5 [STOP]
  FD :size
  LT 30
  TREE :size * 0.7
  RT 60
  TREE :size * 0.7
  LT 30
  BK :size
END

TREE 120
```

```logo
TO TREE :size
  IF :size < 5 [STOP]
  FD :size
  LT 30
  TREE :size * 0.7
  RT 60
  TREE :size * 0.7
  BK :size
END

PU BK 200 PD
TREE 120
```

This tree comes out in a tangle: its branches wander off, and the trunk ends up pointing the wrong way. Step through it with **Thinking** on, and watch which way Kame faces each time a smaller tree finishes. Does each one leave Kame just where it found it?

One way to fix it: after the second smaller tree, Kame is still turned 30 degrees to the right. `LT 30` before `BK :size` points it back up its trunk.
