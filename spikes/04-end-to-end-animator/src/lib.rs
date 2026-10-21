//! Spike 04: the interpreter, a channel and the animator, end to end. See
//! `.scratch/phase-1-spikes/issues/04-end-to-end-animator.md`.
//!
//! The animator and the code that starts a run have no gpui code, so
//! `tests/animator.rs` checks them without a window. `main.rs` is the window.

pub mod animator;
pub mod run;
