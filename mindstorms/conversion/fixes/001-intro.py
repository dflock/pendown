# Introduction, pp. 6-18: checked against PDF pages 15-27.
FIXES = [
    ("from Jean Piaget ~ a model", "from Jean Piaget<sup>1</sup> a model"),
    ('"not mathematical,\' \' " artistic"', '"not mathematical," "artistic"'),
    ("(the LOGO 2 group", "(the LOGO<sup>2</sup> group"),
    ("that of a carrier of cultural", "that of a *carrier* of cultural"),
    ("a little b e t t e r ~ o r at least", "a little better—or at least"),
    ('computational "object-to-think-with." This is the "Turtle. \'\'3', 'computational "object-to-think-with." This is the "Turtle."<sup>3</sup>'),
    ("seek out or avoid objects. 4 Later", "seek out or avoid objects.<sup>4</sup> Later"),
    ("question of the economic feasibility", "question of the *economic* feasibility"),
    # Figure 1, pp. 14-15: four steps in the evolution of the house program.
    (r"re:<!-- p\. 14 --> Figure I\n\nA Plan\n\n.*?<!-- p\. 15 --> And a Compromise\n",
     "\n\n<!-- p. 14 -->\n\n**Figure 1** — *A Plan*, *A Bug*\n\n![Figure 1: A Plan, a drawing of a house, a square with a triangle on top; and A Bug, the triangle drawn inside the square](images/p14-figure-1.png)\n\n"
     "```logo\nTO HOUSE\nSQ\nTRI\n```\n\n"
     "<!-- p. 15 -->\n\n*And a Compromise*\n\n![Figure 1, continued: And a Compromise, two houses, the first lying on its side, the second upright](images/p15-figure-1.png)\n\n"
     "```logo\nTO HOUSE\nSQ\nRIGHT 30\nTRI\nEND\n```\n\n```logo\nTO HOUSE\nRIGHT 90\nSQ\nRIGHT 30\nTRI\nEND\n```\n\n"),
]
