# Chapter 2, pp. 48-54, and Chapter 3, pp. 55-61: checked against PDF pages 57-70.
FIXES = [
    ("what it might be for. And when", "what it might be *for*. And when"),
    (r"re:> INSANE RETARD MAKES.*?SWEET FOGINY SKIPS A FAT LADY RUNS\n\nJenny's Concrete Poetry\n",
     "```\nINSANE RETARD MAKES BECAUSE SWEET SNOOPY SCREAMS\nSEXY WOLF LOVES THATS WHY THE SEXY LADY HATES\n"
     "UGLY MAN LOVES BECAUSE UGLY DOG HATES\nMAD WOLF HATES BECAUSE INSANE WOLF SKIPS\n"
     "SEXY RETARD SCREAMS THATS WHY THE SEXY RETARD HATES\nTHIN SNOOPY RUNS BECAUSE FAT WOLF HOPS\n"
     "SWEET FOGINY SKIPS A FAT LADY RUNS\n```\n\n*Jenny's Concrete Poetry*\n"),
    ("And I think it introduces a deep element of dishonesty into the educational relationship.",
     "*And I think it introduces a deep element of dishonesty into the educational relationship.*"),
    ("The kind of mathematics foisted", "The *kind of mathematics* foisted"),
    ("to say that despite its intrinsic", "to say that *despite* its intrinsic"),
    ("between mathematics—a vast domain", "between *mathematics*—a vast domain"),
    ("which I shall call math or school math.", "which I shall call *math* or *school math.*"),
    ("topics as the mathematical baggage", "topics as *the* mathematical baggage"),
    ("were of a mathetic nature", "were of a *mathetic* nature"),
    ("that y = x z is the equation", "that *y* = *x*<sup>2</sup> is the equation"),
    ("criterion was to be appropriable.", "criterion was to be *appropriable.*"),
    ("there was the continuity principle:", "there was the *continuity principle*:"),
    ("there was the power principle\" It must", "there was the *power principle*: It must"),
    ("there was a principle of cultural resonance:", "there was a *principle of cultural resonance*:"),
    ("Euclid's is a logical style. Descartes's is an algebraic style. Turtle geometry is a computational style",
     "Euclid's is a *logical* style. Descartes's is an *algebraic* style. Turtle geometry is a *computational* style"),
    ("I am here and I am facing", "I am *here* and I am facing"),
    ("Children can identify with the Turtle", "Children can *identify* with the Turtle"),
    ("Two physically different entities can be mathematically the same", "Two *physically* different entities can be *mathematically* the same"),
    ("without affecting the position\" RIGHT and LEFT", "without affecting the position: RIGHT and LEFT"),
    ("Like FORWARD , a turning", "Like FORWARD, a turning"),
    (r"re:\n\nA square can be produced by the commands\n\n.*?An Actual Transcript of a Child's Early Attempt at a Square\n",
     "\n\n```\nA square can be produced by the commands\n\nFORWARD 100\nRIGHT 90\nFORWARD 100\nRIGHT 90\nFORWARD 100\nRIGHT 90\nFORWARD 100\nRIGHT 90\n\n"
     "FD 100      (note the abbreviations to reduce typing)\nRT 100\nFD 100\nERASE 1     (this undoes the effect of the previous command)\n\n"
     "RT10        (fiddling the turtle in search of the right angle)\nLT 10\nLT 10\nFD 100\nRT 100      (gets there faster this time)\nLT 10\n\n"
     "RT 100\nLT 10\nFD 100\nRT 40\nFD 100\nRT 90\nFD 100\n```\n\n**Figure 3** — *An Actual Transcript of a Child's Early Attempt at a Square*\n"),
    ("\n\nTO CIRCLE REPEAT [FORWARD 1 RIGHT 1]\n", "\n\n```logo\nTO CIRCLE REPEAT [FORWARD 1 RIGHT 1]\n```\n"),
    (r"re:\n\nTO SQUARE FORWARD 100 RIGHT 90.*?RIGHT 90 END\n\nSimilarly we can program an equilateral triangle by: <!-- p\. 60 --> TO TRIANGLE.*?RIGHT 1200 END\n",
     "\n\n```logo\nTO SQUARE\nFORWARD 100\nRIGHT 90\nFORWARD 100\nRIGHT 90\nFORWARD 100\nRIGHT 90\nFORWARD 100\nEND\n```\n\n"
     "```logo\nTO SQUARE\nREPEAT 4\n  FORWARD 100\n  RIGHT 90\nEND\n```\n\n"
     "```logo\nTO SQUARE :SIZE\nREPEAT 4\n  FORWARD :SIZE\n  RIGHT 90\nEND\n```\n\n"
     "Similarly we can program an equilateral triangle by:\n\n<!-- p. 60 -->\n\n"
     "```logo\nTO TRIANGLE\nFORWARD 100\nRIGHT 120\nFORWARD 100\nRIGHT 120\nFORWARD 100\nEND\n```\n\n"
     "```logo\nTO TRIANGLE :SIDE\nREPEAT 3\n  FORWARD :SIDE\n  RIGHT 1200\nEND\n```\n"),
    ("general ideas\" for example, the idea", "general ideas: for example, the idea"),
    ("try to forget it as fast", "try to *forget* it as fast"),
    ("So she tries: TO HOUSE SQUARE TRIANGLE END But when she gives the command HOUSE, the Turtle draws (Figure 4.)\"",
     "So she tries:\n\n```logo\nTO HOUSE\nSQUARE\nTRIANGLE\nEND\n```\n\nBut when she gives the command HOUSE, the Turtle draws (Figure 4.):"),
    ("\n\nFlsure 4\n\n", "\n\n![Figure 4: the triangle drawn inside the square](images/p61-figure-4.png)\n\n**Figure 4**\n\n"),
]
