//! The help panel's markup, a small part of Markdown (phase 5): paragraphs,
//! `code`, `**bold**`, `*italics*`, lists of lines starting `- `, headings,
//! notes starting `> `, fenced code blocks, and `[NAME]`, a link to the
//! help for a command or a syntax entry. The help text in
//! `content/reference.kdl` and the tutorial are written in it.

#[derive(Clone, Debug, PartialEq)]
pub enum Inline {
    Text(String),
    Code(String),
    Bold(String),
    Italic(String),
    /// A link to a command's page, by name or syntax key: `[FORWARD]`.
    Link(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    Heading(u8, Vec<Inline>),
    Paragraph(Vec<Inline>),
    List(Vec<Vec<Inline>>),
    /// A note set apart, such as the tutorial's "Try this".
    Note(Vec<Inline>),
    /// A fenced code block: a program, with the language after the fence,
    /// if any, and a name after that (such as ```` ```logo tree ````), so the
    /// gallery can point at it. Markdown viewers read only the language.
    Code {
        language: String,
        name: Option<String>,
        code: String,
    },
}

/// What's between `marker` at the start of `text` and the next `marker`,
/// if there is one and it isn't empty.
fn closed<'a>(text: &'a str, marker: &str) -> Option<&'a str> {
    let rest = text.strip_prefix(marker)?;
    rest.find(marker)
        .filter(|&end| end > 0)
        .map(|end| &rest[..end])
}

/// Splits a line of text into its inline parts. Unclosed markers are left
/// as text.
pub fn inlines(text: &str) -> Vec<Inline> {
    let mut out = Vec::new();
    let mut text_start = 0;
    let mut i = 0;
    let bytes = text.as_bytes();
    let flush = |out: &mut Vec<Inline>, from: usize, to: usize| {
        if to > from {
            match out.last_mut() {
                Some(Inline::Text(t)) => t.push_str(&text[from..to]),
                _ => out.push(Inline::Text(text[from..to].to_string())),
            }
        }
    };
    while i < bytes.len() {
        let rest = &text[i..];
        let found = if let Some(inner) = closed(rest, "**") {
            Some((Inline::Bold(inner.to_string()), inner.len() + 4))
        } else if let Some(inner) = closed(rest, "`") {
            Some((Inline::Code(inner.to_string()), inner.len() + 2))
        } else if let Some(inner) =
            closed(rest, "*").filter(|i| !i.starts_with(' ') && !i.ends_with(' '))
        {
            Some((Inline::Italic(inner.to_string()), inner.len() + 2))
        } else if let Some(inner) = rest
            .strip_prefix('[')
            .and_then(|r| r.split_once(']'))
            .map(|(inner, _)| inner)
        {
            // A link is a name alone in brackets: `[1 2 3]` in prose isn't one.
            (!inner.is_empty() && !inner.contains(char::is_whitespace))
                .then(|| (Inline::Link(inner.to_string()), inner.len() + 2))
        } else {
            None
        };
        match found {
            Some((inline, len)) => {
                flush(&mut out, text_start, i);
                out.push(inline);
                i += len;
                text_start = i;
            }
            None => i += rest.chars().next().map_or(1, char::len_utf8),
        }
    }
    flush(&mut out, text_start, text.len());
    out
}

/// Parses a whole text into blocks.
pub fn parse(text: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut paragraph: Vec<&str> = Vec::new();
    let mut lines = text.lines().peekable();
    let end_paragraph = |paragraph: &mut Vec<&str>, blocks: &mut Vec<Block>| {
        if !paragraph.is_empty() {
            blocks.push(Block::Paragraph(inlines(&paragraph.join(" "))));
            paragraph.clear();
        }
    };
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if let Some(info) = trimmed.strip_prefix("```") {
            end_paragraph(&mut paragraph, &mut blocks);
            let mut code = Vec::new();
            for line in lines.by_ref() {
                if line.trim_start().starts_with("```") {
                    break;
                }
                code.push(line);
            }
            let mut words = info.split_whitespace();
            blocks.push(Block::Code {
                language: words.next().unwrap_or_default().to_string(),
                name: words.next().map(str::to_string),
                code: code.join("\n"),
            });
        } else if trimmed.is_empty() {
            end_paragraph(&mut paragraph, &mut blocks);
        } else if let Some(item) = trimmed.strip_prefix("- ") {
            end_paragraph(&mut paragraph, &mut blocks);
            let mut items = vec![item.to_string()];
            while let Some(next) = lines.peek().map(|l| l.trim()) {
                if let Some(item) = next.strip_prefix("- ") {
                    items.push(item.to_string());
                } else if !next.is_empty()
                    && !next.starts_with("```")
                    && !next.starts_with('#')
                    && !next.starts_with('>')
                {
                    let last = items.last_mut().expect("there's at least one");
                    last.push(' ');
                    last.push_str(next);
                } else {
                    break;
                }
                lines.next();
            }
            blocks.push(Block::List(items.iter().map(|i| inlines(i)).collect()));
        } else if let Some(note) = trimmed.strip_prefix('>') {
            end_paragraph(&mut paragraph, &mut blocks);
            let mut text = vec![note.trim().to_string()];
            while let Some(next) = lines.peek().and_then(|l| l.trim().strip_prefix('>')) {
                text.push(next.trim().to_string());
                lines.next();
            }
            blocks.push(Block::Note(inlines(&text.join(" "))));
        } else if trimmed.starts_with('#') {
            end_paragraph(&mut paragraph, &mut blocks);
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            blocks.push(Block::Heading(
                level as u8,
                inlines(trimmed[level..].trim()),
            ));
        } else {
            paragraph.push(trimmed);
        }
    }
    end_paragraph(&mut paragraph, &mut blocks);
    blocks
}

/// Every link's name in some blocks.
pub fn links(blocks: &[Block]) -> Vec<&str> {
    fn inline_links(inlines: &[Inline]) -> Vec<&str> {
        inlines
            .iter()
            .filter_map(|i| {
                if let Inline::Link(name) = i {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect()
    }
    blocks
        .iter()
        .flat_map(|b| match b {
            Block::Heading(_, i) | Block::Paragraph(i) | Block::Note(i) => inline_links(i),
            Block::List(items) => items.iter().flat_map(|i| inline_links(i)).collect(),
            Block::Code { .. } => Vec::new(),
        })
        .collect()
}

/// Inline parts as plain text.
pub fn plain(inlines: &[Inline]) -> String {
    inlines
        .iter()
        .map(|i| match i {
            Inline::Text(t)
            | Inline::Code(t)
            | Inline::Bold(t)
            | Inline::Italic(t)
            | Inline::Link(t) => t.as_str(),
        })
        .collect()
}

/// The first sentence of some text, without its markup, for a search
/// result: up to the first full stop followed by a space, or the end of the
/// first paragraph.
pub fn summary(text: &str) -> String {
    let first = text
        .split("\n\n")
        .next()
        .unwrap_or_default()
        .replace('\n', " ");
    let plain = plain(&inlines(&first));
    match plain.find(". ") {
        Some(end) => plain[..=end].to_string(),
        None => plain,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &str) -> Inline {
        Inline::Text(s.into())
    }

    #[test]
    fn inline_markup() {
        assert_eq!(
            inlines("Kame moves `FD 10` **far**, *really*. See [BACK]."),
            [
                text("Kame moves "),
                Inline::Code("FD 10".into()),
                text(" "),
                Inline::Bold("far".into()),
                text(", "),
                Inline::Italic("really".into()),
                text(". See "),
                Inline::Link("BACK".into()),
                text(".")
            ]
        );
    }

    #[test]
    fn unclosed_markers_and_lists_in_prose_are_text() {
        assert_eq!(inlines("2 * 3 and `oops"), [text("2 * 3 and `oops")]);
        assert_eq!(inlines("2 * 3 = 3 * 2"), [text("2 * 3 = 3 * 2")]);
        assert_eq!(
            inlines("a list [1 2 3] here"),
            [text("a list [1 2 3] here")]
        );
    }

    #[test]
    fn blocks() {
        let blocks = parse(
            "# Moving\n\nOne\nparagraph.\n\n- first\n- second\n  more\n\n> Try this\n\n```logo\nFD 10\nRT 90\n```\nAfter.",
        );
        assert_eq!(
            blocks,
            [
                Block::Heading(1, vec![text("Moving")]),
                Block::Paragraph(vec![text("One paragraph.")]),
                Block::List(vec![vec![text("first")], vec![text("second more")]]),
                Block::Note(vec![text("Try this")]),
                Block::Code {
                    language: "logo".into(),
                    name: None,
                    code: "FD 10\nRT 90".into()
                },
                Block::Paragraph(vec![text("After.")]),
            ]
        );
    }

    #[test]
    fn a_code_block_can_be_named_after_its_language() {
        assert_eq!(
            parse("```logo tree\nTREE 120\n```"),
            [Block::Code {
                language: "logo".into(),
                name: Some("tree".into()),
                code: "TREE 120".into()
            }]
        );
    }

    #[test]
    fn links_are_collected() {
        let blocks = parse("See [BACK].\n\n- and [LEFT]\n\n> or [RIGHT]");
        assert_eq!(links(&blocks), ["BACK", "LEFT", "RIGHT"]);
    }

    #[test]
    fn summaries() {
        assert_eq!(
            summary("Kame moves forward. If the pen is down, it draws."),
            "Kame moves forward."
        );
        assert_eq!(
            summary("Outputs `TRUE` if so, and `FALSE` if not.\n\nMore."),
            "Outputs TRUE if so, and FALSE if not."
        );
        assert_eq!(
            summary("Sets the colour. A colour can be:\n- red"),
            "Sets the colour."
        );
    }
}
