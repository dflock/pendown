//! How errors read in the console (ADR 0005).

use pendown_logo::{Error, explain::Explanation};

/// What the console prints under an error's message: the lines that say how
/// the command is written.
pub fn console_lines(explanation: &Explanation) -> Vec<String> {
    let usage = explanation
        .usage
        .iter()
        .map(|u| format!("Written like this: {u}"));
    let example = explanation.example.iter().map(|e| format!("Example: {e}"));
    let defined = explanation
        .defined_on
        .iter()
        .map(|line| format!("Defined on line {line}"));
    usage.chain(example).chain(defined).collect()
}

/// An explanation for the editor's hover popup, which renders Markdown. The
/// first line is the message.
pub fn markdown(explanation: &Explanation) -> String {
    let mut out = escape_markdown(&explanation.message);
    let mut code = |label: &str, code: &Option<String>| {
        if let Some(code) = code {
            out.push_str(&format!("\n\n**{label}**\n\n```\n{code}\n```"));
        }
    };
    code("Written like this", &explanation.usage);
    code("Example", &explanation.example);
    if let Some(line) = explanation.defined_on {
        out.push_str(&format!("\n\nDefined on line {line}"));
    }
    out
}

/// A message's own punctuation, such as the brackets of "This ] has no
/// matching [" or a `*` it quotes, isn't Markdown.
fn escape_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(
            c,
            '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '#' | '&' | '~'
        ) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// An error's call chain, one line per step, innermost first. A deep
/// recursion has thousands of identical steps, so runs of the same call are
/// collapsed into one line: "called from DOWN (line 2), 9,998 times".
pub fn chain(error: &Error) -> Vec<String> {
    let mut lines = Vec::new();
    let mut i = 0;
    while i < error.calls.len() {
        let call = &error.calls[i];
        let same = error.calls[i..]
            .iter()
            .take_while(|c| c.procedure == call.procedure && c.span.line == call.span.line)
            .count();
        let how = if i == 0 { "in" } else { "called from" };
        let mut line = format!("{how} {} (line {})", call.procedure, call.span.line + 1);
        if same > 1 {
            if i == 0 {
                // "in DOWN", then the other calls from the same place.
                lines.push(line);
                line = format!(
                    "called from {} (line {}), {} times",
                    call.procedure,
                    call.span.line + 1,
                    thousands(same - 1)
                );
            } else {
                line.push_str(&format!(", {} times", thousands(same)));
            }
        }
        lines.push(line);
        i += same;
    }
    lines
}

/// 9998 as "9,998".
fn thousands(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use pendown_logo::{CallSite, SourceId, Span};

    use super::*;

    fn explanation() -> Explanation {
        Explanation {
            message: "REPEAT needs 2 inputs, but got 1".into(),
            usage: Some("REPEAT number [instructions]".into()),
            example: Some("REPEAT 4 [FD 100 RT 90]".into()),
            ..Explanation::default()
        }
    }

    #[test]
    fn the_console_says_how_a_command_is_written() {
        assert_eq!(
            console_lines(&explanation()),
            [
                "Written like this: REPEAT number [instructions]",
                "Example: REPEAT 4 [FD 100 RT 90]",
            ]
        );
    }

    #[test]
    fn the_console_says_where_a_procedure_is() {
        let explanation = Explanation {
            message: "SQ needs 1 input, but got none".into(),
            usage: Some("SQ :n".into()),
            defined_on: Some(3),
            ..Explanation::default()
        };
        assert_eq!(
            console_lines(&explanation),
            ["Written like this: SQ :n", "Defined on line 3"]
        );
    }

    #[test]
    fn the_console_adds_nothing_to_a_bare_message() {
        assert!(console_lines(&Explanation::default()).is_empty());
    }

    #[test]
    fn the_hover_puts_code_in_code_blocks() {
        assert_eq!(
            markdown(&explanation()),
            "REPEAT needs 2 inputs, but got 1\n\n\
             **Written like this**\n\n```\nREPEAT number [instructions]\n```\n\n\
             **Example**\n\n```\nREPEAT 4 [FD 100 RT 90]\n```"
        );
    }

    #[test]
    fn a_message_is_not_read_as_markdown() {
        let explanation = Explanation {
            message: "This ] has no matching [".into(),
            ..Explanation::default()
        };
        assert_eq!(markdown(&explanation), "This \\] has no matching \\[");
    }

    fn call(procedure: &str, line: u32) -> CallSite {
        CallSite {
            procedure: procedure.into(),
            span: Span {
                source: SourceId::Editor,
                start: 0,
                end: 1,
                line,
            },
        }
    }

    fn error(calls: Vec<CallSite>) -> Error {
        Error {
            message: "oops".into(),
            span: None,
            calls,
            command: None,
        }
    }

    #[test]
    fn a_short_chain_is_listed_in_full() {
        let e = error(vec![call("B", 1), call("A", 8)]);
        assert_eq!(chain(&e), ["in B (line 2)", "called from A (line 9)"]);
    }

    #[test]
    fn repeated_calls_are_collapsed() {
        let mut calls = vec![call("DOWN", 1); 10_000];
        calls.push(call("DOWN", 4));
        let e = error(calls);
        assert_eq!(
            chain(&e),
            [
                "in DOWN (line 2)",
                "called from DOWN (line 2), 9,999 times",
                "called from DOWN (line 5)"
            ]
        );
    }

    #[test]
    fn repeats_further_up_are_counted() {
        let e = error(vec![
            call("FD", 0),
            call("B", 3),
            call("B", 3),
            call("A", 9),
        ]);
        assert_eq!(
            chain(&e),
            [
                "in FD (line 1)",
                "called from B (line 4), 2 times",
                "called from A (line 10)"
            ]
        );
    }

    #[test]
    fn thousands_separators() {
        assert_eq!(thousands(7), "7");
        assert_eq!(thousands(1000), "1,000");
        assert_eq!(thousands(1234567), "1,234,567");
    }
}
