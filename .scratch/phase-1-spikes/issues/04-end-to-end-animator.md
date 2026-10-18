# Spike: interpreter thread, channel and animator end to end

Status: ready-for-agent
Type: prototype
Blocked by: 01, 02, 03

## Question

Does the turtle speed design in `plan-1.md` work as the playground's headline feature: speed control, Pause, Step and Stop, with the running instruction highlighted?

## Build

Connect spikes 01, 02 and 03: the interpreter on its own thread with a large stack, a bounded channel of events, and an animator on the UI thread driven by `request_animation_frame`. Add a speed slider, Pause/Resume, Step and Stop buttons, and the `Instruction` highlight in the editor.

## Answer these

1. **Endless program:** with `FOREVER [FD 1 RT 1]`, does Stop end it immediately, including while the interpreter is blocked on a full channel?
2. **Huge program:** with a fractal of 100k+ segments in Instant mode, does the UI stay responsive, and how long does it take to draw?
3. **Stepping:** does Step stop exactly where the highlight is, with no instructions run ahead of it?
4. **Highlighting:** at slow speeds, does the highlight move through `REPEAT 4 [FD 50 RT 90]` instruction by instruction, in step with the turtle?
5. **Speed changes** take effect immediately, even in the middle of a move.
6. **Channel size:** try a few sizes and recommend one (an open question in `plan-1.md`).
7. **Speed units:** which feels better to use: levels from 0 to 10, or a slider in turtle steps per second (also an open question)?

## Done when

The questions are answered in a comment below, and the open questions in `plan-1.md` are updated with the answers.

## Comments
