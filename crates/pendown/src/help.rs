//! The help panel's parts that don't need a window (phase 5): how examples
//! are drawn for their pictures.

use pendown_cli::Options;

/// The pictures of examples are this many pixels square, fitted to the
/// drawing. The panel shows them smaller, so they're sharp on a HiDPI
/// screen.
pub const PICTURE_SIZE: u32 = 200;

/// How an example is drawn for its picture: fitted, with `RANDOM`'s seed
/// fixed so the picture is always the same, as in the tests.
pub fn picture_options() -> Options {
    Options { width: PICTURE_SIZE, height: PICTURE_SIZE, limit: 1_000_000, seed: Some(1), fit: true }
}

/// An example's name, for its saved picture: `forward-1` for FORWARD's
/// first.
pub fn example_name(key: &str, index: usize) -> String {
    format!("{key}-{}", index + 1)
}
