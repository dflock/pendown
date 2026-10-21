//! Syntax colours for the editor, from spike 01's small lexer. Issue 15
//! replaces it with `pendown-logo`'s own reader.

use std::ops::Range;

use gpui_kit::component::input::{EditorState, FoldRange, HighlightStyleResolver, InputEdit, InputHighlighter, Rope};
use gpui_kit::*;

const COMMANDS: &[&str] = &[
    "fd", "forward", "bk", "back", "rt", "right", "lt", "left", "repeat", "to", "end", "make", "if", "ifelse", "stop",
    "output", "op", "print", "pr", "pu", "pd", "penup", "pendown", "setpc", "setpencolor", "forever", "show", "type",
    "repcount", "thing", "word", "sum", "difference", "product", "quotient", "minus", "heading",
];

fn lex(text: &str) -> Vec<(Range<usize>, &'static str)> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        let c = bytes[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let name = match c {
            b';' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                "comment"
            }
            b'[' | b']' | b'(' | b')' => {
                i += 1;
                "punctuation.bracket"
            }
            b'+' | b'-' | b'*' | b'/' | b'=' | b'<' | b'>' => {
                i += 1;
                "operator"
            }
            _ => {
                while i < bytes.len() && !b" \t\r\n[]();".contains(&bytes[i]) {
                    i += 1;
                }
                let word = &text[start..i];
                if word.starts_with(':') {
                    "variable"
                } else if word.starts_with('"') {
                    "string"
                } else if word.parse::<f64>().is_ok() {
                    "number"
                } else if COMMANDS.contains(&word.to_lowercase().as_str()) {
                    "keyword"
                } else {
                    "function"
                }
            }
        };
        out.push((start..i, name));
    }
    out
}

/// The editor's highlighter for the `"logo"` language (ADR 0008). It
/// re-lexes the whole buffer on every edit, which spike 01 found fast enough.
#[derive(Default)]
pub struct LogoHighlighter {
    runs: Vec<(Range<usize>, &'static str)>,
}

impl InputHighlighter for LogoHighlighter {
    fn language(&self) -> SharedString {
        "logo".into()
    }

    fn update(&mut self, _: Option<InputEdit>, text: &Rope, _: bool, _: &mut Window, _: &mut Context<EditorState>) {
        self.runs = lex(&text.to_string());
    }

    fn styles(&self, range: &Range<usize>, resolver: &dyn HighlightStyleResolver) -> Vec<(Range<usize>, HighlightStyle)> {
        let mut out = Vec::new();
        let mut at = range.start;
        let first = self.runs.partition_point(|(r, _)| r.end <= range.start);
        for (r, name) in &self.runs[first..] {
            if r.start >= range.end {
                break;
            }
            let (start, end) = (r.start.max(range.start), r.end.min(range.end));
            if start > at {
                out.push((at..start, HighlightStyle::default()));
            }
            out.push((start..end, resolver.style(name).unwrap_or_default()));
            at = end;
        }
        if at < range.end {
            out.push((at..range.end, HighlightStyle::default()));
        }
        out
    }

    fn fold_ranges(&self, _: &Rope) -> Vec<FoldRange> {
        Vec::new()
    }
}
