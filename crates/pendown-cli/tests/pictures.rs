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
    let output = render(program, Options { width, height, limit: 10_000_000 });
    assert_eq!(output.result, Ok(()), "{name} should run without errors");
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/pictures").join(format!("{name}.png"));
    if std::env::var_os("UPDATE_PICTURES").is_some() {
        std::fs::write(&path, &output.png).unwrap();
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|e| panic!("{}: {e}. Run with UPDATE_PICTURES=1 to create it", path.display()))
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
        panic!("{name}: {different} pixels differ ({:.2}%); the new picture is in {}", share * 100.0, failed.display());
    }
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
