//! How errors read in the console (ADR 0005).

use pendown_logo::Error;

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
                line = format!("called from {} (line {}), {} times", call.procedure, call.span.line + 1, thousands(same - 1));
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

    fn call(procedure: &str, line: u32) -> CallSite {
        CallSite { procedure: procedure.into(), span: Span { source: SourceId::Editor, start: 0, end: 1, line } }
    }

    fn error(calls: Vec<CallSite>) -> Error {
        Error { message: "oops".into(), span: None, calls }
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
        assert_eq!(chain(&e), ["in DOWN (line 2)", "called from DOWN (line 2), 9,999 times", "called from DOWN (line 5)"]);
    }

    #[test]
    fn repeats_further_up_are_counted() {
        let e = error(vec![call("FD", 0), call("B", 3), call("B", 3), call("A", 9)]);
        assert_eq!(chain(&e), ["in FD (line 1)", "called from B (line 4), 2 times", "called from A (line 10)"]);
    }

    #[test]
    fn thousands_separators() {
        assert_eq!(thousands(7), "7");
        assert_eq!(thousands(1000), "1,000");
        assert_eq!(thousands(1234567), "1,234,567");
    }
}
