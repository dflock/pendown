# Chapter 1, pp. 34-37, and Chapter 2, pp. 38-47: checked against PDF pages 43-56.
FIXES = [
    ("programming language BASIC 3 has", "programming language BASIC<sup>3</sup> has"),
    ("a self-perpetuating social phenomenon.", "a self-perpetuating *social* phenomenon."),
    ("They have to learn to have trouble", "They have to *learn* to have trouble"),
    (r're:\n\n2 Units   1 Unit I\n\n2 Units •            2 Units         1 Unit FiEure 2          The Conservation of Liquids\n',
     "\n\n![Figure 2: The Conservation of Liquids. Liquid 2 units deep in a glass 2 units wide is poured into a glass 1 unit wide, where it stands 4 units deep.](images/p40-figure-2.png)\n\n"
     "**Figure 2** — *The Conservation of Liquids*\n"),
    ("mere lack of knowledge", "mere *lack* of knowledge"),
    ("what we all forget about our thinking as children is so", "*what we all forget about our thinking as children* is so"),
    ('the "uncoOrdinated" can', 'the "uncoördinated" can'),
    ('"dumb at x" (where, as we have pointed out, x often', '"dumb at *x*" (where, as we have pointed out, *x* often'),
    ("the available evidence. ~ But since", "the available evidence.<sup>1</sup> But since"),
    ("\"math acquisition device. ''2 On", "\"math acquisition device.\"<sup>2</sup> On"),
    ("it is a dissociated model", "it is a *dissociated* model"),
]
