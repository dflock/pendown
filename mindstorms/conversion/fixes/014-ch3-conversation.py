# Chapter 3, pp. 76-93: checked against PDF pages 85-102. Pages 78-92 are a
# picture story: each page is kept as an image, with its words written out
# from the image, as the OCR of them is unusable.
def page(n, alt, body):
    return f"<!-- p. {n} -->\n\n![{alt}](images/p{n}-conversation.png)\n\n{body.strip()}\n\n"

def said(*lines):
    return "\n\n".join("— " + l for l in lines)

def code(text):
    return "```logo\n" + text.strip() + "\n```"

STORY = (
    "<!-- p. 77 -->\n\n*What follows is a hypothetical conversation between two children who are working and playing with the computer. "
    "These and other experiments can happen every day—and they do.*\n\n"
    + page(78, "A PLAN: a child's drawing of a flower with six petals, a stem and a leaf", "### A Plan\n\n" + said("Let's make the computer draw a flower like this."))
    + page(79, "FIND RESOURCES: a screen of quarter circles of three sizes, labelled QCIRCLE 25, QCIRCLE 50 and QCIRCLE 100", "### Find Resources\n\n" + said(
        "Do you have any programs we could use?", "Yes, there's that quarter circle thing I made last week.", "Show me.")
        + "\n\n" + said("It draws quarter circles starting wherever the turtle is.", "It needs an input to tell it how big."))
    + page(80, "TRY SOMETHING: two quarter circles drawn one after the other make a semicircle", "### Try Something\n\n" + said(
        "Let's make a petal by putting two QCIRCLES together.", "OK. What size?", "How about 50?")
        + "\n\n" + code("QCIRCLE 50\nQCIRCLE 50") + "\n\n### A First Bug\n\n" + said("It didn't work.", "Of course! Two QCIRCLES make a semicircle."))
    + page(81, "FIX THE BUG: with a left turn of 120 between them, the two quarter circles make the shape of a bird", "### Fix the Bug\n\n" + said(
        "We have to turn the Turtle between QCIRCLES.", "Try 120°.", "OK, that worked for triangles.", "And let's hide the Turtle by typing HIDETURTLE.")
        + "\n\n" + code("QCIRCLE 50\nLEFT 120\nQCIRCLE 50") + "\n\n### It's a Bird!\n\n" + said("What's going on?", "Try a right turn."))
    + page(82, "IT'S A FISH!: with a right turn of 120, the quarter circles cross, like a fish", said(
        "Why don't we just stick with the bird? We could make a flock.", "You do that. I want my flower.", "We could do the flower, then the flock.")
        + "\n\n" + code("QCIRCLE 50\nRIGHT 120\nQCIRCLE 50") + "\n\n### It's a Fish!\n\n" + said(
        "The right turn is better.", "But we don't know how much to turn.", "We could try some more numbers.", "Or we could try some mathematics."))
    + page(83, "MATH TO THE RESCUE: a circle with an arrow going all the way round it", "### Math to the Rescue\n\n" + said(
        "Do you know about the Total Turtle Trip Theorem? You think about the Turtle going all around the petal and add up all the turns. 360°.")
        + "\n\n" + said("All around is 360.", "Each QCIRCLE turns it 90. That makes 180 for two QCIRCLES.",
        "360 altogether. Take away 180 for the QCIRCLES. That leaves 180 for the pointy parts. 90 each.", "So we should do RIGHT 90 at each point.", "Let's try."))
    + page(84, "A WORKING PROCEDURE: PETAL draws a petal, and FLOWER four of them, like a propeller", "### A Working Procedure\n\n"
        + code("TO PETAL\n\nQCIRCLE 50\nRIGHT 90\nQCIRCLE 50\nRIGHT 90\nEND") + "\n\n" + said("Four make a flower.") + "\n\n"
        + code("TO FLOWER\n\nPETAL\nRIGHT 90\nPETAL\nRIGHT 90\nPETAL\nRIGHT 90\nPETAL\nRIGHT 90\nEND") + "\n\n" + said("That's more like a propeller.", "So try ten."))
    + page(85, "A BUILDING BLOCK: NEWFLOWER draws ten petals, a flower", "### A Building Block\n\n" + said(
        "Typing all that ten times hurts my fingers.", "We can use REPEAT.") + "\n\n"
        + code("TO NEWFLOWER\nREPEAT 10\n   PETAL\n   RIGHT 360/10\nEND") + "\n\n" + said("There it is!", "But it's too big.", "All we have to do is change the 50 in PETAL. Make it 25."))
    + page(86, "BUILDING UP: PLANT draws the flower on a stem with a leaf", said(
        "If we let PETAL have an input we can make big or small flowers.", "That's easy. Just do TO PETAL SIZE QCIRCLE SIZE, and so on.",
        "But I bet we'd get bugs if we try that. Let's try plain 25 first.", "Then we can make a superprocedure to draw a plant.")
        + "\n\n### Building Up\n\n" + code("TO PLANT\nNEWFLOWER\nBACK 50\nPETAL\nBACK 50\nEND"))
    + page(87, "ENDS BECOME MEANS: SLIDE moves the Turtle sideways without drawing; TRYING THE NEW TOOL: two plants side by side", "### Ends Become Means\n\n" + said(
        "I have a great procedure for putting several together. It's called SLIDE. You just go, PLANT SLIDE PLANT SLIDE PLANT SLIDE.")
        + "\n\n" + code("TO SLIDE DISTANCE\nPEN UP\nRIGHT 90\nFORWARD DISTANCE\nLEFT 90\nPEN DOWN\nEND") + "\n\n### Trying the New Tool")
    + page(88, "Plants of different sizes, and then a garden of many flowers", said(
        "It would be better with small ones and big ones.", "So, change the procedure to accept inputs.", "And if we use RANDOM we can make a garden."))
    + page(89, "Screens of the GARDEN", said("My next project is a flock of birds.", "Maybe we'll put the birds and flowers together.", "Maybe."))
    + page(90, "SERENDIPITY: FLOCK draws six birds, each turned a quarter turn from the last, in a pinwheel", "### Serendipity\n\n" + said(
        "Make a flock by doing BIRD SLIDE BIRD SLIDE.", "I want six birds, and I'm going to use REPEAT.") + "\n\n"
        + code("TO FLOCK\n\nREPEAT 6\n   BIRD\n   SLIDE 20\nEND") + "\n\n" + said(
        "That's funny. I wanted 6 birds all the same way up.", "But it's neat. If we debug it, we should keep a copy like this.")
        + "\n\n" + code("BIRD\nSLIDE\nBIRD") + "\n\n" + said(
        "Walk through it like the Turtle.", "It starts facing north . . . draws a bird . . . now it's facing east . . . that's the bug.", "And the fifth is on top of the first."))
    + page(91, "DEBUGGING: the new FLOCK draws six birds in a diagonal line", "### Debugging\n\n" + said(
        "If you want to fix the bug, bring the Turtle around to face north after doing the bird.", "And let's make them smaller.") + "\n\n"
        + code("TO FLOCK\n\nRIGHT 20\nREPEAT 6\n   BIRD\n   LEFT 90\n   SLIDE 20\nEND") + "\n\n" + said("Here's the flock.") + "\n\n### The End\n\n**And . .**")
    + page(92, "Several flocks of birds flying in rows", said(
        "It's not finished. Let's give the flock inputs and put several together.") + "\n\n" + said(
        "How can we make them fly?", "I found something neat. In BIRD use SPIN instead of RIGHT . . . it's got bugs, but it is pretty."))
    + "<!-- p. 93 -->\n\n### . . . A Beginning"
)

FIXES = [
    ("far more advanced o n e s ~ t h e flowers", "far more advanced ones—the flowers"),
    ("sum of all turns will be 360 degrees. 5", "sum of all turns will be 360 degrees.<sup>5</sup>"),
    ("Theorem is more powerful: The child", "Theorem is more *powerful*: The child"),
    ("it is more general: It applies", "it is more *general*: It applies"),
    ("it is more intelligible: Its proof", "it is more *intelligible*: Its proof"),
    (r"re: <!-- p\. 77 --> What follows is a hypothetical conversation.*?<!-- p\. 93 --> • \. \. A   BEGINNING",
     "\n\n" + STORY),
]
