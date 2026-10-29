# Chapter 5, p. 134, and Chapter 6, pp. 135-147: checked against PDF pages 143-156.
FIXES = [
    ("technical knowledge as a m e a n s to get", "technical knowledge as a *means* to get"),
    ('> "I love your microworlds but is it physics? I don\'t say it is not. But how can I decide?"\n\n> - - A teacher\n',
     '> "I love your microworlds but is it physics? I don\'t say it is not. But how can I decide?"\n>\n> —*A teacher*\n'),
    ("ACOMMONDISTINCTION between", "A COMMON DISTINCTION between"),
    ("They talk about getting to know an idea, exploring an area of knowledge, and acquiring sensitivity to",
     "They talk about *getting to know* an idea, *exploring* an area of knowledge, and *acquiring sensitivity* to"),
    ("given such equations as f = m a , E = I R , o r P V = R T as", "given such equations as *f* = *ma*, *E* = *IR*, *or PV* = *RT* as"),
    ('among them" What is physics?', "among them: What *is* physics?"),
    ('cleverer than ARI" He knows something', "cleverer than ARI: He *knows* something"),
    ("It is an idea, an intellectual tool", "It is an *idea,* an intellectual tool"),
    ("acquired by adult physicists", "acquired by *adult* physicists"),
    ("an equation to prove that the thing", "an equation to *prove* that the thing"),
    ("He already knew that it would", "He already *knew* that it would"),
    ("By proving that it will stand", "By *proving* that it will stand"),
    ("intuitive thinking: how he thinks about objects.", "intuitive thinking: *how he thinks about objects.*"),
    ("children do to confront their intuitions?", "children do to confront *their* intuitions?"),
    ("a situation where your intuitions", "a situation where *your* intuitions"),
    ('know how to calculate the answer. But before doing so or reading on try to guess" Is it',
     'know how to *calculate* the answer. But before doing so or reading on try to guess: Is it'),
    (r"re:> GAL: Look, your theory has got to be wrong\..*?> laws of falling bodies\?\n",
     "> GAL: Look, your theory has got to be wrong. Here's a two-pound and a one-pound ball. The two-pound ball takes two seconds to fall to the ground. Tell me, how long do you think the one-pound ball would need?\n>\n"
     "> ARI: I suppose it would take four seconds. Anyway, much more than two seconds.\n>\n"
     "> GAL: I thought you would say that. But now please answer another question. I am about to drop two one-pounders simultaneously. How long will the pair of them take to reach the ground?\n>\n"
     "> ARI: That's not another question. I gave my opinion that one-pound balls take four seconds. Two of them must do the same. Each falls independently.\n>\n"
     "> GAL: You are consistent with yourself if two bodies are two bodies, not one.\n>\n"
     "> ARI: As they are . . . of course.\n>\n"
     "> GAL: But now if I connect them by a gossamer thread . . . is this now two bodies or one? Will it (or they) take two seconds or four to fall to the ground?\n>\n"
     "> ARI: I am truly confused. Let me think. . . . It's one body, but then it should fall for four seconds before reaching the earth. But then this would mean that a thread finer than silk could slow down a furiously falling ball of iron. It seems impossible. But if I say it is two bodies . . . I am in deep trouble. What is a body? How do I know when one becomes two? And if I cannot know then how sure can I be of my laws of falling bodies?\n"),
    ("GAL ' s a r g u m e n t is not", "GAL's argument is not"),
    ("for ARI ' s theory.", "for ARI's theory."),
    (r"re:\n\nFigure 15\n\nThe figure shows a string around the earth.*?which is simply 2:h\.\n\nBut the challenge here is to \"intuit\" an approximate answer rather than to \"calculate\" an exact one\.\n",
     "\n\n![Figure 15: a circle, the earth, inside a larger circle, the string, held up on poles](images/p146-figure-15.png)\n\n**Figure 15**\n\n"
     "> **The figure shows a string around the earth supported by poles of greatly exaggerated height. Call the radius of the earth *R* and the height of the poles *h*. "
     "The problem is to estimate the difference in length between the outer circumference and the true circumference. This is easy to calculate from the formula:**\n>\n"
     "> **CIRCUMFERENCE = 2π × RADIUS**\n>\n> **So the difference must be**\n>\n> **2π(*R*+*h*) − 2π*R***\n>\n> ***which is simply 2πh.***\n>\n"
     "> **But the challenge here is to \"intuit\" an approximate answer rather than to \"calculate\" an exact one.**\n"),
    ("before calculati n g ~ a discipline", "before calculating—a discipline"),
    (r"re:\n\nJ Figure 16a\n\nThe string on poles is assumed to be at distance h from the square\.(.*?)that is to say 2—rh\.",
     "\n\n![Figure 16a: a square inside a square with rounded corners, the string on poles, with lines marking off the corners](images/p147-figure-16a.png)\n\n**Figure 16a**\n\n"
     "> **The string on poles is assumed to be at distance *h* from the square. Along the edges the string is straight. As it goes around the corner it follows a circle of radius *h*. "
     "The straight segments of the string have the same length as the edges of the square. The extra length is all at the corners, in the four quarter-circle pie slices. "
     "The four quarter circles make a whole circle of radius *h*. So the \"extra string\" is the circumference of this circle, that is to say 2π*h*.**"),
]
