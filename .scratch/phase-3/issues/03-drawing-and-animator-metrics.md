# Drawing and animator metrics

Status: done
Type: task
Blocked by: 01

## What

- **`Drawing`:** marks drawn, time drawing, tiles taken for upload, re-renders after a resize.
- **`Animator`:** frames, events taken, time in `frame`.

## Done when

Tests check the counts for a fixed drawing and a fixed animated run.

## Comments

### 2026-10-22: done

`DrawingMetrics` in `pendown-render`'s `metrics.rs`, kept by `Drawing`: marks, drawing, re-renders, tiles handed out, PNG encoding. `AnimatorMetrics` in `pendown`'s `metrics.rs`, kept by `Animator`: frames (timed) and events. Tests: the renderer's `metrics_count_marks_tiles_and_rerenders` and the animator's `metrics_count_frames_and_events`.
