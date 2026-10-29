# Epilogue, pp. 190-203: checked against PDF pages 199-212. The equations
# are written out from the page images; the OCR of them is unusable.
P2, Q2, R2 = "*p*<sup>2</sup>", "*q*<sup>2</sup>", "*r*<sup>2</sup>"
EQ = f"{P2} = 2{Q2}"

FIXES = [
    ('REPRINTEDHERE as an epilogue is my first discussion, written a few years ago, o f an idea that developed into a central theme of this book." My rejection o f the dichotomy opposing a stereotypically "\'disembodied" mathematics to activities engaging a full range o f human sensitivities.\\* In the book I discuss this theme in the context of Turtle geometry. In the following pages the reader will find this theme embedded in reflections on the sources of mathematical pleasure.',
     '*REPRINTED HERE as an epilogue is my first discussion, written a few years ago, of an idea that developed into a central theme of this book: My rejection of the dichotomy opposing a stereotypically "disembodied" mathematics to activities engaging a full range of human sensitivities.\\* In the book I discuss this theme in the context of Turtle geometry. In the following pages the reader will find this theme embedded in reflections on the sources of mathematical pleasure.*'),
    ("Wechsler, ed., Aesthetics In Science", "Wechsler, ed., *Aesthetics In Science*"),
    ("The •following paragraphs", "The following paragraphs"),
    ("we have set up the equation:\n\nk/ 2 = p/q    where p and q are whole numbers\n",
     "we have set up the equation:\n\n√2 = *p*/*q*   where *p* and *q* are whole numbers\n"),
    ("we do not really believe that X/~ can be so expressed", "we do not really believe that √2 can be so expressed"),
    ("\n\n> = p/q V'-~×q=p\n\n> p=~/-2Xq (       =\n\n> 2 = if—q2\n\n> f f = 2q 2\n",
     f"\n\n√2 = *p*/*q*  \n√2 × *q* = *p*  \n*p* = √2 × *q*  \n(√2)<sup>2</sup> = (*p*/*q*)<sup>2</sup>  \n2 = {P2}/{Q2}  \n{EQ}\n"),
    ("appearance of 2 = p2/q2 until this turned into p2 = 2q2.", f"appearance of 2 = {P2}/{Q2} until this turned into {EQ}."),
    ('"subject" actor is V"-2. The other two actors, p and q, are', '"subject" actor is √2. The other two actors, *p* and *q*, are'),
    ("Now p has become the subject, and the previous subject, X/~, has vanished.", "Now *p* has become the subject, and the previous subject, √2, has vanished."),
    ("still visible in p~ = 2q2!", f"still visible in {EQ}!"),
    ('reconciled with existing models', 'reconciled with existing models'),
    ('how Pomcare "     "\' s mathematical aesthetic sentinel', "how Poincaré's mathematical aesthetic sentinel"),
    ("in the assumption V' 2 = p/q.", "in the assumption √2 = *p*/*q*."),
    ("We can read off from p2 = 2qZ that p2 is even. It follows that p is even. By definition this means that p is twice some other whole number which we can call r. So:",
     f"We can read off from {EQ} that {P2} is even. It follows that *p* is even. By definition this means that *p* is twice some other whole number which we can call *r*. So:"),
    ("\n\n> p = 2r pZ = 4r 2 2q z = 4r 2 remember: p2 = 2q2(!) qZ = 2r ~\n",
     f"\n\n*p* = 2*r*  \n{P2} = 4{R2}  \n2{Q2} = 4{R2}   remember: {EQ}(!)  \n{Q2} = 2{R2}\n"),
    ("and we deduce that q is also even. But this at last really is manifestly bizarre since we chose p and q in",
     "and we deduce that *q* is also even. But this at last really is manifestly bizarre since we chose *p* and *q* in"),
    ("perfect square (36 or p2 or q2) as an", f"perfect square (36 or {P2} or {Q2}) as an"),
    ("(such as let p = pap2.., p~, so that p 2 = p—p—p2p2... p—p~)",
     "(such as let *p* = *p*<sub>1</sub>*p*<sub>2</sub> . . . *p*<sub>k</sub>, so that *p*<sup>2</sup> = *p*<sub>1</sub>*p*<sub>1</sub>*p*<sub>2</sub>*p*<sub>2</sub> . . . *p*<sub>k</sub>*p*<sub>k</sub>)"),
    ("yourself to see) p2 and q2 as even sets, you will also see p2 = 2q2 as making the absurd assertion that an even set (pZ) is equal to an odd set (q2 and one additional factor: 2). Thus given the right frames for perceiving numbers, p2 = 2q2 is",
     f"yourself to see) {P2} and {Q2} as even sets, you will also see {EQ} as making the absurd assertion that an even set ({P2}) is equal to an odd set ({Q2} and one additional factor: 2). Thus given the right frames for perceiving numbers, {EQ} is"),
    ("all:6:p2 = 2q2", EQ),
    ('independent of other" components', "independent of other components"),
    ("Pirsig's Zen and the Art of Motorcycle Maintenance", "Pirsig's *Zen and the Art of Motorcycle Maintenance*"),
]
