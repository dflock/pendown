# Chapter 7, pp. 162-175: checked against PDF pages 171-184. The agents'
# subscripts (pp. 167-170) were unreadable to the OCR, so those paragraphs
# are written out from the page images.
H, W, Hi, G = "A<sub>height</sub>", "A<sub>width</sub>", "A<sub>history</sub>", "A<sub>geom</sub>"

FIXES = [
    ('in the most superficial sense" the same principle (e.g.', "in the most superficial sense: the same *principle* (e.g."),
    (r're:The first, A ho~,~,judges .*? Rather, it is fanatically dedicated',
     f"The first, {H} judges the quantity of liquids and of anything else by its vertical extent. {H} is a practical agent in the life of the child. "
     "It is accustomed to comparing children by standing them back to back and of equalizing the quantities of Coca-Cola and chocolate milk in children's glasses. "
     f'We emphasize that {H} does not do anything as complicated as "perceive" the quantity of liquid. Rather, it is fanatically dedicated'),
    (r"re:There is a second agent, called A~,~, that .*?less \"influential\" than A—o,,,,\.",
     f'There is a second agent, called {W}, that judges by the horizontal extent. It is not as "practiced" as {H}. It gets its chance to judge that there is a lot of water in the sea, '
     f'but in the mind of the child this principle is less "influential" than {H}.'),
    (r"re:Finally, there is an agent called A,~,o~, that .*? A ~,or, has",
     f"Finally, there is an agent called {Hi} that says that the quantities are the same because once they were the same. {Hi} seems to speak like a conservationist child, but this is an illusion. {Hi} has"),
    ("As we know, A ~o,,~,'s voice", f"As we know, {H}'s voice"),
    (r"re:for this change to take place\. A—o~, and A ~ could .*?if so, in which direction\.",
     f'for this change to take place. {H} and {W} could become more "sophisticated," so that, for example, {H} would disqualify itself except when all other things are equal. '
     f'This would mean that {H} would only step forward to judge by height those things that have equal cross sections. Second, there could be a change in "seniority," in prerogative: '
     f"{Hi} could become the dominant voice. Neither of these two modes of change is impossible. But there is a third mode that produces the same effect in a simpler way. "
     f"Its key idea is that {H} and {W} neutralize one another by giving contradictory opinions. The idea is attractive (and close to Piaget's own concept of grouplike compositions of operations) "
     "but raises some problems. Why do all three agents not neutralize one another so that the child would have no opinion at all? The question is answered by a further postulate "
     "(which has much in common with Piaget's idea that intellectual operators be organized into *groupements*). "
     f"The principle of neutralization becomes workable if enough structure is imposed on the agents for {H} and {W} to be in a special relationship with one another but not with {Hi}. "
     "We have seen that the technique of creating a new entity works powerfully in programming systems. And this is the process we postulate here. A new entity, a new agent comes into being. "
     f'This is {G}, which acts as the supervisor for {H} and {W}. In cases where {H} and {W} agree, {G} passes on their message with great "authority." '
     f"But if they disagree, {G} is undermined and the voices of the underlings are neutralized. "
     f'It must be emphasized that {G} is not meant to "understand" the reasons for decision making by {H} and {W}. {G} knows nothing except whether they agree and, if so, in which direction.'),
    ("might like to impose on A ho—t and A w—dt—a superior agent", f"might like to impose on {H} and {W} a superior agent"),
    ("Our A,oombelongs firmly", f"Our {G} belongs firmly"),
    ("something like A,~omcan become important", f"something like {G} can become important"),
    ("orderliness of p's implying q's.", "orderliness of *p's* implying *q's.*"),
    ("we don't have to reorganize", "we *don't* have to reorganize"),
    ("Claude L6vi-Strauss 2 has spoken", "Claude Lévi-Strauss<sup>2</sup> has spoken"),
    ("we, as learners, are all bricoleurs. 8 This", "we, as learners, are all *bricoleurs.*<sup>3</sup> This"),
    ("If bricolage is a model", "If *bricolage* is a model"),
    ("respect for ourselves as bricoleurs.", "respect for ourselves as *bricoleurs.*"),
    ("> 1. Separate the beads into colors. 2. Choose a color A as color 1. 3. Form all the pairs that can be formed with color 1. 4. Choose color 2. 5. Form all the pairs that can be formed with color 2. 6. Do this for each color. 7. Go back and remove the duplicates.",
     "1. Separate the beads into colors.\n2. Choose a color A as color 1.\n3. Form all the pairs that can be formed with color 1.\n4. Choose color 2.\n5. Form all the pairs that can be formed with color 2.\n6. Do this for each color.\n7. Go back and remove the duplicates."),
    ("little opportunity for bricolage with", "little opportunity for *bricolage* with"),
    ("procedure is poorly represented. 4", "procedure is poorly represented.<sup>4</sup>"),
    ("societies that numerical knowledge", "societies that *numerical* knowledge"),
]
