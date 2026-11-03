# Inputs and variables

`BOX` from the last lesson always draws the same size of square. [FORWARD] is more useful: you tell it how far. Your words can take inputs too.

Put a name with a colon after the word's name, like `:size`. Inside the procedure, `:size` means whatever number was given.

```logo
TO BOX :size
  REPEAT 4 [FD :size RT 90]
END

BOX 50
BOX 100
BOX 150
```

`:size` is a variable: a name for a value. While Kame draws, the variables panel shows `size` and its value, and you can watch it change from one box to the next.

> **Try this:** give `BOX` a second input, `:turn`, and use it in place of `RT 90`. What does `BOX 100 60` draw?

## Making variables

[MAKE] gives a variable a value. The name goes after a quote, `"length`, because it's the name you mean, not its value. Then `:length` gets the value back.

```logo
MAKE "length 10
REPEAT 40 [
  FD :length
  RT 90
  MAKE "length :length + 5
]
```

Each time round, `MAKE "length :length + 5` makes the length 5 steps longer, so the spiral grows. Watch `length` in the variables panel flash each time it changes.

## Inputs and procedures together

Here `POLYGON` draws a shape with any number of sides, any size. Its two inputs are worked into the turn and the length.

```logo
TO POLYGON :sides :length
  REPEAT :sides [FD :length RT 360 / :sides]
END

POLYGON 3 150
POLYGON 5 100
POLYGON 8 60
```

> **Try this:** use `REPEAT` and [REPCOUNT] to draw `POLYGON REPCOUNT + 2 60` for each of 8 times. Every shape has one more side than the last.

## Help Kame fix the bug

Kame wanted three boxes, each bigger than the last.

```picture What Kame meant to draw
PU LT 90 FD 75 RT 90 BK 75 PD
REPEAT 4 [FD 50 RT 90]
REPEAT 4 [FD 100 RT 90]
REPEAT 4 [FD 150 RT 90]
```

```logo
TO BOX :size
  REPEAT 4 [FD 100 RT 90]
END

BOX 50
BOX 100
BOX 150
```

Kame was given three sizes, but all three boxes come out the same. Watch the variables panel as it runs: `size` changes each time. Where does `:size` go?

One way to fix it: `BOX` has to use its input, so `FD 100` becomes `FD :size`.
