//! The `pendown-cli` binary: its options, its output files, its errors and
//! its exit codes.

use std::{path::PathBuf, process::Command};

fn dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

struct Run {
    status: i32,
    stdout: String,
    stderr: String,
}

/// Runs the CLI with metrics off, so stderr has only what the program says.
fn cli(dir: &PathBuf, args: &[&str]) -> Run {
    cli_with(dir, args, "0")
}

fn cli_with(dir: &PathBuf, args: &[&str], metrics: &str) -> Run {
    let out = Command::new(env!("CARGO_BIN_EXE_pendown-cli"))
        .args(args)
        .current_dir(dir)
        .env("PENDOWN_METRICS", metrics)
        .output()
        .unwrap();
    Run {
        status: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8(out.stdout).unwrap(),
        stderr: String::from_utf8(out.stderr).unwrap(),
    }
}

fn size_of(path: PathBuf) -> (u32, u32) {
    image::open(path).unwrap().to_rgba8().dimensions()
}

#[test]
fn saves_next_to_the_program_by_default() {
    let d = dir("default-output");
    std::fs::write(d.join("square.logo"), "REPEAT 4 [FD 100 RT 90]\nPRINT \"done").unwrap();
    let r = cli(&d, &["square.logo"]);
    assert_eq!((r.status, r.stdout.as_str(), r.stderr.as_str()), (0, "done\n", ""));
    assert_eq!(size_of(d.join("square.png")), (1000, 1000));
}

#[test]
fn output_and_size_options() {
    let d = dir("options");
    std::fs::write(d.join("square.logo"), "REPEAT 4 [FD 100 RT 90]").unwrap();
    let r = cli(&d, &["square.logo", "--output", "out.png", "--size", "250x300"]);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(size_of(d.join("out.png")), (250, 300));
    let r = cli(&d, &["square.logo", "-o", "small.png", "-s", "20x10"]);
    assert_eq!(r.status, 0, "{}", r.stderr);
    assert_eq!(size_of(d.join("small.png")), (20, 10));
}

#[test]
fn an_svg_name_saves_an_svg_of_the_turtle_area() {
    let d = dir("svg");
    std::fs::write(d.join("square.logo"), "REPEAT 4 [FD 100 RT 90]").unwrap();
    let r = cli(&d, &["square.logo", "-o", "square.svg", "-s", "250x300"]);
    assert_eq!(r.status, 0, "{}", r.stderr);
    let svg = std::fs::read_to_string(d.join("square.svg")).unwrap();
    assert!(svg.starts_with("<svg "), "{svg}");
    assert!(svg.contains(r#"viewBox="-500 -500 1000 1000""#), "{svg}");
    assert_eq!(svg.matches("<line ").count(), 4, "{svg}");
    assert!(!d.join("square.png").exists());
}

#[test]
fn a_bad_size_is_a_usage_error() {
    let d = dir("bad-size");
    std::fs::write(d.join("p.logo"), "FD 10").unwrap();
    let r = cli(&d, &["p.logo", "--size", "250"]);
    assert_eq!(r.status, 2);
    assert!(r.stderr.contains("expected a size like 250x300, not \"250\""), "{}", r.stderr);
}

#[test]
fn parse_errors_are_reported_and_nothing_is_saved() {
    let d = dir("parse-error");
    std::fs::write(d.join("p.logo"), "FD 10\nRT\nFOO 3").unwrap();
    let r = cli(&d, &["p.logo"]);
    assert_eq!(r.status, 1);
    assert_eq!(r.stderr, "p.logo:2:1: Not enough inputs to RT\np.logo:3:1: I don't know how to FOO\n");
    assert!(!d.join("p.png").exists());
}

#[test]
fn runtime_errors_save_the_picture_so_far() {
    let d = dir("runtime-error");
    let program = "TO BOX :n\n  FD :n RT 90\n  FD \"far\nEND\nFD 50\nBOX 100";
    std::fs::write(d.join("p.logo"), program).unwrap();
    let r = cli(&d, &["p.logo"]);
    assert_eq!(r.status, 1);
    assert_eq!(
        r.stderr,
        "p.logo:3:6: FD doesn't like far as input\n  in BOX (line 6)\nSaved the drawing as far as it got to p.png\n"
    );
    assert!(d.join("p.png").exists());
}

#[test]
fn endless_programs_stop_at_the_limit() {
    let d = dir("limit");
    std::fs::write(d.join("forever.logo"), "FOREVER [FD 1 RT 1]").unwrap();
    let r = cli(&d, &["forever.logo", "--limit", "5000"]);
    assert_eq!(r.status, 1);
    assert!(r.stderr.starts_with("forever.logo:1:"), "{}", r.stderr);
    assert!(r.stderr.contains(": Stopped after 5000 instructions: does this program ever end?\n"), "{}", r.stderr);
    assert!(d.join("forever.png").exists());
}

#[test]
fn a_missing_program_is_an_error() {
    let d = dir("missing");
    let r = cli(&d, &["nothing.logo"]);
    assert_eq!(r.status, 1);
    assert!(r.stderr.starts_with("nothing.logo: can't read it:"), "{}", r.stderr);
}

#[test]
fn a_seed_makes_random_repeatable() {
    let d = dir("seed");
    std::fs::write(d.join("r.logo"), "REPEAT 5 [PRINT RANDOM 1000]").unwrap();
    let a = cli(&d, &["r.logo", "--seed", "7"]);
    let b = cli(&d, &["r.logo", "--seed", "7"]);
    assert_eq!(a.status, 0);
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn metrics_print_a_summary_on_stderr() {
    let d = dir("metrics");
    std::fs::write(d.join("square.logo"), "REPEAT 4 [FD 100 RT 90]").unwrap();
    let r = cli_with(&d, &["square.logo"], "1");
    // REPEAT, its 4 passes, and 4 each of FD and RT.
    assert_eq!((r.status, r.stdout.as_str()), (0, ""));
    assert!(r.stderr.starts_with("run ") && r.stderr.contains(": 13 instructions at "), "{}", r.stderr);
    assert!(r.stderr.contains("4 marks drawn in") && r.stderr.contains("; CPU main "), "{}", r.stderr);
    assert_eq!(cli_with(&d, &["square.logo"], "0").stderr, "");
}
