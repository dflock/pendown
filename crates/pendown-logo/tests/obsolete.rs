//! Obsolete commands say why they aren't here, and errors name the command
//! they're about, for help on it (phase 5 issue 02).

mod common;

use common::run;
use pendown_logo::{RunError, reference::reference};

/// The first error, from parsing or running.
fn first_error(src: &str) -> pendown_logo::Error {
    match run(src).result {
        Err(RunError::Parse(errors)) => errors[0].clone(),
        Err(RunError::Runtime(e)) => e,
        other => panic!("{src}: expected an error, got {other:?}"),
    }
}

#[test]
fn an_obsolete_command_says_why() {
    let e = first_error("EDIT \"square");
    assert_eq!(
        e.message,
        "EDIT isn't in Pendown: it opens an external text editor, and Pendown has its own."
    );
    assert_eq!(e.command.as_deref(), Some("EDIT"));
    // Called by an alias, it's named as it was called.
    assert!(
        first_error("ED")
            .message
            .starts_with("ED isn't in Pendown: it opens an external text editor")
    );
    assert!(
        first_error("TO TILE\n PO \"tile\nEND\nTILE")
            .message
            .starts_with("PO isn't in Pendown: ")
    );
}

#[test]
fn an_unknown_name_still_doesnt_know_how() {
    let e = first_error("FROBNICATE 3");
    assert_eq!(e.message, "I don't know how to FROBNICATE");
    assert_eq!(e.command, None);
}

#[test]
fn errors_name_their_command() {
    assert_eq!(first_error("FD").command.as_deref(), Some("FD"));
    assert_eq!(first_error("(SETPC)").command.as_deref(), Some("SETPC"));
    assert_eq!(
        first_error("SETPC \"tealish").command.as_deref(),
        Some("SETPC")
    );
    assert_eq!(
        first_error("PRINT SQRT -1").command.as_deref(),
        Some("SQRT")
    );
}

#[test]
fn every_reason_reads_as_the_end_of_a_sentence() {
    for command in &reference().commands {
        let Some(reason) = &command.obsolete else {
            continue;
        };
        let first = reason.chars().next().unwrap();
        assert!(
            first.is_lowercase() || reason.starts_with("UCBLogo"),
            "{}: starts with a capital: {reason}",
            command.key
        );
        assert!(
            reason.ends_with('.'),
            "{}: no full stop: {reason}",
            command.key
        );
        assert!(
            !reason.contains("GUI"),
            "{}: say Pendown, not the GUI: {reason}",
            command.key
        );
    }
}
