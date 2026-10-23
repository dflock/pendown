# Chapter 3, pp. 62-75: checked against PDF pages 71-84.
SPI_COIL = """```logo
TO SPI              TO COIL
FORWARD 10          FORWARD 5
RIGHT 90            RIGHT 5
FORWARD 15          FORWARD 5
RIGHT 90            RIGHT 5 * .95
FORWARD 20          FORWARD 5
RIGHT 90            RIGHT 5 * .95 * .95
FORWARD 25          FORWARD 5
RIGHT 90            RIGHT 5 * .95 * .95 * .95
FORWARD 30          FORWARD 5
RIGHT 90            RIGHT 5 * .95 * .95 * .95
FORWARD 35          FORWARD 5
RIGHT 90            RIGHT 5 * .95 * .95 * .95 * .95
FORWARD 40          FORWARD 5
RIGHT 90            RIGHT 5 * .95 * .95 * .95 * .95 * .95
FORWARD 45          FORWARD 5
RIGHT 90            RIGHT 5 * .95 * .95 * .95 * .95 * .95 * .95
FORWARD 50          FORWARD 5
RIGHT 90            RIGHT 5 * .95 * .95 * .95 * .95 * .95 * .95 * .95
FORWARD 55          FORWARD 5
RIGHT 90            RIGHT 5 * .95 * .95 * .95 * .95 * .95 * .95 * .95 * .95
FORWARD 60          FORWARD 5
RIGHT 90
FORWARD 65          etc.
RIGHT 90

etc.
```"""

FIXES = [
    # Figure 5 heads p. 62, and the stray mark after the paragraph that follows it.
    ("the following picture (Figure 5). <!-- p. 62 --> Figure 5\n\n",
     "the following picture (Figure 5).\n\n<!-- p. 62 -->\n\n![Figure 5: the house drawn on its side, the triangle to the left of the square](images/p62-figure-5.png)\n\n**Figure 5**\n\n"),
    ("as the first step in the program. ¢", "as the first step in the program."),
    ("\n\nFisure 6a                            Fisure6b\n",
     "\n\n![Figure 6a: three squares, each turned 120 degrees from the last; Figure 6b: squares turned by a smaller angle, making a star](images/p62-figure-6.png)\n\n**Figure 6a**, **Figure 6b**\n"),
    ("One kind is mathematical: The Turtles", "One kind is *mathematical*: The Turtles"),
    ("knowledge is mathetic: knowledge about", "knowledge is *mathetic*: knowledge about"),
    ("she could not identify with this enterprise", "she could not *identify with* this enterprise"),
    ("something children could make sense of", "something children *could* make sense of"),
    ("illustrates syntonic learning.~ This", "illustrates *syntonic learning.*<sup>2</sup> This"),
    ("the Turtle circle is body syntonic in", "the Turtle circle is *body syntonic* in"),
    ("Or it is ego syntonic in", "Or it is *ego syntonic* in"),
    ("Mathematician George Polya ~ has argued", "Mathematician George Polya<sup>3</sup> has argued"),
    ("principle: Play Turtle. Do it yourself In Turtle", "principle: *Play Turtle. Do it yourself.* In Turtle"),
    ("as the core of basic", "as *the* core of basic"),
    ("school math but that ought to be", "school math but that *ought* to be"),
    ("about the formalism of calculus, for example that the derivative of x n is nx n-l, but about its use and its meaning.",
     "about the *formalism* of calculus, for example that the derivative of *x*<sup>*n*</sup> is *nx*<sup>*n*-1</sup>, but about its use and its *meaning.*"),
    ("Turtle; the Turtle program is an intuitive analog of the differential equation, a concept one finds in almost every example of traditional applied mathematics.",
     "Turtle; *the Turtle program is an intuitive analog of the differential equation, a concept one finds in almost every example of traditional applied mathematics.*"),
    ("makes the instructions differential.", "makes the instructions *differential.*"),
    ("described as: (x-a) 2 + (y-b) 2= R 2", "described as:\n\n(*x*−*a*)<sup>2</sup> + (*y*−*b*)<sup>2</sup> = *R*<sup>2</sup>\n\n"),
    ("for a given forward motion. 4", "for a given forward motion.<sup>4</sup>"),
    ("noted that the differential equation is the formalism", "noted that the *differential equation* is the formalism"),
    ("is primarily relational or affective:", "is primarily *relational* or *affective:*"),
    ("the concept of angle plays", "the concept of *angle* plays"),
    ("between these live activities and the dead school", "between these *live* activities and the *dead* school"),
    ("Here we see a cultural syntonicity:", "Here we see a *cultural syntonicity:*"),
    ("is the idea of a variable: the idea", "is the idea of a *variable:* the idea"),
    ("dealing with a variable quantity", "dealing with a *variable* quantity"),
    ("\n\nf\n\nlIt 'l Figure 7\n",
     "\n\n![Figure 7: a coil spiral, and a square spiral](images/p69-figure-7.png)\n\n**Figure 7**\n"),
    (r"re:ideas ever invented\. <!-- p\. 70 --> TO SPI .*?\n\netc\.\n\nFigure 8 How NOT to Draw Spirals\n",
     "ideas ever invented.\n\n<!-- p. 70 -->\n\n" + SPI_COIL + "\n\n**Figure 8** — *How NOT to Draw Spirals*\n"),
    ("such as X, or it could be a whole word, such as ANGLE OR DISTANCE.", "such as *X*, or it could be a whole word, such as *ANGLE OR DISTANCE.*"),
    ("can be done by typing: <!-- p. 71 --> TO STEP DISTANCEFORWARD DISTANCE RIGHT 90 END",
     "can be done by typing:\n\n<!-- p. 71 -->\n\n```logo\nTO STEP DISTANCE\nFORWARD DISTANCE\nRIGHT 90\nEND\n```\n"),
    ("\n\nTO SPI DISTANCE FORWARD DISTANCE RIGHT 90 SPI DISTANCE + 5 END\n",
     "\n\n```logo\nTO SPI DISTANCE\nFORWARD DISTANCE\nRIGHT 90\nSPI DISTANCE + 5\nEND\n```\n"),
    ("\n\n<!-- p. 72 --> Figure 9a <!-- p. 73 --> Flsure 9b <!-- p. 74 --> think this",
     "\n\n<!-- p. 72 -->\n\n![Figure 9a: six frames of the Turtle drawing a square spiral, one move at a time](images/p72-figure-9a.png)\n\n**Figure 9a**\n\n"
     "<!-- p. 73 -->\n\n![Figure 9b: SPI drawn with angles close to 90 degrees, the spirals' arms twisting into galaxies](images/p73-figure-9b.png)\n\n**Figure 9b**\n\n"
     "<!-- p. 74 --> think this"),
    ("such as: 5 + X = 8. What is X?", "such as:\n\n5 + X = 8. What is X?\n\n"),
    ("Mathpower, one might say, becomes a way of life.", "*Mathpower,* one might say, *becomes a way of life.*"),
    ("developing ways to talk about mathematics", "developing ways to talk *about* mathematics"),
    (r"re:The form of the program might be:\n\nTO TRIANGLE REPEAT 3\n.*?RIGHT SOMETHING END\n",
     "The form of the program might be:\n\n```logo\nTO TRIANGLE\nREPEAT 3\n  FORWARD 100\n  RIGHT SOMETHING\nEND\n```\n"),
    ("What is the quantity we called SOMETHING ?", "What is the quantity we called SOMETHING?"),
    (r"re:so that the square program was:\n\nTO SQUARE REPEAT 4\n.*?RIGHT 90 END\n",
     "so that the square program was:\n\n```logo\nTO SQUARE\nREPEAT 4\n  FORWARD 100\n  RIGHT 90\nEND\n```\n"),
    ("work together. What is the same in the square", "work together. *What is the same* in the square"),
    ("one complete turn. What is different <!-- p. 76 -->", "one complete turn. *What is different* <!-- p. 76 -->"),
]
