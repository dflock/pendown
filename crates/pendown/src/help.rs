//! The help panel's parts that don't need a window (phase 5): how examples
//! are drawn for their pictures.

use pendown_cli::Options;

/// The panel shows the pictures of examples this many logical pixels
/// square. They're SVGs, fitted to the drawing, and rendered at exactly the
/// size they're shown.
pub const PICTURE_SIZE: u32 = 84;

/// How an example is drawn for its picture: fitted, with `RANDOM`'s seed
/// fixed so the picture is always the same, as in the tests. No line is
/// thinner than a pixel, or thin lines would be grey on a 1× screen. It's
/// shown from its SVG, so it needs no pixels.
pub fn picture_options() -> Options {
    let thinnest = 1.0 / PICTURE_SIZE as f64;
    Options {
        width: PICTURE_SIZE,
        height: PICTURE_SIZE,
        limit: 1_000_000,
        seed: Some(1),
        fit: true,
        svg: Some(thinnest),
        crop: false,
        pixels: false,
    }
}

/// A lesson's picture: as [`picture_options`], but the shape of the
/// drawing (`.scratch/lesson-8-rules/` issue 02).
pub fn lesson_picture_options() -> Options {
    Options {
        crop: true,
        ..picture_options()
    }
}

/// An example's name, for its saved picture: `forward-1` for FORWARD's
/// first.
pub fn example_name(key: &str, index: usize) -> String {
    format!("{key}-{}", index + 1)
}

/// What the help panel should show for the editor's cursor at byte
/// `offset` in `text`: the name to look up, and where that token is. A
/// variable shows the help for `:`, a number for numbers, and so on. A
/// procedure the program defines has none.
pub fn word_at(text: &str, offset: usize) -> Option<(String, std::ops::Range<usize>)> {
    use pendown_logo::syntax::{Kind, highlight};
    let tokens = highlight(text);
    // The token the cursor is in, or just after, as when typing.
    let (range, kind) = tokens
        .iter()
        .find(|(r, _)| r.start <= offset && offset < r.end)
        .or_else(|| tokens.iter().find(|(r, _)| r.end == offset))?
        .clone();
    let word = &text[range.clone()];
    let name = match kind {
        Kind::Procedure => return None,
        Kind::Variable => ":".to_string(),
        Kind::Quoted => "\"".to_string(),
        Kind::Number => "numbers".to_string(),
        Kind::Comment => ";".to_string(),
        // `?2` is a template slot, like `?`.
        _ if word.starts_with('?') && word[1..].chars().all(|c| c.is_ascii_digit()) => {
            "?".to_string()
        }
        Kind::Bracket | Kind::Operator | Kind::Keyword | Kind::Command | Kind::Unknown => {
            word.to_string()
        }
    };
    Some((name, range))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str, marker: &str) -> Option<String> {
        let offset = text.find(marker).unwrap();
        let text = text.replace('|', "");
        word_at(&text, offset).map(|(name, _)| name)
    }

    #[test]
    fn the_word_under_the_cursor() {
        assert_eq!(at("REPEAT 4 [F|D 10 RT 90]", "|").as_deref(), Some("FD"));
        // Just after a word, as when typing it.
        assert_eq!(at("FD| 10", "|").as_deref(), Some("FD"));
        assert_eq!(at("FD :si|ze", "|").as_deref(), Some(":"));
        assert_eq!(at("SETPC \"re|d", "|").as_deref(), Some("\""));
        assert_eq!(at("FD 1|0", "|").as_deref(), Some("numbers"));
        assert_eq!(at("PRINT 2 |+ 3", "|").as_deref(), Some("+"));
        assert_eq!(at("REPEAT 4 |[FD 10]", "|").as_deref(), Some("["));
        assert_eq!(at("; a com|ment", "|").as_deref(), Some(";"));
        assert_eq!(at("SHOW MAP [?|2] [1]", "|").as_deref(), Some("?"));
        assert_eq!(at("EDI|T", "|").as_deref(), Some("EDIT"));
    }

    #[test]
    fn a_procedure_of_the_program_has_no_help() {
        assert_eq!(at("TO BOX\nEND\nBO|X", "|"), None);
        assert_eq!(at("FD 10   |  RT 90", "|"), None);
    }
}
