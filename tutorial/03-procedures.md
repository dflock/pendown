# Teaching Kame new words

Kame knows about 160 words, like `FD`, `RT` and `REPEAT`. You can teach it more. A new word is called a procedure.

[TO] starts teaching a word, and `END` finishes it. Everything in between is what the word means.

```logo
TO BOX
  REPEAT 4 [FD 100 RT 90]
END

BOX
```

The first four lines teach Kame the word `BOX`, and don't draw anything. The last line uses the new word, and Kame draws a square. Now you can use `BOX` as often as you like.

```logo
TO BOX
  REPEAT 4 [FD 100 RT 90]
END

BOX
RT 45
BOX
```

Kame already knows a word called `SQUARE`, so give your words names of their own, like `BOX`.

## Words that use other words

A procedure can use the words you taught Kame, as well as the ones it knew. Here `FLOWER` draws `BOX` twelve times, turning in between.

```logo
TO BOX
  REPEAT 4 [FD 100 RT 90]
END

TO FLOWER
  REPEAT 12 [BOX RT 30]
END

FLOWER
```

Step through it, slowly. When Kame is drawing a box, the variables panel shows `FLOWER` and then `BOX` inside it: the words Kame is in the middle of.

> **Try this:** teach Kame `TRIANGLE`, which draws a triangle, and a `WINDMILL` that draws six triangles, turning 60 degrees after each one.

## Make it tidy

Procedures make a program easier to read. A good name says what it draws, and the steps inside can be short. `HOUSE` reads almost like English:

```logo
TO BOX
  REPEAT 4 [FD 100 RT 90]
END

TO ROOF
  FD 100
  RT 30
  REPEAT 3 [FD 100 RT 120]
  LT 30
  BK 100
END

TO HOUSE
  BOX
  ROOF
END

HOUSE
```
