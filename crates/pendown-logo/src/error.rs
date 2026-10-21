use std::fmt;

use crate::span::Span;

/// A parse or runtime error, pointing at the code that caused it (ADR 0005).
#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub message: String,
    pub span: Option<Span>,
    /// The procedure calls the error happened inside, innermost first. Tail
    /// calls replace their caller, so they leave no entry.
    pub calls: Vec<CallSite>,
}

/// One step of an error's call chain: `procedure` was called at `span`.
#[derive(Clone, Debug, PartialEq)]
pub struct CallSite {
    pub procedure: String,
    pub span: Span,
}

impl Error {
    pub fn new(message: impl Into<String>, span: Option<Span>) -> Error {
        Error { message: message.into(), span, calls: Vec::new() }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.message)?;
        for (i, call) in self.calls.iter().enumerate() {
            let how = if i == 0 { "in" } else { "called from" };
            write!(f, ", {how} {} (line {})", call.procedure, call.span.line + 1)?;
        }
        Ok(())
    }
}

/// Why a run ended early.
#[derive(Clone, Debug, PartialEq)]
pub enum RunError {
    /// The source didn't parse, so nothing ran.
    Parse(Vec<Error>),
    Runtime(Error),
    /// Stopped from outside, by the cancel flag or because the event
    /// receiver went away.
    Halted,
}
