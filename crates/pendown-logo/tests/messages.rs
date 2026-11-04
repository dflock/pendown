//! What errors say to a learner.

mod common;

use common::*;

#[test]
fn inputs_say_what_was_wanted() {
    assert_eq!(error("FD \"x"), "FD needs a number, but got the word x");
    assert_eq!(error("RT [90]"), "RT needs a number, but got the list [90]");
    assert_eq!(error("RT []"), "RT needs a number, but got the empty list");
    assert_eq!(error("FD \""), "FD needs a number, but got the empty word");
    assert_eq!(
        error("SETPOS 5"),
        "SETPOS needs a list of two numbers, but got 5"
    );
    assert_eq!(
        error("RANDOM 2.5"),
        "RANDOM needs a whole number, but got 2.5"
    );
    assert_eq!(error("IF 5 [FD 1]"), "IF needs TRUE or FALSE, but got 5");
    assert_eq!(
        error("MAKE [a] 1"),
        "MAKE needs a word, but got the list [a]"
    );
}

#[test]
fn a_value_that_is_the_right_kind_but_not_allowed_keeps_its_wording() {
    assert_eq!(error("SQRT -1"), "SQRT doesn't like -1 as input");
    assert_eq!(error("REPEAT -1 [FD 1]"), "REPEAT doesn't like -1 as input");
}

#[test]
fn instructions_with_no_brackets() {
    let message = "REPEAT needs a list of instructions here. Put them in [ ]";
    assert_eq!(error("REPEAT 4 FD 50"), message);
    assert_eq!(
        error("IF \"true FD 50"),
        "IF needs a list of instructions here. Put them in [ ]"
    );
    assert_eq!(
        error("FOREVER FD 50"),
        "FOREVER needs a list of instructions here. Put them in [ ]"
    );
    assert_eq!(
        error("TO NOTHING\nEND\nREPEAT 4 NOTHING"),
        message,
        "a procedure that outputs nothing, too"
    );
    // The help's about the command that wanted the list.
    assert_eq!(
        run("REPEAT 4 FD 50").error().command.as_deref(),
        Some("REPEAT")
    );
}

#[test]
fn a_value_where_instructions_should_be() {
    assert_eq!(
        error("REPEAT 4 5"),
        "REPEAT needs a list of instructions, but got 5"
    );
    assert_eq!(
        error("REPEAT 4 \"FD"),
        "REPEAT needs a list of instructions, but got the word FD"
    );
}

#[test]
fn other_missing_outputs_keep_their_wording() {
    assert_eq!(error("PRINT FD 10"), "FD didn't output to PRINT");
    // Inside the list, it isn't the list that's missing.
    assert_eq!(error("REPEAT 4 [PRINT FD 1]"), "FD didn't output to PRINT");
}

#[test]
fn a_variable_with_no_value_says_how_to_give_it_one() {
    assert_eq!(
        error("FD :size"),
        ":size has no value yet. Give it one with MAKE \"size 100"
    );
    // A procedure's input only has a value inside it.
    assert_eq!(
        error("TO SQ :n\nFD :n\nEND\nSQ 10\nFD :n"),
        ":n has no value yet. Give it one with MAKE \"n 100"
    );
}

#[test]
fn brackets_and_end() {
    let parse = |src| {
        let r = run(src);
        let Err(pendown_logo::RunError::Parse(errors)) = r.result else {
            panic!("{src} parsed")
        };
        errors[0].message.clone()
    };
    assert_eq!(parse("REPEAT 4 [FD 50]]"), "This ] has no matching [");
    assert_eq!(parse("SHOW a}"), "This } has no matching {");
    assert_eq!(parse("FD 1)"), "This ) has no matching (");
    assert_eq!(
        parse("TO SQ\nFD 10"),
        "SQ has no END. Add END after its last line"
    );
}
