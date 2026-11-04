//! "Did you mean": what a word that isn't a command was probably meant to
//! be. A wrong guess could mislead a beginner, so it only speaks up for a
//! very close match, with no other as close.

use crate::{parse::Arity, prims::PRIMS};

/// What to say a word might have been, and the command that's about.
#[derive(Debug, PartialEq)]
pub struct Suggestion {
    /// `FORWARD`, or `FD 100`.
    pub text: String,
    pub command: String,
}

/// Words shorter than this are left alone: too many things are close to them.
const SHORTEST: usize = 4;
/// Words this long can be two edits away from what was meant.
const LONG: usize = 7;

/// Edits between two words: a letter added, dropped, changed, or swapped with
/// the next, the slips typing makes.
fn distance(a: &[char], b: &[char]) -> usize {
    let mut before = vec![0; b.len() + 1];
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut row = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let mut best = (previous[j] + usize::from(ca != cb))
                .min(previous[j + 1] + 1)
                .min(row[j] + 1);
            if i > 0 && j > 0 && *ca == b[j - 1] && a[i - 1] == *cb {
                best = best.min(before[j - 1] + 1);
            }
            row.push(best);
        }
        before = std::mem::replace(&mut previous, row);
    }
    previous[b.len()]
}

/// The commands, and `procedures`, as upper-case names with how many inputs
/// they usually take. TO and END aren't primitives, but can be misspelt.
fn known(procedures: &[(String, Arity)]) -> Vec<(String, usize)> {
    let commands = PRIMS.iter().flat_map(|prim| {
        prim.names
            .iter()
            .map(|name| (name.to_uppercase(), prim.arity.default))
    });
    let procedures = procedures
        .iter()
        .map(|(name, arity)| (name.to_uppercase(), arity.default));
    let syntax = [("TO".to_string(), 1), ("END".to_string(), 0)];
    commands.chain(procedures).chain(syntax).collect()
}

/// `FD100` as `FD 100`: a command that takes an input, then a number.
fn missing_space(word: &str, known: &[(String, usize)]) -> Option<Suggestion> {
    let name = word.trim_end_matches(|c: char| c.is_ascii_digit() || c == '.');
    let number = &word[name.len()..];
    if name.is_empty() || !number.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    number.parse::<f64>().ok()?;
    known
        .iter()
        .find(|(known, inputs)| known == name && *inputs > 0)
        .map(|(known, _)| Suggestion {
            text: format!("{known} {number}"),
            command: known.clone(),
        })
}

/// A misspelling: the one known name closest to it, if that's close enough.
fn misspelt(word: &str, known: &[(String, usize)]) -> Option<Suggestion> {
    let letters: Vec<char> = word.chars().collect();
    if letters.len() < SHORTEST {
        return None;
    }
    let furthest = if letters.len() >= LONG { 2 } else { 1 };
    let mut closest: Vec<&str> = Vec::new();
    let mut nearest = furthest + 1;
    for (name, _) in known {
        let d = distance(&letters, &name.chars().collect::<Vec<_>>());
        if d > furthest {
            continue;
        }
        if d < nearest {
            nearest = d;
            closest.clear();
        }
        if d == nearest && !closest.contains(&name.as_str()) {
            closest.push(name);
        }
    }
    match closest[..] {
        [name] => Some(Suggestion {
            text: name.to_string(),
            command: name.to_string(),
        }),
        _ => None,
    }
}

/// What `word`, which isn't a command, might have been meant as.
pub fn suggest(word: &str, procedures: &[(String, Arity)]) -> Option<Suggestion> {
    let word = word.to_uppercase();
    let known = known(procedures);
    missing_space(&word, &known).or_else(|| misspelt(&word, &known))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(word: &str) -> Option<String> {
        suggest(word, &[]).map(|s| s.text)
    }

    #[test]
    fn real_looking_typos_are_caught() {
        for (typo, meant) in [
            ("FORWRD", "FORWARD"),
            ("FOWARD", "FORWARD"),
            ("FORWARDS", "FORWARD"),
            ("forwrd", "FORWARD"),
            ("RIHGT", "RIGHT"),
            ("RIGTH", "RIGHT"),
            ("LETF", "LEFT"),
            ("REPAT", "REPEAT"),
            ("REPEET", "REPEAT"),
            ("RPEAT", "REPEAT"),
            ("PRNT", "PRINT"),
            ("PIRNT", "PRINT"),
            ("PENUPP", "PENUP"),
            ("PNEUP", "PENUP"),
            ("CIRCEL", "CIRCLE"),
            ("SQAURE", "SQUARE"),
            ("RECTANLGE", "RECTANGLE"),
            ("SETPENCOLR", "SETPENCOLOR"),
            ("STPO", "STOP"),
            ("MAEK", "MAKE"),
        ] {
            assert_eq!(text(typo).as_deref(), Some(meant), "{typo}");
        }
    }

    #[test]
    fn other_words_get_no_suggestion() {
        for word in [
            "X", "AB", "SQ", "FOO", "BOX", "CAT", "TRI", "ROOF", "TREE", "STAR", "KOCH", "WALK",
            "DRAW", "HOUSE", "AHEAD", "GUIDE", "GASKET", "SPIRAL", "FLOWER", "FOOBAR", "POLYGON",
            "CORNERS", "TRIANGLE",
        ] {
            assert_eq!(text(word), None, "{word}");
        }
    }

    #[test]
    fn a_short_word_can_be_only_one_edit_away() {
        assert_eq!(text("FRWRD"), None);
    }

    #[test]
    fn a_command_right_up_against_a_number_gets_a_space() {
        assert_eq!(text("FD100").as_deref(), Some("FD 100"));
        assert_eq!(text("rt90").as_deref(), Some("RT 90"));
        assert_eq!(text("FORWARD2.5").as_deref(), Some("FORWARD 2.5"));
        assert_eq!(suggest("FD100", &[]).unwrap().command, "FD");
    }

    #[test]
    fn digits_after_anything_else_get_no_space() {
        for word in ["BOX100", "CS5", "FD1.2.3", "100"] {
            assert_eq!(text(word), None, "{word}");
        }
    }

    #[test]
    fn a_learners_procedures_are_offered() {
        let procedures = [("Staircase".to_string(), Arity::fixed(1))];
        let misspelt = suggest("STARICASE", &procedures).unwrap();
        assert_eq!(misspelt.text, "STAIRCASE");
        assert_eq!(misspelt.command, "STAIRCASE");
        assert_eq!(
            suggest("STAIRCASE9", &procedures).unwrap().text,
            "STAIRCASE 9"
        );
    }

    #[test]
    fn two_equally_close_names_get_no_suggestion() {
        let procedures = [
            ("HOUSES".to_string(), Arity::fixed(0)),
            ("MOUSES".to_string(), Arity::fixed(0)),
        ];
        assert_eq!(suggest("LOUSES", &procedures), None);
    }

    #[test]
    fn obsolete_commands_are_not_offered() {
        // EDIT is in the reference, but not implemented.
        assert_eq!(text("EDITT"), None);
    }

    #[test]
    fn distance_counts_a_swap_as_one() {
        let d = |a: &str, b: &str| {
            distance(
                &a.chars().collect::<Vec<_>>(),
                &b.chars().collect::<Vec<_>>(),
            )
        };
        assert_eq!(d("LETF", "LEFT"), 1);
        assert_eq!(d("FORWRD", "FORWARD"), 1);
        assert_eq!(d("abc", "abc"), 0);
        assert_eq!(d("", "abc"), 3);
        assert_eq!(d("kitten", "sitting"), 3);
    }
}
