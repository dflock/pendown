//! Examples are tests (phase 5 issue 04): every example in the help, and
//! every program in the tutorial, runs as it should, and draws the picture
//! the panel shows, which must match the saved one in
//! `tests/help-pictures/`. What a program prints is saved beside it, as
//! `.txt`.
//!
//! After a change that's meant to alter them, look at the pictures, then
//! update them with `UPDATE_PICTURES=1 cargo test -p pendown --test examples`.
//!
//! The same programs check SVG export: rendered back into pixels by resvg,
//! each SVG must look like the PNG.

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
    // And the tutorial's programs, as `tutorial-1-2` for lesson 1's second.
    for (l, lesson) in pendown::tutorial::lessons().iter().enumerate() {
        for (p, program) in lesson.programs().enumerate() {
            let name = format!("tutorial-{}-{}", l + 1, p + 1);
            problems.extend(check(&name, program, false));
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

/// Every example and lesson program, named as their pictures are.
fn programs() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for command in reference().commands.iter().filter(|c| c.implemented) {
        for (i, example) in command.examples.iter().enumerate() {
            out.push((pendown::help::example_name(&command.key, i), example.code.clone()));
        }
    }
    for (l, lesson) in pendown::tutorial::lessons().iter().enumerate() {
        for (p, program) in lesson.programs().enumerate() {
            out.push((format!("tutorial-{}-{}", l + 1, p + 1), program.to_string()));
        }
    }
    out
}

/// `svg` with every number after `attribute` made at least `min`.
fn at_least(svg: &str, attribute: &str, min: f64) -> String {
    let mut out = String::new();
    let mut rest = svg;
    while let Some(i) = rest.find(attribute) {
        let (before, after) = rest.split_at(i + attribute.len());
        let end = after.find('"').expect("a closing quote");
        let value: f64 = after[..end].parse().expect("a number");
        out.push_str(before);
        out.push_str(&value.max(min).to_string());
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

#[test]
fn every_svg_looks_like_its_png() {
    const SIZE: u32 = 500;
    let options = pendown_cli::Options { width: SIZE, height: SIZE, limit: 1_000_000, seed: Some(1), fit: false, svg: true };
    // And some that the examples don't cover: shapes at headings where
    // turning the wrong way would show, arcs both ways and past a whole
    // turn, rounded corners, fills, and translucent pens.
    let extra = [
        ("turned-shapes", "SETPENSIZE 4 SETPC \"orange RT 30 ELLIPSE 80 200\nPU FD 250 PD SETFILLCOLOR [0 128 255 160] RT 20 (RECTANGLE 120 260 30)"),
        ("arcs", "SETPENSIZE 3 ARC 270 150\nPU HOME PD LT 70 ARC -200 120\nPU SETPOS [-250 -250] PD ARC 500 80"),
        ("translucent", "SETPC [255 0 0 120] SETPENSIZE 20 REPEAT 5 [FD 300 RT 144]"),
    ];
    let mut problems = Vec::new();
    for (name, code) in programs().into_iter().chain(extra.map(|(n, c)| (n.to_string(), c.to_string()))) {
        let output = pendown_cli::render(&code, options);
        let svg = output.svg.expect("asked for");
        // The PNG never draws a line thinner than a pixel, so for the same
        // picture, neither does this.
        let min_width = pendown_render::TURTLE_AREA / SIZE as f64;
        let widened = at_least(&svg, "stroke-width=\"", min_width);
        let tree = match resvg::usvg::Tree::from_str(&widened, &resvg::usvg::Options::default()) {
            Ok(tree) => tree,
            Err(e) => {
                problems.push(format!("{name}: the SVG doesn't parse: {e}"));
                continue;
            }
        };
        let mut pixmap = resvg::tiny_skia::Pixmap::new(SIZE, SIZE).unwrap();
        // The SVG is the turtle area, a step to a pixel.
        let scale = SIZE as f32 / pendown_render::TURTLE_AREA as f32;
        resvg::render(&tree, resvg::tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
        // Compared at a quarter of the size: the two renderers smooth edges
        // differently, which averages out, while anything drawn in the
        // wrong place still shows. The background is opaque, so resvg's
        // premultiplied pixels are straight.
        let small = |image: image::RgbaImage| image::imageops::thumbnail(&image, SIZE / 4, SIZE / 4);
        let png = small(image::load_from_memory(&output.png).unwrap().to_rgba8());
        let resvg = small(image::RgbaImage::from_raw(SIZE, SIZE, pixmap.data().to_vec()).unwrap());
        let different = png
            .pixels()
            .zip(resvg.pixels())
            .filter(|(a, e)| a.0.iter().zip(e.0).any(|(a, e)| a.abs_diff(e) > TOLERANCE))
            .count();
        if different as f64 / (png.width() * png.height()) as f64 > MAX_DIFFERENT {
            let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("svg-differences");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(format!("{name}.svg")), &svg).unwrap();
            std::fs::write(dir.join(format!("{name}.png")), &output.png).unwrap();
            pixmap.save_png(dir.join(format!("{name}.resvg.png"))).unwrap();
            problems.push(format!("{name}: {different} pixels differ; see {}", dir.display()));
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
}
