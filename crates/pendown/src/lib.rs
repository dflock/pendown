//! The parts of the Pendown app that don't need a window: the animator, the
//! interpreter thread, speed levels and error messages. `main.rs` and the
//! modules it declares are the window.

pub mod animator;
pub mod breakpoints;
pub mod colours;
pub mod console;
pub mod document;
pub mod drive;
pub mod errors;
pub mod gallery;
pub mod help;
pub mod markup;
pub mod metrics;
pub mod run;
pub mod source;
pub mod speed;
pub mod teach;
pub mod tutorial;
