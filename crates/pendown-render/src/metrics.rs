//! What the drawing measures (ADR 0020).

use pendown_logo::metrics::{Counter, Timer};

#[derive(Debug, Default)]
pub struct DrawingMetrics {
    /// Lines, arcs and shapes drawn into the buffer, not counting re-renders.
    pub marks: Counter,
    /// Drawing new marks into the buffer.
    pub drawing: Timer,
    /// Drawing everything again after a resize or a new background.
    pub rerenders: Timer,
    /// Tiles handed out for upload because they changed.
    pub tiles: Counter,
    /// Encoding PNGs, for the CLI and Export.
    pub png: Timer,
}
