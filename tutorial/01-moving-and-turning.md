# Moving and turning

This is Kame, the turtle in the middle of the drawing. Kame has a pen, and when Kame moves, the pen draws a line. You tell Kame where to go with commands.

`FD` is short for [FORWARD]. Give it a number, and Kame walks that many steps the way it's facing. Kame starts in the middle, facing up.

```logo
FD 100
```

Press **Load into editor**, then **Run** at the top. Kame walks up and draws a line.

`RT` is short for [RIGHT]. It turns Kame to its right, by a number of degrees. A whole turn is 360 degrees, so a quarter turn is 90.

```logo
FD 100
RT 90
FD 100
```

> **Try this:** add more instructions to finish the square. How many times do you need `FD 100` and `RT 90`?

## Four sides make a square

Here's the whole square. Kame ends where it started, facing up again, because it turned 4 times 90 degrees: a whole turn.

```logo
FD 100
RT 90
FD 100
RT 90
FD 100
RT 90
FD 100
RT 90
```

Slow Kame down with the speed slider, and watch the highlight in the editor move from line to line as Kame goes. **Pause** stops Kame part-way, and **Step** goes one line at a time.

## Backwards and left

[BACK], or `BK`, walks Kame backwards, and [LEFT], or `LT`, turns it left. [PENUP], or `PU`, lifts the pen so Kame moves without drawing, and [PENDOWN], or `PD`, puts it down again.

```logo
FD 100
LT 90
PU
FD 50
PD
FD 50
BK 150
```

> **Try this:** draw a triangle. Kame has to turn 120 degrees at each corner, because 3 times 120 is 360.

If you want to know more about any command, put the editor's cursor on it and press **F1**.
