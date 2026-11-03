//! Where warnings, errors and panics go. A release build on Windows has no
//! console, so there they're written to a file, started afresh each run.
//! Everywhere else they go to stderr.

use std::{
    env,
    ffi::OsString,
    fs::{self, File},
    io::{self, Write},
    panic,
    path::PathBuf,
    sync::Mutex,
};

use log::{Level, LevelFilter, Log, Metadata, Record};

/// The log file on Windows, given `%LOCALAPPDATA%`.
fn windows_path(local_app_data: Option<OsString>) -> Option<PathBuf> {
    Some(
        PathBuf::from(local_app_data?)
            .join("Pendown")
            .join("pendown.log"),
    )
}

/// One line of the log.
fn line(level: Level, target: &str, message: impl std::fmt::Display) -> String {
    format!("{level} {target}: {message}\n")
}

struct Logger(Mutex<Box<dyn Write + Send>>);

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Warn
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata())
            && let Ok(mut out) = self.0.lock()
        {
            let text = line(record.level(), record.target(), record.args());
            // There's nowhere left to say that the log can't be written.
            let _ = out.write_all(text.as_bytes());
            let _ = out.flush();
        }
    }

    fn flush(&self) {}
}

/// The file on Windows, if it can be made, and otherwise stderr.
fn output() -> Box<dyn Write + Send> {
    let file = cfg!(windows)
        .then(|| windows_path(env::var_os("LOCALAPPDATA")))
        .flatten()
        .and_then(|path| {
            fs::create_dir_all(path.parent()?).ok()?;
            File::create(path).ok()
        });
    match file {
        Some(file) => Box::new(file),
        None => Box::new(io::stderr()),
    }
}

/// Starts logging, and logs panics too. Call it first.
pub fn init() {
    if log::set_boxed_logger(Box::new(Logger(Mutex::new(output())))).is_ok() {
        log::set_max_level(LevelFilter::Warn);
    }
    let usual = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        log::error!("{info}");
        usual(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_file_is_in_local_app_data() {
        assert_eq!(
            windows_path(Some("base".into())),
            Some(PathBuf::from("base").join("Pendown").join("pendown.log"))
        );
        assert_eq!(windows_path(None), None);
    }

    #[test]
    fn a_line_says_what_and_where() {
        assert_eq!(
            line(Level::Error, "pendown::help_panel", "it failed"),
            "ERROR pendown::help_panel: it failed\n"
        );
    }
}
