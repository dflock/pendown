# The `try` field, with first examples

Status: resolved
Type: task

## What

A new optional field in `content/reference.kdl`, the card's example:

```kdl
forward {
  eg """
  FORWARD dist
  FD dist
  """
  try "FD 100"
}
```

- `Command` in `reference.rs` gets it, under a name that isn't a Rust keyword, such as `short_example`. The header comment in the kdl says what it's for.
- Rules, checked by a test: one line, 40 characters at most, and it runs without an error on an empty program.
- It needs no saved picture, as `tests/examples.rs` requires of the help's examples. A separate test runs them.

**First examples:** draft `try` for each command the tutorial teaches, found by going through `content/tutorial`, plus any command a help page's first examples lean on. Use the shortest form a learner would write (`FD`, not `FORWARD`). Where a command only makes sense with others, such as `REPCOUNT`, the example can be a small whole instruction.

## Done when

- The field parses, and the test above passes for every `try`.
- The list of drafted examples is in a comment below, for the maintainer to skim. Then set the status to `resolved` once they have.

## Comments

2026-11-04: Field and drafts done, waiting for the maintainer's skim. `try` becomes `Command::short_example`. `tests/reference.rs` checks that each is one line, at most 40 characters, runs without an error, and uses its own command. Breaking `forward`'s example four ways made the right check fail each time. All 48 ran; I also read what each printed or how many moves it made, and called `SQ` and `TWICE` to confirm the one-line `TO` forms define them.

**The tutorial teaches (28),** from its code, inline code and `[NAME]` links:

| Command | `try` |
|---|---|
| FORWARD | `FD 100` |
| BACK | `BK 50` |
| RIGHT | `RT 90` |
| LEFT | `LT 90` |
| PENUP, PENDOWN | `PU FD 50 PD` |
| REPEAT | `REPEAT 4 [FD 100 RT 90]` |
| REPCOUNT | `REPEAT 4 [FD REPCOUNT * 20 RT 90]` |
| TO | `TO SQ FD 50 RT 90 END` |
| MAKE | `MAKE "size 100` |
| IF | `IF HEADING = 0 [FD 100]` |
| STOP | `IF XCOR > 100 [STOP]` |
| SETPENCOLOR | `SETPC "red` |
| SETBACKGROUND | `SETBG "navy` |
| SETPENSIZE | `SETPENSIZE 5` |
| SETFILLCOLOR | `SETFC "orange` |
| ITEM | `PRINT ITEM 2 [a b c]` |
| CIRCLE | `CIRCLE 100` |
| ELLIPSE | `ELLIPSE 100 50` |
| RECTANGLE | `RECTANGLE 100 50` |
| SQUARE | `SQUARE 100` |
| SUM | `PRINT 3 + 4` |
| DIFFERENCE | `PRINT 10 - 4` |
| PRODUCT | `PRINT 6 * 7` |
| QUOTIENT | `PRINT 360 / 5` |
| EQUALP | `PRINT 2 = 2` |
| LESSP | `PRINT 3 < 4` |
| GREATERP | `PRINT 4 > 3` |

**Help pages' first examples lean on (20).** "Lean on" is taken as appearing in the first example of three or more pages, counted with a throwaway script:

| Command | `try` |
|---|---|
| PRINT | `PRINT "hello` |
| SHOW | `SHOW [1 2 3]` |
| TYPE | `TYPE "hello` |
| SETXY | `SETXY 50 100` |
| YCOR | `PRINT YCOR` |
| HIDETURTLE | `HT` |
| OUTPUT | `TO TWICE :n OP :n * 2 END` |
| CATCH | `CATCH "oops [THROW "oops]` |
| MAP | `SHOW MAP [? * 2] [1 2 3]` |
| FOREACH | `FOREACH [50 80] [FD ? RT 90]` |
| WORD | `PRINT WORD "sun "flower` |
| LIST | `SHOW LIST 1 2` |
| SENTENCE | `SHOW SENTENCE [a b] [c]` |
| FIRST | `PRINT FIRST [a b c]` |
| BUTFIRST | `SHOW BF [a b c]` |
| ISEQ | `SHOW ISEQ 1 5` |
| POP | `MAKE "s [1 2] PRINT POP "s` |
| SETITEM | `SETITEM 1 ARRAY 3 "x` |
| MDARRAY | `SHOW MDARRAY [2 3]` |
| MDSETITEM | `MDSETITEM [1 1] MDARRAY [2 2] "x` |

**Worth a look:**

- **`TO` and `OUTPUT`** are on one line, because `try` must be, though the tutorial puts `END` on its own line. It's valid, but it's the one place the card shows a shape the lessons don't.
- **The infix forms** show `+` for SUM, as the tutorial teaches, so `SUM 1` gets the syntax line with SUM and an example with `+`. MINUS has no `try`, as `-` finds DIFFERENCE.
- **`SETITEM` and `MDSETITEM`** run correctly but change an array nobody can see. A clearer version doesn't fit one short line.

2026-11-04: The maintainer skimmed the list and is happy with it, including the points above.
