# The first fix

Status: done
Type: task
Blocked by: 04

## What

Whatever the numbers show. Expected, from the first look: every frame re-renders the whole window. Make the editor and console cached views, so a frame only rebuilds the toolbar and repaints the canvas. Also check the canvas uploads tiles only when they change.

## Done when

A slow run costs little more than an idle window, by the summary and the overlay, and Instant is as fast as before by `perf` and the scripted session.

## Comments

### 2026-10-22: done

What the numbers showed (release, 60 Hz, 8 CPUs; the scripted session's speed 3 run of the welcome program):

| | renders/s | UI CPU a frame | UI CPU |
|---|---|---|---|
| Before | 60 | 3.3–4.6 ms | 22–27% |
| After | 12 | | 4–6%, some of it the session's own timer |

Instant is unchanged: the 131,070-line tree takes 0.62–0.74 s, as on main.

Four changes, in order of what they saved:
1. **Frames only when the picture changes.** `Animator::next_change` says how long until the drawing or the turtle moves about a device pixel, or a dwell or `WAIT` ends. The window asks for an animation frame only when that's within 12 ms, and otherwise sets a timer, at most 100 ms away (`MAX_FRAME_GAP`). At speed 3 the turtle moves under a pixel a frame, so most frames looked the same.
2. **The editor no longer springs back after scrolling past its end.** Highlighting a line near the bottom of a short program scrolled past the end of the text. The editor then animated an overscroll spring for about a third of a second, asking for a frame each time, so the whole window ran at 60 fps again after each highlight change. Now it scrolls only where there's more text, and never past the end.
3. **The editor, the console and the toolbar are cached views** (`panes.rs`), so a frame doesn't lay them out again unless they changed. That's about 1 ms of each frame at 60 fps. The status beside the toolbar has a fixed width, so its changing text doesn't resize the toolbar and redraw it.
4. **The scripted session runs on a 5 ms timer** instead of asking for a frame every time, so it doesn't change how often the window renders. It records the turtle's position at each frame, as before.

The floor: an empty frame, with nothing but cached views, costs gpui about 2.2 ms of UI CPU here, so drawing fewer frames is what matters most.
