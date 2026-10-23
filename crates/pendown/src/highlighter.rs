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

/// Each token in `text` with a colour, and its theme name.
fn runs(text: &str) -> Vec<(Range<usize>, &'static str)> {
    highlight(text).into_iter().filter_map(|(r, k)| Some((r, style_name(k)?))).collect()
}

/// The styles of all of `range`, from `runs`: between tokens, the default.
fn styles(runs: &[(Range<usize>, &str)], range: &Range<usize>, resolver: &dyn HighlightStyleResolver) -> Vec<(Range<usize>, HighlightStyle)> {
    let mut out = Vec::new();
    let mut at = range.start;
    let first = runs.partition_point(|(r, _)| r.end <= range.start);
    for (r, name) in &runs[first..] {
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

/// The styles of a program shown in the help panel: the editor's colours,
/// with `running`'s style added over the instruction running, if any.
pub fn program_styles(
    code: &str,
    resolver: &dyn HighlightStyleResolver,
    running: Option<(Range<usize>, HighlightStyle)>,
) -> Vec<(Range<usize>, HighlightStyle)> {
    let coloured = styles(&runs(code), &(0..code.len()), resolver);
    let Some((run, over)) = running.filter(|(r, _)| r.end <= code.len()) else { return coloured };
    // gpui wants runs that don't overlap, so a token the instruction starts
    // or ends in is split there.
    let mut out = Vec::new();
    for (r, style) in coloured {
        let start = r.start.max(run.start).min(r.end);
        let end = r.end.min(run.end).max(start);
        for (piece, style) in [(r.start..start, style), (start..end, style.highlight(over)), (end..r.end, style)] {
            if !piece.is_empty() {
                out.push((piece, style));
            }
        }
    }
    out
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
        self.runs = runs(&text.to_string());
    }

    fn styles(&self, range: &Range<usize>, resolver: &dyn HighlightStyleResolver) -> Vec<(Range<usize>, HighlightStyle)> {
        styles(&self.runs, range, resolver)
    }

    fn fold_ranges(&self, _: &Rope) -> Vec<FoldRange> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use gpui_kit::component::input::HighlightStyleResolver;
    use gpui_kit::{FontWeight, HighlightStyle, Hsla};

    use super::program_styles;

    /// Gives each kind of token a colour of its own: a hue from its name.
    struct ByName;

    impl HighlightStyleResolver for ByName {
        fn style(&self, name: &str) -> Option<HighlightStyle> {
            Some(HighlightStyle { color: Some(Hsla { h: name.len() as f32 / 100., s: 1., l: 0.5, a: 1. }), ..Default::default() })
        }
    }

    fn colour(name: &str) -> Option<Hsla> {
        ByName.style(name).unwrap().color
    }

    #[test]
    fn programs_are_coloured_like_the_editor() {
        let styles = program_styles("FD :size", &ByName, None);
        let ranges: Vec<Range<usize>> = styles.iter().map(|(r, _)| r.clone()).collect();
        assert_eq!(ranges, [0..2, 2..3, 3..8]);
        assert_eq!(styles[0].1.color, colour("keyword"));
        assert_eq!(styles[1].1, HighlightStyle::default());
        assert_eq!(styles[2].1.color, colour("variable"));
    }

    #[test]
    fn the_running_instruction_is_split_out_and_keeps_its_colours() {
        let bold = HighlightStyle { font_weight: Some(FontWeight::BOLD), ..Default::default() };
        // `RT 90` running, in `REPEAT 4 [FD 10 RT 90]`, and part of a token.
        for (run, expected) in [(16..21, vec![16..18, 18..19, 19..21]), (1..3, vec![Range { start: 1, end: 3 }])] {
            let styles = program_styles("REPEAT 4 [FD 10 RT 90]", &ByName, Some((run.clone(), bold)));
            let mut at = 0;
            for (r, _) in &styles {
                assert_eq!(r.start, at, "runs cover the program in order: {styles:?}");
                at = r.end;
            }
            assert_eq!(at, 22);
            let bolded: Vec<_> = styles.iter().filter(|(_, s)| s.font_weight.is_some()).map(|(r, _)| r.clone()).collect();
            assert_eq!(bolded, expected);
            // A token keeps its colour where the instruction starts.
            assert!(styles.iter().any(|(r, s)| r.start == run.start && s.color.is_some()));
        }
    }
}
