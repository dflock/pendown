//! `pendown-cli`: runs a Logo program and saves the drawing as a PNG, or an
//! SVG.

use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use pendown_cli::{Options, describe, parse_size, render};
use pendown_logo::RunError;

/// Runs a Pendown Logo program and saves its drawing as a PNG, or an SVG.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The program to run, a `.logo` file.
    program: PathBuf,

    /// Where to save the drawing. Defaults to the program's name with `.png`.
    /// A name ending `.svg` saves an SVG of the whole turtle area instead,
    /// and ignores `--size`.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// The picture's size in pixels, as WIDTHxHEIGHT. The turtle area is
    /// fitted to it without stretching.
    #[arg(short, long, default_value = "1000x1000", value_parser = parse_size)]
    size: (u32, u32),

    /// Stop with an error after this many instructions, so a program that
    /// never ends, such as `FOREVER`, still finishes.
    #[arg(long, default_value_t = 10_000_000)]
    limit: u64,

    /// The seed for `RANDOM`, so a program that uses it draws the same
    /// picture every time. Without it, each run is different.
    #[arg(long)]
    seed: Option<u64>,

    /// Zoom in on the drawing, rather than showing the whole turtle area.
    #[arg(long)]
    fit: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let path = args.program.display().to_string();
    let source = match std::fs::read_to_string(&args.program) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("{path}: can't read it: {e}");
            return ExitCode::FAILURE;
        }
    };
    let output_path = args
        .output
        .unwrap_or_else(|| args.program.with_extension("png"));
    let (width, height) = args.size;
    let svg = output_path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("svg"))
        .then_some(0.0);
    let output = render(
        &source,
        Options {
            width,
            height,
            limit: args.limit,
            seed: args.seed,
            fit: args.fit,
            svg,
        },
    );
    print!("{}", output.printed);
    if let Some(summary) = &output.summary {
        eprintln!("{summary}");
    }

    let picture = output
        .svg
        .as_ref()
        .map_or(&output.png[..], |svg| svg.as_bytes());
    let save = || match std::fs::write(&output_path, picture) {
        Ok(()) => true,
        Err(e) => {
            eprintln!("{}: can't save the drawing: {e}", output_path.display());
            false
        }
    };
    match &output.result {
        Ok(()) => {
            if save() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(RunError::Parse(errors)) => {
            for e in errors {
                eprintln!("{}", describe(&path, &source, e));
            }
            ExitCode::FAILURE
        }
        Err(RunError::Runtime(e)) => {
            eprintln!("{}", describe(&path, &source, e));
            if save() {
                eprintln!(
                    "Saved the drawing as far as it got to {}",
                    output_path.display()
                );
            }
            ExitCode::FAILURE
        }
        Err(RunError::Halted) => unreachable!("nothing stops the CLI from outside"),
    }
}
