//! What the animator and the window measure (ADR 0020). The interpreter's
//! metrics are in `pendown_logo::metrics`, the drawing's in
//! `pendown_render::metrics`.

use pendown_logo::metrics::{Counter, Timer};

#[derive(Debug, Default)]
pub struct AnimatorMetrics {
    /// Calls to `frame`, each timed.
    pub frames: Timer,
    /// Events taken from the channel.
    pub events: Counter,
}
