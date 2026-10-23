# Do it again with REPEAT

The square in the last lesson took eight lines: `FD 100` and `RT 90`, four times over. That's a lot of typing for one square.

[REPEAT] does the typing for you. Give it a number and a list of instructions in square brackets, and Kame follows the list that many times.

```logo
REPEAT 4 [FD 100 RT 90]
```

Run it slowly and look at the **Repeat counters** beside the console. It counts from 1 to 4 as Kame goes round.

> **Try this:** change the `4` to `6`, and the `90` to `60`. What shape does Kame draw now?

## Any shape you like

For a shape with any number of sides, the turns have to add up to a whole turn, 360 degrees. So each turn is 360 divided by the number of sides. Logo can work that out for you: `360 / 8` is 45.

```logo
REPEAT 8 [FD 60 RT 360 / 8]
```

With lots of tiny sides, the shape looks like a circle.

```logo
REPEAT 360 [FD 2 RT 1]
```

## A REPEAT inside a REPEAT

The list can hold more than one instruction, and it can hold another `REPEAT`. Here Kame draws a square, turns a little, and does it again, eight times.

```logo
REPEAT 8 [
  REPEAT 4 [FD 100 RT 90]
  RT 45
]
```

Now there are two counters beside the console: one for each `REPEAT`.

> **Try this:** make it 36 squares, turning 10 degrees after each one.

## Which time is it?

Inside a `REPEAT`, [REPCOUNT] says which time round it is. Using it as a distance makes each line longer than the one before: a spiral.

```logo
REPEAT 50 [FD REPCOUNT * 4 RT 90]
```

> **Try this:** change `RT 90` to `RT 91`, or `RT 121`. Small changes make very different spirals.
