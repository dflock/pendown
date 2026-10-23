# Chapter 5, pp. 120-133: checked against PDF pages 129-142.
FIXES = [
    ("Piaget calls assimilation, and", "Piaget calls *assimilation,* and"),
    ("It has m a s s and v e l o c i t y or, if one prefers to merge these two, it has m o m e n t u m .",
     "It has *mass* and *velocity* or, if one prefers to merge these two, it has *momentum.*"),
    ("refuse to recognize as physics. ~", "refuse to recognize as physics.<sup>1</sup>"),
    ("In the absence of direct and physical experiences", "In the absence of *direct* and *physical* experiences"),
    (r"re:\n\n> 0   Every particle continues.*?\n\n3\.       All forces arise from the interaction of particles, and whenever a particle acts on another there is an equal and opposite reaction on the first\.\n",
     "\n\n1. Every particle continues in a state of rest or motion with constant speed in a straight line unless compelled by a force to change that state.\n"
     "2. The net unbalanced force (*F*) producing a change of motion is equal to the product of the mass (*m*) and the acceleration (*a*) of the particle: *F*=*ma*.\n"
     "3. All forces arise from the interaction of particles, and whenever a particle acts on another there is an equal and opposite reaction on the first.\n"),
    (r"re:\n\n> Every Turtle remains in its state of rest until compelled by a\n\n.*?\n\n> Turtle's change of the HEADING part of its state\.\n",
     "\n\n1. Every Turtle remains in its state of rest until compelled by a TURTLE COMMAND to change that state.\n"
     "2. a. The input to the command FORWARD is equal to the Turtle's change in the POSITION part of its state.  \n"
     "   b. The input to the command RIGHT TURN is equal to the Turtle's change of the HEADING part of its state.\n"),
    ("operator, called force. The effect", "operator, called *force.* The effect"),
    ("\n\nVELOCITY TURTLES\n", "\n\n### Velocity Turtles\n"),
    ("\n\nACCELERATION TURTLES\n", "\n\n### Acceleration Turtles\n"),
    ('"Change your velocity by x, no matter', '"Change your velocity by *x,* no matter'),
    ("a feel for why the world works", "a feel for *why* the world works"),
    ("giving commands to one another rather than obeying commands from the outside. They are linked Turtles.",
     "giving commands *to one another* rather than obeying commands from the outside. They are *linked Turtles.*"),
    ("for understanding the idea of a law", "for understanding the *idea* of a law"),
    ("monkey problem. It is a law-of-motion", "monkey problem. It *is* a law-of-motion"),
    ("goes up at the same speed. 2", "goes up at the same speed.<sup>2</sup>"),
    ("spiral in a non-Newtonian microworld", "spiral in a *non-Newtonian* microworld"),
]
