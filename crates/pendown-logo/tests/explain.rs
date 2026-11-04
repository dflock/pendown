//! What's shown about an error beyond its message.

mod common;

use pendown_logo::{
    RunError,
    explain::{Explanation, explain},
    reference::reference,
};

/// The explanation of a program's first error, from the parser or the run.
fn explained(program: &str) -> Explanation {
    match &common::run(program).result {
        Err(RunError::Parse(errors)) => explain(&errors[0], program),
        Err(RunError::Runtime(error)) => explain(error, program),
        other => panic!("expected an error, got {other:?}"),
    }
}

fn page(title: &str) -> Option<usize> {
    reference().commands.iter().position(|c| c.title == title)
}

#[test]
fn a_command_says_how_it_is_written_and_shows_an_example() {
    let e = explained("REPEAT 4");
    assert_eq!(e.message, "REPEAT needs 2 inputs, but got 1");
    assert_eq!(e.usage.as_deref(), Some("REPEAT number [instructions]"));
    assert_eq!(e.example.as_deref(), Some("REPEAT 4 [FD 100 RT 90]"));
    assert_eq!(e.defined_on, None);
    assert_eq!(e.help, page("REPEAT"));
}

#[test]
fn each_name_gets_its_own_line() {
    assert_eq!(explained("FD").usage.as_deref(), Some("FD distance"));
    assert_eq!(
        explained("FORWARD").usage.as_deref(),
        Some("FORWARD distance")
    );
    assert_eq!(explained("forward").help, page("FORWARD"));
    assert_eq!(explained("SETPC").usage.as_deref(), Some("SETPC colour"));
}

#[test]
fn an_operator_gets_its_own_line() {
    let e = explained("PRINT \"a + 1");
    assert_eq!(e.usage.as_deref(), Some("number + number"));
    assert_eq!(e.help, page("SUM"));
}

#[test]
fn the_form_in_parentheses_is_not_chosen_first() {
    assert_eq!(
        explained("SUM 1").usage.as_deref(),
        Some("SUM number number")
    );
}

#[test]
fn manual_notes_are_left_out() {
    assert_eq!(explained("ISEQ 1").usage.as_deref(), Some("ISEQ from to"));
}

#[test]
fn input_names_are_made_plain() {
    for (program, usage) in [
        ("IF", "IF condition [instructions]"),
        ("PRINT", "PRINT value"),
        ("MAKE", "MAKE \"name value"),
        ("SETPOS", "SETPOS [x y]"),
        ("SETXY 1", "SETXY x y"),
        ("BF", "BF word-or-list"),
    ] {
        assert_eq!(
            explained(program).usage.as_deref(),
            Some(usage),
            "{program}"
        );
    }
}

#[test]
fn a_command_with_no_try_shows_its_syntax_alone() {
    let e = explained("ARCTAN");
    assert_eq!(e.usage.as_deref(), Some("ARCTAN number"));
    assert_eq!(e.example, None);
    assert_eq!(e.help, page("ARCTAN"));
}

#[test]
fn a_procedure_with_inputs_says_where_it_is() {
    let e = explained("FD 10\nTO SQ :n :m\n  FD :n\nEND\nSQ 1");
    assert_eq!(e.message, "SQ needs 2 inputs, but got 1");
    assert_eq!(e.usage.as_deref(), Some("SQ :n :m"));
    assert_eq!(e.example, None);
    assert_eq!(e.defined_on, Some(2));
    assert_eq!(e.help, None);
}

#[test]
fn a_procedure_without_inputs_is_found_in_any_case() {
    let e = explained("  to Sq ; a square\nFD 1\nend\n(SQ 1)");
    assert_eq!(e.message, "SQ takes no inputs, but got 1");
    assert_eq!(e.usage.as_deref(), Some("Sq"));
    assert_eq!(e.defined_on, Some(1));
}

#[test]
fn a_one_line_procedure_stops_at_its_inputs() {
    let e = explained("TO TWICE :n OP :n * 2 END\nPRINT TWICE");
    assert_eq!(e.usage.as_deref(), Some("TWICE :n"));
}

#[test]
fn an_error_with_no_command_has_only_its_message() {
    let e = explained("REPEAT 4 [FD 50]]");
    assert_eq!(
        e,
        Explanation {
            message: "This ] has no matching [".into(),
            ..Explanation::default()
        }
    );
    assert_eq!(explained("FD :size").usage, None);
    assert_eq!(explained("FOOBAR").help, None);
}

#[test]
fn an_obsolete_command_offers_help_but_no_syntax() {
    let e = explained("EDIT \"x");
    assert!(
        e.message.starts_with("EDIT isn't in Pendown"),
        "{}",
        e.message
    );
    assert_eq!(e.usage, None);
    assert_eq!(e.example, None);
    assert_eq!(e.help, page("EDIT"));
}
