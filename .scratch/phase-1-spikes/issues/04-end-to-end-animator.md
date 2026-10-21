# Spike: interpreter thread, channel and animator end to end

Status: done
Type: prototype
Blocked by: 01, 02, 03

## Question

Does the turtle speed design in `implementation-plan.md` work as the playground's headline feature: speed control, Pause, Step and Stop, with the running instruction highlighted?

## Build

Connect spikes 01, 02 and 03: the interpreter on its own thread with a large stack, a bounded channel of events, and an animator on the UI thread driven by `request_animation_frame`. Add a speed slider, Pause/Resume, Step and Stop buttons, and the `Instruction` highlight in the editor.

## Answer these

1. **Endless program:** with `FOREVER [FD 1 RT 1]`, does Stop end it immediately, including while the interpreter is blocked on a full channel?
2. **Huge program:** with a fractal of 100k+ segments in Instant mode, does the UI stay responsive, and how long does it take to draw?
3. **Stepping:** does Step stop exactly where the highlight is, with no instructions run ahead of it?
4. **Highlighting:** at slow speeds, does the highlight move through `REPEAT 4 [FD 50 RT 90]` instruction by instruction, in step with the turtle?
5. **Speed changes** take effect immediately, even in the middle of a move.
6. **Channel size:** try a few sizes and recommend one (an open question in `implementation-plan.md`).
7. **Speed units:** which feels better to use: levels from 0 to 10, or a slider in turtle steps per second (also an open question)?

## Done when

The questions are answered in a comment below, and the open questions in `implementation-plan.md` are updated with the answers.

## Comments

### 2026-10-21: spike results

Code: `spikes/04-end-to-end-animator/`. It uses the `pendown-logo` crate from spike 03, extended with:
- a `Control` shared with the GUI (cancel, lockstep, and a count of lines started);
- a `Sync` event;
- `PU`, `PD` and `SETPC`.

Run it:
- **The window:** `cargo run --release`. It has Run, Pause/Resume, Step, Step instruction and Stop, two speed sliders, and five example programs.
- **The scripted session:** `SPIKE_AUTOTEST=1` runs the session that answers the questions below. It writes `autotest.log` and the screenshots.
- **The animator's tests:** `cargo test --release` runs 12 tests in `tests/animator.rs`. They drive the animator and a real interpreter thread, without a window.

How it's built:
- **The interpreter** runs on its own thread with a 256 MiB stack. It sends batches of events (spike 03) over a bounded channel.
- **The animator** (`src/animator.rs`) is a plain state machine on the UI thread, with no gpui code. Each frame it plays events at the current speed and hands back:
  - the moves that finished, which are drawn into the vello_cpu pixel buffer and uploaded as dirty tiles (ADR 0010);
  - console output;
  - the highlight.
  
  The move in progress and the turtle are painted as paths.
- **Lockstep (ADR 0012), as built:** when the animator sets `Control::lockstep`, the interpreter sends a `Sync` after the first `Instruction` of each line, and waits for a reply. The animator holds the `Sync` while paused.

**Summary: yes, the design works.** It needed three changes:
1. **Lockstep at every animated speed,** not only while paused, so that Pause is exact.
2. **Instant mode keeps taking events for its whole 8 ms slice,** waiting for the interpreter when the channel is empty. Without that, the channel's size capped Instant at a channelful a frame, and the 131k-move tree took 9 s instead of 0.2 s.
3. **A brief dwell on instructions that don't move the turtle,** so the highlight is visible on them at slow speeds.

Measurements: Intel HD 530, 8 CPUs, X11, release build. Every frame interval below is 16.7 ms p50.

#### 1. Endless program and Stop

`FOREVER [FD 3 RT 1.5]` stops at once in every state. The times run from pressing Stop to the interpreter thread exiting:

| State | Time to stop |
|---|---|
| Running at Instant | 0.008 ms |
| At 20 steps/s, interpreter blocked on the full channel | 0.039 ms |
| Paused, interpreter waiting at a held line | 0.034 ms |
| Paused after a pause from Instant | 0.071 ms |

Stop sets the cancel flag, then drops the channel and any held `Sync`. Whichever of them the interpreter is blocked on fails, and it ends with `Halted`. The animator tests check the same three states without a window, at 2–32 µs.

#### 2. Huge program

`TREE 230 16` draws 131,070 moves from 983,033 events at Instant.

- **It finishes in 0.16–0.31 s with a batched channel,** and in 0.68 s on the first run after the app starts. The UI stays responsive:
  - frames are 16.7 ms p50 and at most 33 ms;
  - the animator takes at most 8.5 ms a frame;
  - vello_cpu draws about 0.35–0.55 µs a segment, capped at about 6 ms a frame by an adaptive limit.
  
  `screenshot-3-tree-instant.png` shows it.
- **Resizing** re-renders all 131,070 segments in 42–50 ms, so ADR 0003's "background re-render later" stays the right call.
- **Instant mode must wait out its slice.**
  - The first version took whatever was in the channel and stopped. A channel of 4 × 256 events then delivered about 1,000 events a frame, and the tree took 8.9 s with the UI thread idle.
  - Instant mode now waits on the channel (`recv_timeout`) until its 8 ms slice is used up, and that's what gives the times above.
  - It costs nothing when the program is fast. A slow program just leaves the UI thread waiting part of each 8 ms.
- **One full session had 33–50 ms frames throughout its later phases.** I couldn't reproduce it in three later runs, including the same sequence of runs; it looked like load from outside the spike. Worth watching in the real app.

#### 3. Stepping

- **Step is exact.** After every Step, the interpreter has started exactly as many lines as the highlight has reached. The animator test checks this after every step of a multi-line program, with a 20 ms pause between steps to give the interpreter every chance to run ahead. Step stops at the start of each line, and console output appears only when its `PRINT` line runs.
- **Pausing mid-line at an animated speed** leaves the interpreter waiting at the start of the next line. It has sent that line's `Instruction` and is blocked on its `Sync`, but hasn't run any of it. The log shows "1 ahead" in that case, and then "0 ahead" after each Step.
- **That needs lockstep at all animated speeds, not just while paused.**
  - Turning lockstep on only while paused, as ADR 0012 says, leaves the interpreter as far ahead as the channel lets it, up to about 1,000 events.
  - With lockstep at animated speeds, a pause lands within the current line.
  - Lockstep costs a round trip per line, about 10 µs. The animator waits up to 4 ms a frame for the next line's events after replying, so lockstep allows about 25,000 lines a second: 20,000 lines took 0.71 s at 1,000,000 steps/s, against 0.02 s at Instant. That's far faster than anyone can watch, so only Instant runs free.
- **Pausing from Instant** leaves the interpreter ahead: 171 lines in the GUI session, 742 in a test.
  - The first Step plays those lines, which have already run, until it reaches the first held line. After that, stepping is exact.
  - That's unavoidable without lockstep at Instant, which would cost Instant mode most of its speed (`FIB` runs a line per instruction).
- **How Step works:** Step releases the held `Sync`, plays events at the current speed, and holds the next `Sync`. The one subtlety is the `Sync` belonging to the line the highlight is already on, which must be released rather than held. The animator recognises it as a `Sync` arriving straight after a line-start `Instruction` that was shown before this step began.
- **Step instruction** stops at every instruction, including inside brackets. One-line loops need it:
  - `REPEAT 4 [FD 10 RT 90]` on one line is one Step, as the plan says.
  - `FOREVER [FD 1 RT 1]` on one line never reaches another line, so Step would never stop. Step instruction goes `FD 1`, `RT 1`, `FD 1`… in the test.

#### 4. Highlighting

- **At 100 steps/s, through `REPEAT 4 [FD 300 RT 90]`, the highlight moves instruction by instruction, in step with the turtle.** `autotest.log` logs each change with the turtle's position:
  - `REPEAT` at 0.03 s;
  - `FD 300` at 0.13 s, with the turtle just leaving (0, 0);
  - `RT 90` at 3.16 s, with the turtle at (0, 300);
  - `FD 300` at 3.46 s, heading 90.
- **While the turtle moves along a line, the highlight is on that line's `FD`.** The test checks this on every frame of the move. `screenshot-1-highlight-mid-move.png` shows it.
- **Dwell:** an instruction that moves nothing (`REPEAT`, `MAKE`, a procedure call) would otherwise flash past at any speed. The highlight now stays on it for the time it takes to move 10 steps, at most 0.25 s: 0.1 s at 100 steps/s, and nothing noticeable at fast speeds. That's the gap between `REPEAT` and `FD 300` above.
- **The editor is read-only while running,** using gpui-kit's `Editor::readonly`, so spans stay valid (ADR 0006).
- **Runtime and parse errors** show as diagnostics on the code and in the console (ADR 0005).

#### 5. Speed changes

- **A speed change applies on the very next frame, even mid-move.**
  - In `FD 600`, the speed went from 50 to 200 steps/s at 2.0 s. The frames read 50, 50, then 200, 200, 200 steps/s.
  - The test checks the same with exact steps: 10 steps a frame, then 40, and a change to Instant finishes the move at once.
- The animator keeps the move's progress in steps, and each frame turns elapsed time into steps at the current speed, so nothing jumps.

#### 6. Channel size

Times for `TREE 230 16` at Instant, from the last full session:

| Batches × channel | Time |
|---|---|
| 1 event × 1,024 | 0.42 s |
| 64 × 4 | 0.31 s |
| **256 × 4** | **0.18 s** |
| 1,024 × 4 | 0.16 s |
| 256 × 16 | 0.16 s |
| 256 × 32 | 0.16 s |
| 1,024 × 16 | 0.16 s |

- **Recommendation: keep spike 03's batches of 256, with a channel of 4 batches.**
  - Once Instant waits out its slice, larger channels gain only about 0.02 s.
  - With lockstep, the channel no longer sets how far ahead the interpreter gets at animated speeds; each line start does.
  - A small channel also keeps memory down for endless programs at Instant.
- **Before the Instant fix, size mattered a lot,** from 9 s down to 1.1 s. That was the symptom that led to the fix.

#### 7. Speed units

- **The window has both, linked:**
  - a **level** slider from 1 to 10, where 10 is Instant and each level doubles the speed: 5, 10, 20 … 1,280 steps a second;
  - a **steps per second** slider, logarithmic from 5 to 5,000, with Instant at its top end.
- **My recommendation: levels from 1 to 10.**
  - They're easy to say ("try speed 3") and easy to put in a program: `SETSPEED 3` reads better than `SETSPEED 160`.
  - Doubling each level covers the whole useful range in ten steps.
  - Instant is simply the top level.
  - Show the steps per second next to it, so the scale isn't a mystery.
  
  But this one needs a person to try both, so it's a decision for you.

#### Changes this suggests

- **ADR 0011:**
  - Lockstep at every animated speed, and while paused or stepping. Only Instant, running, lets the interpreter run ahead.
  - Instant takes events for its whole slice, waiting on the channel if it's empty, and draws at most about 6 ms of segments a frame.
  - A dwell on instructions that move nothing.
  - The channel stays at 4 batches of 256.
- **ADR 0012:**
  - Step holds and releases `Sync`s, as described above.
  - Keep both Step (a line) and Step instruction: one-line loops can only be stepped by instruction.
  - Pausing from Instant can leave the interpreter ahead, and the first Step catches up.
- **`implementation-plan.md`, open questions:**
  - Channel size: answered.
  - "What does Step step over": answered; both buttons.
  - Speed units: waits for your decision.

#### For a human to decide

- **Speed units.** Try both sliders in the window; the recommendation is levels from 1 to 10.
- **Is "the first Step after pausing from Instant catches up" acceptable?** The alternative is lockstep at Instant too, which makes Instant several times slower for programs with many short lines.

#### Not covered

- Input sync points (`READLIST`, the mouse), `WAIT` and `SETSPEED`. The `Sync` mechanism they need is now built and tested.
- The console's own input line.
- A real HiDPI display, Wayland, and other GPUs.

### 2026-10-21: closed

Decisions:
- **Speed levels run from 1 to 11, with 11 as Instant.** Below it each level doubles the speed, from 5 steps a second at level 1 to 2,560 at level 10. The window now has just the level slider, and shows the steps per second in its status line.
- **Pausing from Instant may leave the interpreter ahead,** and the first Step catches up. That's fine for now.

Updated:
- **ADR 0011:** lockstep at animated speeds, speed levels, Instant waiting out its slice, the dwell, and the channel size.
- **ADR 0012:** lockstep, how Step holds and releases `Sync`s, Step instruction, and pausing from Instant.
- **`implementation-plan.md`:** the speed model, the controls, how stepping works, and the three open questions it answers (speed units, Step, channel size).
