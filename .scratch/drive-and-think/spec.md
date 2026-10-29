# Drive Kame, and Show Kame thinking

Two ideas from `mindstorms/mindstorm-implementation-plan.md`, built as one way of playing: **Drive Kame** moves and turns Kame straight from the keyboard, the floor turtle's button box, and types each move into the console as the Logo it stands for. **Show Kame thinking** draws what the numbers mean on the canvas as Kame moves: a protractor arc with its number on a turn, and a ruler with the length so far on a move. Driving always shows them, and they can be switched on for any program too.

Learners find shapes with their hands, see the Logo for each move and the angle or length it means, then use Teach Kame on it. Seeing 90 as a quarter, 120 as a third and 45 as half of 90 is how the numbers get their meaning.

There are no new Logo commands. The UI design is on a canvas of mockups: https://claude.ai/artifact/TJWwqBsxSmpwcC7HvXWUVD (private to the owner unless shared).

## What "done" looks like

- **Show Kame thinking:**
  - a toolbar toggle, **Show thinking**, off at first;
  - on a turn (`RT`, `LT`, and the turn in `SETHEADING`), a faint wedge from the old heading to the new one, growing as Kame turns, with the angle turned beside it: `90°`. Faint lines along the old and new headings show where it started and where it ends;
  - on a move (`FD`, `BK`, and `SETPOS`'s move), a faint ruler alongside the line, with a tick every 10 steps and a longer one every 50, growing as Kame moves, with the length so far at its end: `60`;
  - each one stays until the next instruction that moves or turns Kame starts, then fades over a third of a second;
  - only at animated speeds: at Instant, nothing is shown, since a thousand arcs a second would be noise;
  - never in Export, and never on the drawing itself: it's drawn over the canvas, like Kame.
- **Drive Kame:**
  - a toolbar toggle, **Drive**, and Esc to stop driving. Run, Pause and Step stop it too;
  - while driving, a strip across the top of the canvas says what the keys do, with a choice of step for moves (1, 5, **10**, 25, 50) and for turns (1, 5, **15**, 30, 45, 90);
  - ↑ `FD`, ↓ `BK`, ← `LT`, → `RT` by the chosen step; Space `PU` or `PD`, whichever changes the pen; Backspace or Ctrl+Z is Undo;
  - each press runs as a console entry, so Undo and Teach Kame work on it as on anything typed. Presses of the same key in a row join up into one entry, `FD 30` rather than three `FD 10`s, and the console's last line changes to match. Any other key, typing in the console, or stopping driving starts a new entry;
  - moves and turns while driving play at a speed of their own, quick enough to feel direct (`FD 10` and `RT 15` each take about a tenth of a second), whatever the speed slider says. Presses made while one is playing wait their turn, so holding a key down moves Kame smoothly;
  - Kame thinking is always shown while driving. A faint line also runs out in front of Kame, and the arc of the turn so far in this entry shows with its number, with Kame's heading under it: `RT 45 · heading 135`;
  - the console's input line and the editor work as usual; the arrow keys drive only while the canvas has focus, which Drive gives it.

## Decisions

- **Thinking is the animator's to know, the canvas's to draw.** The animator already knows the turn or move in progress (`Current::Turn` and `Current::Move`); it says what to show in each frame, with what was last shown fading, and the canvas draws it over the drawing as it draws Kame. No interpreter changes.
- **The arc's radius and the ruler's offset are in screen pixels,** 40 and 10, so they read the same at any zoom. Ticks too close together, under 4 pixels, are dropped to every 50, then every 100.
- **A turn of more than 360°** draws the arc for what's left over a whole turn, with the full number: `RT 450` shows a quarter with `450°`.
- **Driving entries are console entries,** with one new job: carrying on an entry runs without saving Kame for Undo, as Teach Kame's definition does (`define_console`), and changes the entry's text in the console's record and on its line. Undo then takes back the whole joined entry.
- **Driving has its own speed,** 100 steps a second and 150° a second, so the slider stays the program's.
- **Joining:** only presses of the same key, so `FD 10` then `BK 10` are two entries, and Teach Kame keeps both. Nothing is joined across Space, Undo, or anything typed.
- **Pen:** Space runs `PU` or `PD`. Kame shows the pen is up as the canvas already does; the strip says `Pen up` too.
- **Settings:** Show thinking and the two steps last for the session, not saved, like the speed slider.
- **An ADR** records that driving is console entries and that thinking is never part of the drawing.

## How the work is split

| Issue | What | Blocked by |
|---|---|---|
| 01 | The animator says what to think: the turn or move in progress, the last one fading, nothing at Instant | |
| 02 | The canvas draws it: arcs, rulers, labels, the heading line; the Show thinking toggle | 01 |
| 03 | Driving: the toggle and strip, the keys, joining presses into entries, its own speed, the ADR | 02 |
| 04 | Session checks and screenshots for both | 03 |
