//! The language reference, from `content/reference.kdl` (phase 5 issue 01).

use pendown_logo::reference::{Kind, reference};

fn title(i: usize) -> &'static str {
    &reference().commands[i].title
}

#[test]
fn every_phase_1_and_2_command_is_implemented() {
    let r = reference();
    let implemented: Vec<_> = r
        .commands
        .iter()
        .filter(|c| c.implemented && c.kind != Kind::Syntax)
        .collect();
    assert_eq!(
        implemented.len(),
        90 + 76,
        "the plan's counts for phases 1 and 2"
    );
    assert!(implemented.iter().all(|c| matches!(c.phase, Some(1 | 2))));
}

#[test]
fn commands_are_found_by_any_name_in_any_case() {
    let r = reference();
    assert_eq!(r.find("fd").unwrap().title, "FORWARD");
    assert_eq!(r.find("Forward").unwrap().key, "forward");
    assert_eq!(r.find("SETPC").unwrap().title, "SETPENCOLOR");
    assert_eq!(r.find("+").unwrap().title, "SUM");
    assert_eq!(r.find("<>").unwrap().title, "NOTEQUALP");
    assert_eq!(r.find("caseignoredp").unwrap().kind, Kind::Variable);
    assert!(r.find("nosuchcommand").is_none());
}

#[test]
fn every_implemented_name_has_a_page() {
    let r = reference();
    for names in pendown_logo::command_names() {
        for name in names {
            let command = r.find(name).unwrap_or_else(|| panic!("no page for {name}"));
            assert!(
                command.implemented,
                "{name}'s page is for {}",
                command.title
            );
        }
    }
}

#[test]
fn chapters_and_sections() {
    let r = reference();
    let graphics = r.chapters.iter().find(|c| c.key == "graphics").unwrap();
    assert_eq!(graphics.title, "Graphics");
    let motion = graphics
        .sections
        .iter()
        .find(|s| s.key == "turtle-motion")
        .unwrap();
    assert_eq!(motion.title, "Turtle motion");
    assert_eq!(title(motion.commands[0]), "FORWARD");
    // The special variables are directly in their chapter.
    let specials = r
        .chapters
        .iter()
        .find(|c| c.key == "special-variables")
        .unwrap();
    assert!(specials.sections.is_empty() && !specials.commands.is_empty());
}

#[test]
fn search_finds_names_and_aliases_anywhere_in_them() {
    let r = reference();
    let titles = |q, obsolete| {
        r.search(q, obsolete)
            .into_iter()
            .map(title)
            .collect::<Vec<_>>()
    };
    assert_eq!(titles("fd", false), ["FORWARD"]);
    let setp = titles("setp", false);
    assert!(
        setp.contains(&"SETPENCOLOR") && setp.contains(&"SETPOS") && setp.contains(&"SETPENSIZE"),
        "{setp:?}"
    );
    assert!(
        !setp.contains(&"SETPENPATTERN"),
        "obsolete commands only when asked"
    );
    assert!(titles("setp", true).contains(&"SETPENPATTERN"));
    // An exact name comes first, then names starting with it, then names
    // with it anywhere.
    let pos = titles("pos", false);
    assert_eq!(pos[0], "POS");
    let at = |name| pos.iter().position(|t| *t == name).unwrap();
    assert!(at("SETPOS") > 0, "{pos:?}");
    assert!(
        pos.iter().take_while(|t| t.starts_with("POS")).count() <= at("SETPOS"),
        "{pos:?}"
    );
    let colour = titles("color", false);
    assert!(
        colour.contains(&"PENCOLOR")
            && colour.contains(&"SETPENCOLOR")
            && colour.contains(&"FILLCOLOR"),
        "{colour:?}"
    );
    assert!(titles("", false).is_empty());
}

#[test]
fn see_also_is_the_rest_of_the_section() {
    let r = reference();
    let forward = r.find_index("forward").unwrap();
    let see: Vec<_> = r.see_also(forward).into_iter().map(title).collect();
    assert!(
        see.contains(&"BACK") && !see.contains(&"FORWARD"),
        "{see:?}"
    );
}

#[test]
fn obsolete_commands_have_reasons() {
    let r = reference();
    assert!(r.obsolete_reason("edit").is_some());
    assert!(r.obsolete_reason("fd").is_none());
    let obsolete: Vec<_> = r.commands.iter().filter(|c| c.obsolete.is_some()).collect();
    assert!(obsolete.len() > 90);
}

#[test]
fn extensions_are_marked() {
    let r = reference();
    assert!(r.find("circle").unwrap().extension);
    assert!(!r.find("forward").unwrap().extension);
}

#[test]
fn every_implemented_command_and_syntax_entry_has_help_and_an_example() {
    let r = reference();
    let mut missing = Vec::new();
    for c in r.commands.iter().filter(|c| c.implemented) {
        if c.help.as_deref().is_none_or(|h| h.trim().is_empty()) {
            missing.push(format!("{}: help", c.key));
        }
        if c.examples.is_empty() {
            missing.push(format!("{}: example", c.key));
        }
    }
    assert!(missing.is_empty(), "{missing:#?}");
}

#[test]
fn syntax_entries_are_found_by_what_the_cursor_is_on() {
    let r = reference();
    assert_eq!(r.find(":").unwrap().key, "variables");
    assert_eq!(r.find("[").unwrap().key, "lists");
    assert_eq!(r.find("?rest").unwrap().key, "template-slots");
    assert_eq!(r.find("#").unwrap().title, "REPCOUNT");
    assert_eq!(r.find("end").unwrap().title, "TO");
    // And by key, as links name them.
    assert_eq!(r.find("parentheses").unwrap().title, "Parentheses: ( )");
    assert_eq!(r.chapters[0].title, "Syntax");
}

#[test]
fn see_lists_name_real_pages() {
    let r = reference();
    for c in r.commands.iter().filter(|c| c.implemented) {
        for name in &c.see {
            assert!(
                r.find(name).is_some_and(|s| s.implemented),
                "{}: see {name}",
                c.key
            );
        }
    }
}

#[test]
fn erract_s_example_fails_on_purpose() {
    let r = reference();
    assert!(r.find("erract").unwrap().examples[0].fails);
    assert!(!r.find("forward").unwrap().examples[0].fails);
}
