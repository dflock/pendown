## Proposed Button States:

Initial State, nothing running: Run, Pause (disabled), Single Step (disabled)
After clicking run:             Restart, Pause, Single Step (disabled)
When paused:                    Restart, Resume, Single Step
When on a breakpoint:           Restart, Resume, Single Step

So, while paused the "Run" button becomes "Restart", with a different icon. This describes what that button does better, and you done get [Run] [Resume] next to each other with the same icon, which is confusing.

## Current Line Gutter Highlight

I want the current line higlighted in the gutter too, not just in the editor - maybe a green background behind the line number, with a taper to the right like a chevron? Not just a green pill - must also be a different shape. If that line is also a breakpoint, draw the current line marker, the red pill, then the number on top.
