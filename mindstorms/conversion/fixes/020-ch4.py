# Chapter 4, pp. 95-108: checked against PDF pages 104-117.
MAN = """TO MAN
FORWARD 300
RIGHT 120
FORWARD 300
RIGHT 180
FORWARD 300
LEFT 120
FORWARD 300
LEFT 120
FORWARD 300
RIGHT 180
FORWARD 300
RIGHT 120
FORWARD 300
RIGHT 180
FORWARD 300
LEFT 120
FORWARD 150
LEFT 45
FORWARD 50
RIGHT 90
FORWARD 50
RIGHT 90
FORWARD 50
RIGHT 90
FORWARD 50
END"""

FIXES = [
    # The epigraph, dropped with the chapter's printed title.
    ("<!-- p. 95 -->\n\nTHE CENTIPEDE STORY",
     "<!-- p. 95 -->\n\n> The Centipede was happy quite  \n> Until the toad in fun  \n> Said, Pray which leg comes after which?  \n"
     "> This wrought her mind to such a pitch  \n> She lay distracted in a ditch  \n> Considering how to run  \n>\n> —Anonymous\n\nTHE CENTIPEDE STORY"),
    ("verbalizable versus nonverbalizabl¢.", "verbalizable versus nonverbalizable."),
    ("learning of physical skills. 2 Without", "learning of physical skills.<sup>2</sup> Without"),
    ("In his book Inner Tennis he", "In his book *Inner Tennis* he"),
    ("skills very likely would leave", "skills very likely *would* leave"),
    ("\n\nFigure tOa Goal\n",
     "\n\n![Figure 10a: Goal, a stick figure: two V shapes for legs and arms, and a diamond for a head](images/p100-figure-10a.png)\n\n**Figure 10a** — *Goal*\n"),
    (r"re:He wrote \(Figure 10b\): <!-- p\. 101 --> TO MflN .*?\n\nEND\n\nFIEure rob +Fisure 10c BuEEed Man\n",
     "He wrote (Figure 10b):\n\n<!-- p. 101 -->\n\n```logo\n" + MAN + "\n```\n\n**Figure 10b**\n\n"
     "![Figure 10c: Bugged Man, a triangle with a small square at its point](images/p101-figure-10c.png)\n\n**Figure 10c** — *Bugged Man*\n"),
    ("\n\nTO MAN VEE FORWARD 50 VEE FORWARD 25 HEAD END\n", "\n\n```logo\nTO MAN\nVEE\nFORWARD 50\nVEE\nFORWARD 25\nHEAD\nEND\n```\n"),
    ("understand as a whole. For example: <!-- p. 103 --> TO VEE RIGHT 120 LINE 50 RIGHT 120 LINE 50 RIGHT 120 END",
     "understand as a whole. For example:\n\n<!-- p. 103 -->\n\n```logo\nTO VEE\nRIGHT 120\nLINE 50\nRIGHT 120\nLINE 50\nRIGHT 120\nEND\n```"),
    ("\n\nTO LINE :DISTANCE FORWARD :DISTANCE BACK :DISTANCE END\n", "\n\n```logo\nTO LINE :DISTANCE\nFORWARD :DISTANCE\nBACK :DISTANCE\nEND\n```\n"),
    ("\n\nTO HEAD RIGHT 45 SQUARE 20 END\n", "\n\n```logo\nTO HEAD\nRIGHT 45\nSQUARE 20\nEND\n```\n"),
    ('to say: "You know what to do!"', 'to say: "You *know* what to do!"'),
    # Figure 11 sits in the middle of a sentence on p. 106; it goes before the paragraph.
    ("\n\nCascade juggling has a simpler structure.",
     "\n\n![Figure 11: Two Forms of Juggling, Showers and Cascade](images/p106-figure-11.png)\n\n**Figure 11** — *Two Forms of Juggling*\n\nCascade juggling has a simpler structure."),
    (r"re:over the upper arc\. There\n\n> <!-- p\. 106 --> /.*?> Two Forms of Juggling\n\nis only one kind of toss",
     "over the upper arc. There <!-- p. 106 --> is only one kind of toss"),
    ("But, of course, s o m e verbal", "But, of course, *some* verbal"),
    (r"re:\n\n> 1\.   Start with balls.*?\n\n> and so on\.\n",
     "\n\n1. Start with balls 1 and 2 in the left hand and ball 3 in the right.\n2. Throw ball 1 in a high parabola to the right hand.\n"
     "3. When ball 1 is at the vertex throw ball 3 over to the left hand in a similar high parabola, but take care to toss ball 3 under the trajectory of ball 1.\n"
     "4. When ball 1 arrives at the right hand and ball 3 is at the vertex, catch ball 1 and throw ball 2 in a trajectory under that of ball 3, and so on.\n"),
    ("structured programming that are good for", "structured programming that *are* good for"),
    ("juggling we must worry about it", "juggling we *must* worry about it"),
    ("and catch it when it arrives. 3", "and catch it when it arrives.<sup>3</sup>"),
    ("dealing with parallel processes as opposed to the strictly serial processes", "dealing with *parallel* processes as opposed to the strictly *serial* processes"),
    ('The concept of a " WHEN DEMON."', 'The concept of a "WHEN DEMON."'),
    ("by the instruction: WHENHUNGRY EAT.", "by the instruction: WHEN HUNGRY EAT."),
    ("\n\nWHEN something TOSSLEFT WHEN something TOSSRIGHT\n", "\n\n```logo\nWHEN something TOSSLEFT\nWHEN something TOSSRIGHT\n```\n"),
    ("this (Figure 12):\n\ne\n\n~~L .... Left Hand     Right Hand\n\nFigure 12\n",
     "this (Figure 12):\n\n![Figure 12: one ball in the air above the hands, and one ball in each hand, Left Hand and Right Hand](images/p108-figure-12.png)\n\n**Figure 12**\n"),
]
