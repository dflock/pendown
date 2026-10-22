//! What the animator and the window measure (ADR 0020), and the summary of a
//! run printed on stderr. The interpreter's metrics are in
//! `pendown_logo::metrics`, the drawing's in `pendown_render::metrics`.

use std::time::{Duration, Instant};

use pendown_logo::metrics::{Counter, InterpreterMetrics, ThreadCpu, Timer};
use pendown_render::DrawingMetrics;

#[derive(Debug, Default)]
pub struct AnimatorMetrics {
    /// Calls to `frame`, each timed.
    pub frames: Timer,
    /// Events taken from the channel.
    pub events: Counter,
}

/// The window's own measurements, kept for its whole life.
#[derive(Debug, Default)]
pub struct WindowMetrics {
    /// Renders of the window's root view, each timed: our code in `render`,
    /// including the animator's frame, but not gpui's layout and paint.
    pub renders: Timer,
    /// Painting the canvas, including drawing new marks into the buffer and
    /// uploading the tiles that changed.
    pub canvas: Timer,
    /// Uploading tiles.
    pub upload: Timer,
    /// Renders of the editor's and the console's views, which should only
    /// happen when they change.
    pub editor_renders: Counter,
    pub console_renders: Counter,
}

/// Every number at one moment, so two can be compared: over a run for the
/// summary, or over the last second for the overlay.
#[derive(Clone, Copy, Debug)]
pub struct Snapshot {
    pub at: Instant,
    pub cpu: ThreadCpu,
    pub renders: u64,
    pub editor_renders: u64,
    pub console_renders: u64,
    pub render: Duration,
    pub canvas: Duration,
    pub upload: Duration,
    pub frames: u64,
    pub animator: Duration,
    pub events: u64,
    pub marks: u64,
    pub drawing: Duration,
    pub tiles: u64,
    pub instructions: u64,
    pub lockstep: Duration,
    pub channel: Duration,
}

impl Snapshot {
    /// The numbers now. The animator and the interpreter's metrics belong to
    /// one job, so they count from zero at its start; the others don't.
    pub fn take(
        window: &WindowMetrics,
        drawing: &DrawingMetrics,
        animator: Option<&AnimatorMetrics>,
        interpreter: Option<&InterpreterMetrics>,
    ) -> Snapshot {
        Snapshot {
            at: Instant::now(),
            cpu: ThreadCpu::now(),
            renders: window.renders.count(),
            editor_renders: window.editor_renders.get(),
            console_renders: window.console_renders.get(),
            render: window.renders.total(),
            canvas: window.canvas.total(),
            upload: window.upload.total(),
            frames: animator.map_or(0, |a| a.frames.count()),
            animator: animator.map_or(Duration::ZERO, |a| a.frames.total()),
            events: animator.map_or(0, |a| a.events.get()),
            marks: drawing.marks.get(),
            drawing: drawing.drawing.total(),
            tiles: drawing.tiles.get(),
            instructions: interpreter.map_or(0, |i| i.instructions.get()),
            lockstep: interpreter.map_or(Duration::ZERO, |i| i.lockstep.total_now()),
            channel: interpreter.map_or(Duration::ZERO, |i| i.channel.total_now()),
        }
    }

    /// What happened between `earlier` and this.
    pub fn since(&self, earlier: &Snapshot) -> Interval {
        Interval {
            wall: self.at.saturating_duration_since(earlier.at),
            cpu: self.cpu.since(&earlier.cpu),
            renders: self.renders.saturating_sub(earlier.renders),
            editor_renders: self.editor_renders.saturating_sub(earlier.editor_renders),
            console_renders: self.console_renders.saturating_sub(earlier.console_renders),
            render: self.render.saturating_sub(earlier.render),
            canvas: self.canvas.saturating_sub(earlier.canvas),
            upload: self.upload.saturating_sub(earlier.upload),
            frames: self.frames.saturating_sub(earlier.frames),
            animator: self.animator.saturating_sub(earlier.animator),
            events: self.events.saturating_sub(earlier.events),
            marks: self.marks.saturating_sub(earlier.marks),
            drawing: self.drawing.saturating_sub(earlier.drawing),
            tiles: self.tiles.saturating_sub(earlier.tiles),
            instructions: self.instructions.saturating_sub(earlier.instructions),
            lockstep: self.lockstep.saturating_sub(earlier.lockstep),
            channel: self.channel.saturating_sub(earlier.channel),
        }
    }
}

/// The difference between two snapshots.
#[derive(Clone, Copy, Debug, Default)]
pub struct Interval {
    pub wall: Duration,
    pub cpu: ThreadCpu,
    pub renders: u64,
    pub editor_renders: u64,
    pub console_renders: u64,
    pub render: Duration,
    pub canvas: Duration,
    pub upload: Duration,
    pub frames: u64,
    pub animator: Duration,
    pub events: u64,
    pub marks: u64,
    pub drawing: Duration,
    pub tiles: u64,
    pub instructions: u64,
    pub lockstep: Duration,
    pub channel: Duration,
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

/// A count, shortened: 950, 12.3 k, 4.5 M.
pub fn count(n: u64) -> String {
    match n {
        0..1_000 => n.to_string(),
        1_000..1_000_000 => format!("{:.1} k", n as f64 / 1e3),
        _ => format!("{:.1} M", n as f64 / 1e6),
    }
}

impl Interval {
    /// Each render's average time, which is the average of every frame's
    /// work on the UI thread: `cpu.main` per render, and our share of it.
    pub fn per_render(&self, d: Duration) -> Duration {
        d.checked_div(self.renders.max(1) as u32).unwrap_or_default()
    }

    /// The UI thread's CPU time for each render, the number that says
    /// whether the window costs too much.
    pub fn ui_per_frame(&self) -> Duration {
        self.per_render(self.cpu.main)
    }

    fn per_second(&self, n: u64) -> f64 {
        n as f64 / self.wall.as_secs_f64().max(1e-9)
    }

    /// The one-line summary at the end of a run, such as
    /// `run 3.2 s: 190 renders, UI 4.1 ms a frame (ours 0.3 ms, canvas 0.2 ms) ...`.
    pub fn summary(&self) -> String {
        format!(
            "run {:.1} s: {} renders at {:.0}/s (editor {}, console {}), UI {:.2} ms a frame (ours {:.2} ms, canvas {:.2} ms, animator {:.2} ms); \
             {} events, {} marks drawn in {:.0} ms, {} tiles uploaded in {:.0} ms; \
             {} instructions at {}/s, waiting {:.1} s in lockstep, {:.1} s on the channel; CPU {}",
            self.wall.as_secs_f64(),
            self.renders,
            self.per_second(self.renders),
            self.editor_renders,
            self.console_renders,
            ms(self.ui_per_frame()),
            ms(self.per_render(self.render)),
            ms(self.per_render(self.canvas)),
            ms(self.per_render(self.animator)),
            count(self.events),
            count(self.marks),
            ms(self.drawing),
            count(self.tiles),
            ms(self.upload),
            count(self.instructions),
            count(self.per_second(self.instructions) as u64),
            self.lockstep.as_secs_f64(),
            self.channel.as_secs_f64(),
            self.cpu.percentages(self.wall, "UI"),
        )
    }

    /// The overlay's lines.
    pub fn lines(&self) -> Vec<String> {
        vec![
            format!("{:.0} renders/s, UI CPU {:.2} ms a frame", self.per_second(self.renders), ms(self.ui_per_frame())),
            format!(
                "ours {:.2} ms, canvas {:.2} ms, animator {:.2} ms",
                ms(self.per_render(self.render)),
                ms(self.per_render(self.canvas)),
                ms(self.per_render(self.animator))
            ),
            format!(
                "{} events/s, {} marks/s, {} tiles/s",
                count(self.per_second(self.events) as u64),
                count(self.per_second(self.marks) as u64),
                count(self.per_second(self.tiles) as u64)
            ),
            format!(
                "{} instructions/s, lockstep {:.0}%, channel {:.0}%",
                count(self.per_second(self.instructions) as u64),
                100.0 * self.lockstep.as_secs_f64() / self.wall.as_secs_f64().max(1e-9),
                100.0 * self.channel.as_secs_f64() / self.wall.as_secs_f64().max(1e-9)
            ),
            format!("CPU {}", self.cpu.percentages(self.wall, "UI")),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_are_shortened() {
        assert_eq!(count(950), "950");
        assert_eq!(count(12_345), "12.3 k");
        assert_eq!(count(4_500_000), "4.5 M");
    }

    #[test]
    fn the_summary_divides_by_renders() {
        let interval = Interval {
            wall: Duration::from_secs(2),
            cpu: ThreadCpu { main: Duration::from_millis(500), interpreter: Duration::from_millis(40), other: Duration::ZERO },
            renders: 100,
            render: Duration::from_millis(30),
            instructions: 2_000_000,
            ..Interval::default()
        };
        let summary = interval.summary();
        assert!(summary.starts_with("run 2.0 s: 100 renders at 50/s (editor 0, console 0), UI 5.00 ms a frame (ours 0.30 ms"), "{summary}");
        assert!(summary.contains("2.0 M instructions at 1.0 M/s"), "{summary}");
        assert!(summary.ends_with("CPU UI 25% interpreter 2% other 0%"), "{summary}");
    }
}
