# Chapter 4, pp. 109-119: checked against PDF pages 118-128.
FIXES = [
    (r"re:\(Figures 13b and 13c\)\. <!-- p\. 109 --> Figure 13a\n\nFigure 13b\n\nTOPRIGHT: The ball is at the top and is moving to the right\n\n.*?\n\nFigure 13c\n\nTOPLEFT: The ball is at the top and is moving to the left\n",
     "(Figures 13b and 13c).\n\n<!-- p. 109 -->\n\n![Figure 13a: the ball at the top, with an arrow showing it moving](images/p109-figure-13a.png)\n\n**Figure 13a**\n\n"
     "![Figure 13b: the ball at the top, moving to the right](images/p109-figure-13b.png)\n\n**Figure 13b** — *TOPRIGHT: The ball is at the top and is moving to the right*\n\n"
     "![Figure 13c: the ball at the top, moving to the left](images/p109-figure-13c.png)\n\n**Figure 13c** — *TOPLEFT: The ball is at the top and is moving to the left*\n"),
    ("> <!-- p. 110 --> TO KEEP JUGGLING WHEN TOPRIGHT TOSSRIGHT WHEN TOPLEFT TOSSLEFT\n",
     "<!-- p. 110 -->\n\n```logo\nTO KEEP JUGGLING\nWHEN TOPRIGHT TOSSRIGHT\nWHEN TOPLEFT TOSSLEFT\n```\n"),
    ("\n\nTO KEEPJUGGLING WHEN TOPX TOSSX\n", "\n\n```logo\nTO KEEP JUGGLING\nWHEN TOPX TOSSX\n```\n"),
    ("turned juggling into a people procedure be", "turned juggling into a *people procedure* be"),
    (r"re:> 1\.   that the learner can perform.*?\n\n> definitions of the procedure TO KEEP JUGGLING\n",
     "1. that the learner can perform TOSSRIGHT and TOSSLEFT\n2. that she can recognize the trigger states TOPLEFT and TOPRIGHT\n"
     "3. that she can combine these performance abilities according to the definitions of the procedure TO KEEP JUGGLING\n"),
    ("> STEP 1: Verify that the learner can toss.", "> STEP 1: Verify that the learner *can* toss."),
    ("STEP 2: Verify that the learner can combine tosses. Try with two balls\n\nwith instructions\"\n\n> TO CROSS TOPRIGHT WHEN TOPRIGHT TOSSLEFT END\n",
     "> STEP 2: Verify that the learner can combine tosses. Try with two balls with instructions:\n>\n> ```logo\n> TO CROSS\n> TOPRIGHT\n> WHEN TOPRIGHT TOSSLEFT\n> END\n> ```\n"),
    (r"re:\n\n> <!-- p\. 112 --> Ill,,,.*?\n\nFigure 14 Cascade Juggling\n",
     "\n\n<!-- p. 112 -->\n\n![Figure 14: Cascade Juggling, six pictures of the balls passing between the hands](images/p112-figure-14.png)\n\n**Figure 14** — *Cascade Juggling*\n"),
    ("and that everyone learns from", "and that *everyone* learns from"),
    (r"re:We have discussed the program:\n\nTO HOUSE SQUARE TRIANGLE END\n\nTO SQUARE REPEAT 4\n\nFORWARD 100\n\nRIGHT 90 END\n\nTO TRIANGLE REPEAT 3\n\nFORWARD 100\n\nRIGHT 120 END But this program",
     "We have discussed the program:\n\n```logo\nTO HOUSE\nSQUARE\nTRIANGLE\nEND\n\nTO SQUARE\nREPEAT 4\n     FORWARD 100\n     RIGHT 90\nEND\n\nTO TRIANGLE\nREPEAT 3\n     FORWARD 100\n     RIGHT 120\nEND\n```\n\nBut this program"),
    ("The Turtle turned into the square because", "The Turtle turned *into* the square because"),
]
