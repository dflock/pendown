//! `pendown-cli`: runs a Logo program and saves the drawing as a PNG.

use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use pendown_cli::{Options, describe, parse_size, render};
use pendown_logo::RunError;

/// Runs a Pendown Logo program and saves its drawing as a PNG.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The program to run, a `.logo` file.
    program: PathBuf,

    /// Where to save the drawing. Defaults to the program's name with `.png`.
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
    let output_path = args.output.unwrap_or_else(|| args.program.with_extension("png"));
    let (width, height) = args.size;
    let output = render(&source, Options { width, height, limit: args.limit });
    print!("{}", output.printed);

    let save = || match std::fs::write(&output_path, &output.png) {
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
                eprintln!("Saved the drawing as far as it got to {}", output_path.display());
            }
            ExitCode::FAILURE
        }
        Err(RunError::Halted) => unreachable!("nothing stops the CLI from outside"),
    }
}
