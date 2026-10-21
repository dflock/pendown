//! Syntax colours for the editor, from `pendown-logo`'s own tokens, so the
//! editor and the interpreter agree on what's a word, a number, a variable
//! or a comment (ADR 0008).

use std::ops::Range;

use gpui_kit::component::input::{EditorState, FoldRange, HighlightStyleResolver, InputEdit, InputHighlighter, Rope};
use gpui_kit::*;
use pendown_logo::syntax::{Kind, highlight};

/// The theme's name for each kind of token.
fn style_name(kind: Kind) -> Option<&'static str> {
    Some(match kind {
        Kind::Comment => "comment",
        Kind::Bracket => "punctuation.bracket",
        Kind::Number => "number",
        Kind::Quoted => "string",
        Kind::Variable => "variable",
        Kind::Operator => "operator",
        // Built-in commands look like keywords, as most Logo editors show
        // them, and the user's own procedures like functions.
        Kind::Keyword | Kind::Command => "keyword",
        Kind::Procedure => "function",
        // Unknown words get a diagnostic instead.
        Kind::Unknown => return None,
    })
}

/// The editor's highlighter for the `"logo"` language. It colours the whole
/// buffer on every edit, which takes under a millisecond for 500 lines.
#[derive(Default)]
pub struct LogoHighlighter {
    runs: Vec<(Range<usize>, &'static str)>,
}

impl InputHighlighter for LogoHighlighter {
    fn language(&self) -> SharedString {
        "logo".into()
    }

    fn update(&mut self, _: Option<InputEdit>, text: &Rope, _: bool, _: &mut Window, _: &mut Context<EditorState>) {
        self.runs = highlight(&text.to_string()).into_iter().filter_map(|(r, k)| Some((r, style_name(k)?))).collect();
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
