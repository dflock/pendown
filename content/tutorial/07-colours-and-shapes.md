# Colours and shapes

Kame's pen starts out white. [SETPENCOLOR], or `SETPC`, changes it. You can give a colour's name, like `"orange` or `"skyblue`, a palette number from 0 to 15, or a list of how much red, green and blue to mix, each from 0 to 255.

```logo
SETPC "orange
REPEAT 4 [FD 100 RT 90]
RT 45
SETPC [80 160 255]
REPEAT 4 [FD 100 RT 90]
```

[SETPENSIZE] makes the lines thicker, and [SETBACKGROUND], or `SETBG`, changes the colour behind the drawing.

```logo star
SETBG "midnightblue
SETPC "gold
SETPENSIZE 4
REPEAT 5 [FD 200 RT 144]
```

The variables panel shows the colours Kame is using, with a square of each.

## A rainbow

A list can hold the colours, and [ITEM] picks one by its position. With [REPCOUNT] as the position, each time round has the next colour.

```logo hexagons
MAKE "colours [red orange yellow green blue purple]
SETPENSIZE 6
REPEAT 6 [
  SETPC ITEM REPCOUNT :colours
  REPEAT 6 [FD 80 RT 60]
  RT 60
]
```

## Shapes

Kame can draw whole shapes at once, centred on where it stands: [CIRCLE], [ELLIPSE], [SQUARE] and [RECTANGLE]. Kame doesn't move while it draws them.

```logo
CIRCLE 150
SQUARE 200
ELLIPSE 60 120
```

[SETFILLCOLOR], or `SETFC`, fills the shapes with a colour. With the pen up, a shape has no outline, only its fill.

```logo target
SETFC "gold
CIRCLE 120
SETFC "orangered
PU
CIRCLE 70
SETFC "white
CIRCLE 20
```

> **Try this:** draw a snowman: three white circles, one above the other, getting smaller. Use `PU`, `FD` and `PD` to move between them.

## Help Kame fix the bug

Kame wanted an orange square, and a sky blue one turned across it.

```picture
PU LT 90 FD 71 RT 90 BK 15 PD
SETPC "orange
REPEAT 4 [FD 100 RT 90]
RT 45
SETPC "skyblue
REPEAT 4 [FD 100 RT 90]
```

```logo
REPEAT 4 [FD 100 RT 90]
SETPC "orange
RT 45
REPEAT 4 [FD 100 RT 90]
SETPC "skyblue
```

The first square is white, the second is orange, and there's no sky blue at all. [SETPENCOLOR] changes the colour of what Kame draws next. When does each `SETPC` happen?

One way to fix it: put each `SETPC` before the square it's for.

## What next?

Recursion again, with snowflakes, triangles made of triangles, and curves that fill a shape: the next lesson is about L-systems.
