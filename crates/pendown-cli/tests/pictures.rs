//! Picture tests: small programs are drawn and compared with the saved PNGs
//! in `tests/pictures/`. After a change that's meant to alter the pictures,
//! look at them, then update them with
//! `UPDATE_PICTURES=1 cargo test -p pendown-cli --test pictures`.

use std::path::PathBuf;

use pendown_cli::{Options, render};

/// Anti-aliasing can differ slightly between machines and versions, so a
/// pixel only counts as different if a channel is off by more than this...
const TOLERANCE: u8 = 40;
/// ...and the picture only fails if more than this share of pixels differ.
const MAX_DIFFERENT: f64 = 0.002;

fn check(name: &str, program: &str, width: u32, height: u32) {
    check_output(name, program, width, height);
}

/// Checks the picture, and returns what the program printed.
fn check_output(name: &str, program: &str, width: u32, height: u32) -> String {
    let output = render(
        program,
        Options {
            width,
            height,
            limit: 10_000_000,
            seed: Some(1),
            fit: false,
            svg: None,
            crop: false,
        },
    );
    assert_eq!(output.result, Ok(()), "{name} should run without errors");
    let printed = output.printed.clone();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/pictures")
        .join(format!("{name}.png"));
    if std::env::var_os("UPDATE_PICTURES").is_some() {
        std::fs::write(&path, &output.png).unwrap();
        return printed;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|e| {
            panic!(
                "{}: {e}. Run with UPDATE_PICTURES=1 to create it",
                path.display()
            )
        })
        .to_rgba8();
    let actual = image::load_from_memory(&output.png).unwrap().to_rgba8();
    assert_eq!(actual.dimensions(), expected.dimensions(), "{name}: size");
    let different = actual
        .pixels()
        .zip(expected.pixels())
        .filter(|(a, e)| a.0.iter().zip(e.0).any(|(a, e)| a.abs_diff(e) > TOLERANCE))
        .count();
    let share = different as f64 / (width * height) as f64;
    if share > MAX_DIFFERENT {
        let failed = path.with_extension("actual.png");
        std::fs::write(&failed, &output.png).unwrap();
        panic!(
            "{name}: {different} pixels differ ({:.2}%); the new picture is in {}",
            share * 100.0,
            failed.display()
        );
    }
    printed
}

/// The programs in `examples/phase-N/`, which between them use every
/// command of that phase (checked by `pendown-logo`'s `tests/kdl.rs`). Each
/// one's picture and printed output are saved, as `example-NAME.png` and
/// `.txt` for phase 1, and `phase-N-NAME` after that.
fn examples(phase: u32) {
    let dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../examples/phase-{phase}"));
    // Only the programs: the folder may hold pictures of them too.
    let mut programs: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "logo"))
        .collect();
    programs.sort();
    assert!(programs.len() >= 4);
    for path in programs {
        let stem = path.file_stem().unwrap().to_string_lossy();
        let name = if phase == 1 {
            format!("example-{stem}")
        } else {
            format!("phase-{phase}-{stem}")
        };
        let program = std::fs::read_to_string(&path).unwrap();
        let printed = check_output(&name, &program, 400, 400);
        let expected_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/pictures")
            .join(format!("{name}.txt"));
        if std::env::var_os("UPDATE_PICTURES").is_some() {
            std::fs::write(&expected_path, &printed).unwrap();
            continue;
        }
        let expected = std::fs::read_to_string(&expected_path).unwrap();
        assert_eq!(printed, expected, "{name} printed");
    }
}

#[test]
fn phase_1_examples() {
    examples(1);
}

#[test]
fn phase_2_examples() {
    examples(2);
}

#[test]
fn filled_shapes() {
    let program = "
SETFC \"gold CIRCLE 200
SETFC [0 0 255 128] PU SETXY 150 150 SQUARE 250 PD
SETPC \"white SETFC \"#ff000080 RT 30 (RECTANGLE 300 100 30)
SETFC \"none HOME ELLIPSE 100 50
";
    check("filled-shapes", program, 300, 300);
}

#[test]
fn square() {
    check("square", "REPEAT 4 [FD 300 RT 90]", 250, 250);
}

#[test]
fn coloured_spiral() {
    // Cycles through palette colours 1 to 15.
    let program = "
TO SPIRAL :size
  IF :size > 450 [STOP]
  MAKE \"colour :colour + 1
  IF :colour > 15 [MAKE \"colour 1]
  SETPC :colour
  FD :size RT 89
  SPIRAL :size + 6
END
MAKE \"colour 0
SPIRAL 6
";
    check("coloured-spiral", program, 300, 300);
}

#[test]
fn non_square_size() {
    // The turtle area is fitted to the shorter side, without stretching.
    check("wide", "REPEAT 36 [FD 400 BK 400 RT 10]", 400, 200);
}

#[test]
fn tree() {
    let program = "
TO TREE :size :depth
  IF :depth = 0 [STOP]
  FD :size
  LT 30 TREE :size * 0.7 :depth - 1
  RT 60 TREE :size * 0.7 :depth - 1
  LT 30 BK :size
END
PU BK 400 PD
SETPC [100 200 80]
TREE 250 9
";
    check("tree", program, 300, 300);
}

#[test]
fn setxy_home_and_clean() {
    // The first square is cleaned away; only the star and its spokes home
    // remain.
    let program = "
REPEAT 4 [FD 200 RT 90]
CLEAN
PU SETXY -300 -100 PD
REPEAT 5 [FD 300 RT 144]
SETXY 300 300
HOME
SETPOS [-400 400]
";
    check("setxy-home-clean", program, 300, 300);
}

#[test]
fn wrapping_spiral() {
    // Starts near the right edge, so the spiral wraps round on itself.
    check(
        "wrapping-spiral",
        "PU SETXY 350 0 PD\nREPEAT 120 [FD REPCOUNT * 6 RT 61]",
        300,
        300,
    );
}

#[test]
fn colours_and_pen_sizes() {
    // Every colour form, a translucent pen where lines cross, and a
    // background set after drawing, which goes under it.
    let program = r##"
SETPENSIZE 20
SETPC 4 FD 300 BK 300 RT 45
SETPC [0 200 255] FD 300 BK 300 RT 45
SETPC "gold FD 300 BK 300 RT 45
SETPC "#f0f FD 300 BK 300 RT 45
SETPENSIZE 60
SETPC [255 255 255 128] FD 300 BK 300
SETPC "#00ff0080 RT 90 BK 150 FD 300
SETBG "midnightblue
"##;
    check("colours", program, 300, 300);
}

#[test]
fn arcs_and_shapes() {
    let program = r#"
SETPENSIZE 4
SETPC "orange CIRCLE 300
SETPC "skyblue ELLIPSE 100 250
SETPC "lime RT 30 SQUARE 300 LT 30
SETPC "violet (RECTANGLE 500 200 60)
PU SETXY -200 -350 PD SETPC "white
REPEAT 4 [ARC 90 50 ARC -90 50]
"#;
    check("arcs-and-shapes", program, 300, 300);
}

#[test]
fn random_with_a_seed() {
    let program = "REPEAT 200 [PU SETXY (RANDOM -500 500) (RANDOM -500 500) PD SETPC RANDOM 16 CIRCLE RANDOM 60]";
    check("random", program, 300, 300);
}

/// The gallery's showcase programs, in `examples/gallery/`
/// (`.scratch/gallery/` issue 04). Each runs without errors and draws its
/// saved picture, `gallery-NAME.png`.
#[test]
fn gallery_programs() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/gallery");
    let mut programs: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "logo"))
        .collect();
    programs.sort();
    assert!(!programs.is_empty());
    for path in programs {
        let stem = path.file_stem().unwrap().to_string_lossy();
        let program = std::fs::read_to_string(&path).unwrap();
        check(&format!("gallery-{stem}"), &program, 400, 400);
    }
}
