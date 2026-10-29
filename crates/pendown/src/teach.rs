//! Teach Kame: entries typed at the console become a procedure. The parts
//! that don't need a window: names, the procedure's text, and what it uses.

use pendown_logo::reference::reference;

/// The procedures `text` defines: the word after each `TO` that starts a
/// line, in capitals.
pub fn defined(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let to = words.next()?;
            to.eq_ignore_ascii_case("to")
                .then(|| words.next().map(str::to_uppercase))
                .flatten()
        })
        .collect()
}

/// The `TO … END` definitions in `text`, without its other instructions, so
/// a picture can use the program's procedures without running it.
pub fn definitions(text: &str) -> String {
    let mut out = String::new();
    let mut open = false;
    for line in text.lines() {
        let first = line.split_whitespace().next();
        if first.is_some_and(|w| w.eq_ignore_ascii_case("to")) {
            open = true;
        }
        if open {
            out.push_str(line);
            out.push('\n');
        }
        if line
            .split_whitespace()
            .any(|w| w.eq_ignore_ascii_case("end"))
        {
            open = false;
        }
    }
    out
}

/// Which of `names` an entry calls, in the order of `names`.
pub fn uses<'a>(entry: &str, names: &'a [String]) -> Vec<&'a str> {
    let words: Vec<String> = entry
        .split(|c: char| c.is_whitespace() || "[]()".contains(c))
        .filter(|w| !w.starts_with('"') && !w.starts_with(':'))
        .map(str::to_uppercase)
        .collect();
    names
        .iter()
        .filter(|name| words.iter().any(|w| w == *name))
        .map(String::as_str)
        .collect()
}

/// Whether `name` can be the new word: in capitals if so, or why not. An
/// empty name has no reason, since there's nothing wrong yet.
/// `in_program` and `at_console` are the procedures already defined there.
pub fn check_name(
    name: &str,
    in_program: &[String],
    at_console: &[String],
) -> Result<String, Option<String>> {
    let name = name.trim();
    if name.is_empty() {
        return Err(None);
    }
    let upper = name.to_uppercase();
    let why = if name.contains(char::is_whitespace) {
        "A name is one word, with no spaces.".to_string()
    } else if !name.starts_with(|c: char| c.is_alphabetic()) {
        "A name starts with a letter.".to_string()
    } else if !name
        .chars()
        .all(|c| c.is_alphanumeric() || ".?_".contains(c))
    {
        "A name has only letters, digits, and . _ or ?".to_string()
    } else if reference().find(name).is_some() {
        format!("{upper} is already a Logo word. Choose another name.")
    } else if in_program.contains(&upper) {
        format!(
            "{upper} is already in your program. Choose another name, or change it in the editor."
        )
    } else if at_console.contains(&upper) {
        format!("{upper} is already defined at the console. Choose another name.")
    } else {
        return Ok(upper);
    };
    Err(Some(why))
}

/// The procedure, as it goes in the editor: each entry on its own lines,
/// indented two spaces more than it was typed.
pub fn procedure(name: &str, entries: &[&str]) -> String {
    let mut text = format!("TO {name}\n");
    for entry in entries {
        for line in entry.lines() {
            text.push_str("  ");
            text.push_str(line);
            text.push('\n');
        }
    }
    text.push_str("END\n");
    text
}

/// What the dialog's picture draws: the procedures it may use, then the
/// entries, from the centre, facing up.
pub fn picture_program(definitions: &str, entries: &[&str]) -> String {
    let mut text = definitions.to_string();
    for entry in entries {
        text.push_str(entry);
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn what_a_program_defines() {
        let program =
            "; Squares\nto box :n\n  repeat 4 [fd :n rt 90]\nend\nTO Tri\nEND\nBOX 50\nPRINT \"to";
        assert_eq!(defined(program), ["BOX", "TRI"]);
        assert_eq!(
            definitions(program),
            "to box :n\n  repeat 4 [fd :n rt 90]\nend\nTO Tri\nEND\n"
        );
        assert_eq!(
            definitions("TO TWICE :n OUTPUT :n * 2 END\nTWICE 3"),
            "TO TWICE :n OUTPUT :n * 2 END\n"
        );
    }

    #[test]
    fn what_an_entry_uses() {
        let console = names(&["BOX", "TRI"]);
        assert_eq!(uses("box 40 rt 10", &console), ["BOX"]);
        assert_eq!(uses("REPEAT 3 [TRI 10 BOX 5]", &console), ["BOX", "TRI"]);
        assert!(uses("PRINT \"box MAKE \"tri 3 SHOW :box", &console).is_empty());
    }

    #[test]
    fn names_that_can_be_used() {
        let program = names(&["BOX"]);
        let console = names(&["STAR"]);
        let check = |name| check_name(name, &program, &console);
        assert_eq!(check("house"), Ok("HOUSE".into()));
        assert_eq!(check("  house2.big?_ "), Ok("HOUSE2.BIG?_".into()));
        // A Pendown extension, so taken too.
        assert_eq!(
            check("square").unwrap_err().unwrap(),
            "SQUARE is already a Logo word. Choose another name."
        );
        assert_eq!(check(""), Err(None));
        assert_eq!(check("  "), Err(None));
        let why = |name| check(name).unwrap_err().unwrap();
        assert_eq!(why("my square"), "A name is one word, with no spaces.");
        assert_eq!(why("2square"), "A name starts with a letter.");
        assert_eq!(
            why("sq-uare"),
            "A name has only letters, digits, and . _ or ?"
        );
        assert_eq!(why("fd"), "FD is already a Logo word. Choose another name.");
        assert_eq!(
            why("Forward"),
            "FORWARD is already a Logo word. Choose another name."
        );
        assert_eq!(why("to"), "TO is already a Logo word. Choose another name.");
        assert_eq!(
            why("box"),
            "BOX is already in your program. Choose another name, or change it in the editor."
        );
        assert_eq!(
            why("Star"),
            "STAR is already defined at the console. Choose another name."
        );
    }

    #[test]
    fn the_procedure() {
        assert_eq!(
            procedure("SQUARE", &["FD 100", "RT 90", "REPEAT 2 [\n  FD 5\n]"]),
            "TO SQUARE\n  FD 100\n  RT 90\n  REPEAT 2 [\n    FD 5\n  ]\nEND\n"
        );
        assert_eq!(
            picture_program("TO BOX\nEND\n", &["BOX", "FD 10"]),
            "TO BOX\nEND\nBOX\nFD 10\n"
        );
    }
}
