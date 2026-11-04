//! What errors say to a learner.

mod common;

use common::*;
use pendown_logo::RunError;

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

/// The first error found on reading a program, while typing or on Run.
fn on_reading(program: &str) -> String {
    let checked = pendown_logo::syntax::check(program);
    let Err(RunError::Parse(errors)) = run(program).result else {
        panic!("{program} parsed");
    };
    assert_eq!(checked[0].message, errors[0].message, "{program}");
    errors[0].message.clone()
}

#[test]
fn a_typo_gets_a_suggestion() {
    assert_eq!(
        on_reading("FORWRD 10"),
        "I don't know how to FORWRD. Did you mean FORWARD?"
    );
    assert_eq!(
        on_reading("FD100"),
        "I don't know how to FD100. Did you mean FD 100?"
    );
    assert_eq!(on_reading("BOX 10"), "I don't know how to BOX");
}

#[test]
fn a_suggestion_can_be_a_procedure_defined_anywhere() {
    assert_eq!(
        on_reading("STARICASE\nTO STAIRCASE\nEND"),
        "I don't know how to STARICASE. Did you mean STAIRCASE?"
    );
}

#[test]
fn a_suggestion_explains_the_command_it_means() {
    let program = "FORWRD 10";
    let errors = pendown_logo::syntax::check(program);
    assert_eq!(errors[0].command.as_deref(), Some("FORWARD"));
    let explanation = pendown_logo::explain::explain(&errors[0], program);
    assert_eq!(explanation.usage.as_deref(), Some("FORWARD distance"));
    assert_eq!(explanation.example.as_deref(), Some("FD 100"));

    let program = "STARICASE\nTO STAIRCASE :n\nEND";
    let errors = pendown_logo::syntax::check(program);
    let explanation = pendown_logo::explain::explain(&errors[0], program);
    assert_eq!(explanation.usage.as_deref(), Some("STAIRCASE :n"));
    assert_eq!(explanation.defined_on, Some(2));
}

#[test]
fn a_value_straight_after_a_call_says_nothing_uses_it() {
    assert_eq!(
        error("FORWARD 10 20"),
        "Nothing uses 20. FORWARD takes 1 input"
    );
    assert_eq!(error("FD 10 \"x"), "Nothing uses x. FD takes 1 input");
    assert_eq!(error("CS 5"), "Nothing uses 5. CS takes no inputs");
    assert_eq!(error("SETXY 1 2 3"), "Nothing uses 3. SETXY takes 2 inputs");
    assert_eq!(
        error("REPEAT 2 [FD 10 20]"),
        "Nothing uses 20. FD takes 1 input"
    );
    assert_eq!(
        error("TO SQ :n\nFD :n 20\nEND\nSQ 1"),
        "Nothing uses 20. FD takes 1 input"
    );
    assert_eq!(
        error("TO SQ :n\nEND\nSQ 1 2"),
        "Nothing uses 2. SQ takes 1 input"
    );
    assert_eq!(error("FD 10 SUM 1 2"), "Nothing uses 3. FD takes 1 input");
}

#[test]
fn a_value_on_its_own_keeps_the_old_wording() {
    assert_eq!(error("20"), "You don't say what to do with 20");
    assert_eq!(error("FD 10\n20"), "You don't say what to do with 20");
    assert_eq!(error("SUM 1 2"), "You don't say what to do with 3");
    assert_eq!(error("REPEAT 2 [5]"), "You don't say what to do with 5");
}

#[test]
fn the_call_it_comes_after_gets_the_help() {
    let run = run("FORWARD 10 20");
    assert_eq!(run.error().command.as_deref(), Some("FORWARD"));
}

#[test]
fn instructions_with_no_brackets_say_so_for_every_command_that_runs_them() {
    for (program, name) in [
        ("REPEAT 4 FD 5", "REPEAT"),
        ("FOR [i 1 3] FD 5", "FOR"),
        ("WHILE \"true FD 5", "WHILE"),
        ("UNTIL \"true FD 5", "UNTIL"),
        ("DO.WHILE FD 5 \"true", "DO.WHILE"),
        ("DO.UNTIL FD 5 \"true", "DO.UNTIL"),
        ("CATCH \"x FD 5", "CATCH"),
        ("RUN FD 5", "RUN"),
        ("IF \"true FD 5", "IF"),
    ] {
        assert_eq!(
            error(program),
            format!("{name} needs a list of instructions here. Put them in [ ]"),
            "{program}"
        );
    }
}

#[test]
fn other_commands_keep_saying_it_did_not_output() {
    assert_eq!(error("PRINT FD 5"), "FD didn't output to PRINT");
    assert_eq!(error("FD FD 5"), "FD didn't output to FD");
    assert_eq!(error("MAKE \"x FD 5"), "FD didn't output to MAKE");
    assert_eq!(error("WHILE FD 5 [RT 1]"), "FD didn't output to WHILE");
    assert_eq!(error("TO SQ :n\nEND\nSQ FD 5"), "FD didn't output to SQ");
}
