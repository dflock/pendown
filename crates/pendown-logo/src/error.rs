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
    /// The command the error is about, as it was called, such as `FD`, for
    /// help on it.
    pub command: Option<String>,
}

/// "1 input", "2 inputs" or "no inputs".
fn inputs(n: usize) -> String {
    match n {
        0 => "no inputs".to_string(),
        1 => "1 input".to_string(),
        n => format!("{n} inputs"),
    }
}

/// What to say when a call has too few inputs: `name` as it should read,
/// and whether it can only take `needs`, or could take more.
pub fn not_enough_inputs(name: &str, needs: usize, exactly: bool, got: usize) -> String {
    let at_least = if exactly { "" } else { "at least " };
    let got = match got {
        0 => "none".to_string(),
        n => n.to_string(),
    };
    format!("{name} needs {at_least}{}, but got {got}", inputs(needs))
}

/// What to say when a call has too many inputs: `name` as it should read,
/// and whether it can only take `max`, or could take fewer.
pub fn too_many_inputs(name: &str, max: usize, exactly: bool, got: usize) -> String {
    let at_most = if exactly { "" } else { "at most " };
    format!("{name} takes {at_most}{}, but got {got}", inputs(max))
}

/// One step of an error's call chain: `procedure` was called at `span`.
#[derive(Clone, Debug, PartialEq)]
pub struct CallSite {
    pub procedure: String,
    pub span: Span,
}

impl Error {
    pub fn new(message: impl Into<String>, span: Option<Span>) -> Error {
        Error {
            message: message.into(),
            span,
            calls: Vec::new(),
            command: None,
        }
    }

    /// The error is about this command.
    pub fn about(mut self, command: &str) -> Error {
        self.command = Some(command.to_uppercase());
        self
    }

    /// For a name that isn't a procedure: why, if it's an obsolete command,
    /// and otherwise "I don't know how to".
    pub fn unknown(name: &str, span: Option<Span>) -> Error {
        let upper = name.to_uppercase();
        match crate::reference::reference().obsolete_reason(name) {
            Some(reason) => {
                Error::new(format!("{upper} isn't in Pendown: {reason}"), span).about(name)
            }
            None => Error::new(format!("I don't know how to {upper}"), span),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.message)?;
        for (i, call) in self.calls.iter().enumerate() {
            let how = if i == 0 { "in" } else { "called from" };
            write!(
                f,
                ", {how} {} (line {})",
                call.procedure,
                call.span.line + 1
            )?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_few_inputs_read_well() {
        assert_eq!(
            not_enough_inputs("REPEAT", 2, true, 1),
            "REPEAT needs 2 inputs, but got 1"
        );
        assert_eq!(
            not_enough_inputs("FD", 1, true, 0),
            "FD needs 1 input, but got none"
        );
        assert_eq!(
            not_enough_inputs("IF", 2, false, 1),
            "IF needs at least 2 inputs, but got 1"
        );
    }

    #[test]
    fn too_many_inputs_read_well() {
        assert_eq!(
            too_many_inputs("FD", 1, true, 2),
            "FD takes 1 input, but got 2"
        );
        assert_eq!(
            too_many_inputs("CS", 0, true, 1),
            "CS takes no inputs, but got 1"
        );
        assert_eq!(
            too_many_inputs("IF", 3, false, 4),
            "IF takes at most 3 inputs, but got 4"
        );
    }
}
