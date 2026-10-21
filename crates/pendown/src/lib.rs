//! The parts of the Pendown app that don't need a window: the animator, the
//! interpreter thread, speed levels and error messages. `main.rs` and the
//! modules it declares are the window.

pub mod animator;
pub mod console;
pub mod document;
pub mod errors;
pub mod run;
pub mod speed;
