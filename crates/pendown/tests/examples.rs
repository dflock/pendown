//! Examples are tests (phase 5 issue 04): every example in the help runs as
//! its page says, and draws the picture the panel shows, which must match
//! the saved one in `tests/help-pictures/`. What an example prints is
//! saved beside it, as `.txt`.
//!
//! After a change that's meant to alter them, look at the pictures, then
//! update them with `UPDATE_PICTURES=1 cargo test -p pendown --test examples`.

use std::path::PathBuf;

use pendown::help::{PICTURE_SIZE, picture_options};
use pendown_logo::reference::reference;

/// As in `pendown-cli`'s picture tests: a pixel counts as different if a
/// channel is off by more than this, and the picture fails if more than
/// this share of pixels do.
const TOLERANCE: u8 = 40;
const MAX_DIFFERENT: f64 = 0.002;

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/help-pictures")
}

/// Checks one program, named `name`, and returns what went wrong, if
/// anything.
fn check(name: &str, code: &str, fails: bool) -> Option<String> {
    let output = pendown_cli::render(code, picture_options());
    match (&output.result, fails) {
        (Ok(()), true) => return Some(format!("{name}: should stop with an error, but didn't")),
        (Err(e), false) => return Some(format!("{name}: {e:?}")),
        _ => {}
    }
    let png = dir().join(format!("{name}.png"));
    let txt = dir().join(format!("{name}.txt"));
    if std::env::var_os("UPDATE_PICTURES").is_some() {
        std::fs::write(&png, &output.png).unwrap();
        if output.printed.is_empty() {
            let _ = std::fs::remove_file(&txt);
        } else {
            std::fs::write(&txt, &output.printed).unwrap();
        }
        return None;
    }
    let printed = std::fs::read_to_string(&txt).unwrap_or_default();
    if output.printed != printed {
        return Some(format!("{name}: printed {:?}, not {printed:?}", output.printed));
    }
    let expected = match image::open(&png) {
        Ok(image) => image.to_rgba8(),
        Err(e) => return Some(format!("{}: {e}. Run with UPDATE_PICTURES=1 to make it", png.display())),
    };
    let actual = image::load_from_memory(&output.png).unwrap().to_rgba8();
    let different = actual
        .pixels()
        .zip(expected.pixels())
        .filter(|(a, e)| a.0.iter().zip(e.0).any(|(a, e)| a.abs_diff(e) > TOLERANCE))
        .count();
    let share = different as f64 / (PICTURE_SIZE * PICTURE_SIZE) as f64;
    if actual.dimensions() != expected.dimensions() || share > MAX_DIFFERENT {
        let failed = png.with_extension("actual.png");
        std::fs::write(&failed, &output.png).unwrap();
        return Some(format!("{name}: {different} pixels differ; the new picture is in {}", failed.display()));
    }
    None
}

#[test]
fn every_example_runs_and_draws_its_picture() {
    std::fs::create_dir_all(dir()).unwrap();
    let mut problems = Vec::new();
    let mut names = Vec::new();
    for command in reference().commands.iter().filter(|c| c.implemented) {
        for (i, example) in command.examples.iter().enumerate() {
            let name = pendown::help::example_name(&command.key, i);
            problems.extend(check(&name, &example.code, example.fails));
            names.push(name);
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
    // No pictures are left from examples that have gone.
    let stale: Vec<_> = std::fs::read_dir(dir())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            let stem = p.file_name().unwrap().to_string_lossy();
            let stem = stem.trim_end_matches(".actual.png").trim_end_matches(".png").trim_end_matches(".txt");
            !names.iter().any(|n| n == stem)
        })
        .collect();
    assert!(stale.is_empty(), "pictures with no example: {stale:#?}");
}
