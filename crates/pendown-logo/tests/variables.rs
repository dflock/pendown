//! Procedures, variables and control (phase 1 issue 09). Spike 03's tests
//! cover tail calls, the depth limit and parsing; these cover the rest.

mod common;

use common::*;

#[test]
fn local_hides_a_global_until_the_procedure_ends() {
    let src = "MAKE \"x 1\nTO F\n LOCAL \"x\n MAKE \"x 2\n PRINT :x\nEND\nF\nPRINT :x";
    assert_eq!(output(src), "2\n1\n");
}

#[test]
fn a_local_has_no_value_until_made() {
    let r = run("TO F\n LOCAL \"x\n PRINT :x\nEND\nMAKE \"x 5\nF");
    assert_eq!(r.error().message, ":x has no value");
}

#[test]
fn local_takes_words_and_lists() {
    let src = "MAKE \"a 1 MAKE \"b 2 MAKE \"c 3
TO F
 (LOCAL \"a \"b)
 LOCAL [c]
 MAKE \"a 10 MAKE \"b 20 MAKE \"c 30
 PRINT (SUM :a :b :c)
END
F
PRINT (SUM :a :b :c)";
    assert_eq!(output(src), "60\n6\n");
}

#[test]
fn localmake() {
    let src = "MAKE \"x 1\nTO F\n LOCALMAKE \"x 7\n PRINT :x\nEND\nF\nPRINT :x";
    assert_eq!(output(src), "7\n1\n");
}

#[test]
fn dynamic_scope_through_several_levels() {
    // C sees B's local, and B's local hides A's.
    let src = "TO A\n LOCALMAKE \"v \"a\n B\n PRINT :v\nEND
TO B\n LOCALMAKE \"v \"b\n C\nEND
TO C\n PRINT :v\nEND
A";
    assert_eq!(output(src), "b\na\n");
    let r = run("TO A\n LOCALMAKE \"v 5\nEND\nA\nPRINT :v");
    assert_eq!(r.error().message, ":v has no value", "gone once A ends");
}

#[test]
fn locals_in_tail_calls_share_the_frame() {
    // COUNT calls itself in tail position, so its locals go in one frame;
    // they still disappear when it ends.
    let src = "TO COUNTDOWN :n\n LOCALMAKE \"last :n\n IF :n = 0 [STOP]\n COUNTDOWN :n - 1\nEND\nCOUNTDOWN 100000\nPRINT \"done";
    assert_eq!(output(src), "done\n");
    let r = run(
        "TO COUNTDOWN :n\n LOCALMAKE \"last :n\n IF :n = 0 [STOP]\n COUNTDOWN :n - 1\nEND\nCOUNTDOWN 3\nPRINT :last",
    );
    assert_eq!(r.error().message, ":last has no value");
}

#[test]
fn make_inside_a_procedure_sets_the_nearest_variable_or_a_global() {
    let src = "TO F\n MAKE \"g 3\nEND\nF\nPRINT :g";
    assert_eq!(output(src), "3\n");
    let src = "TO F :n\n G\n PRINT :n\nEND\nTO G\n MAKE \"n 99\nEND\nMAKE \"n 1\nF 5\nPRINT :n";
    assert_eq!(
        output(src),
        "99\n1\n",
        "G's MAKE sets F's input, not the global"
    );
}

#[test]
fn thing() {
    assert_eq!(
        output("MAKE \"x1 42 PRINT THING WORD \"x 1 PRINT THING \"X1"),
        "42\n42\n"
    );
    assert_eq!(error("PRINT THING \"nothing"), "nothing has no value");
}

#[test]
fn repcount_in_nested_repeats() {
    assert_eq!(
        output("REPEAT 2 [REPEAT 3 [TYPE REPCOUNT] PRINT REPCOUNT]"),
        "1231\n1232\n"
    );
    assert_eq!(
        output("REPEAT 3 [TYPE #] PRINT \""),
        "123\n",
        "# is REPCOUNT"
    );
    assert_eq!(
        error("PRINT REPCOUNT"),
        "REPCOUNT can only be used inside REPEAT or FOREVER"
    );
}

#[test]
fn local_outside_a_procedure_is_an_error() {
    assert_eq!(
        error("LOCAL \"x"),
        "LOCAL can only be used inside a procedure"
    );
    assert_eq!(
        error("LOCALMAKE \"x 1"),
        "LOCALMAKE can only be used inside a procedure"
    );
}

#[test]
fn control() {
    assert_eq!(
        output("IF 1 < 2 [PRINT \"yes] IF 1 > 2 [PRINT \"no]"),
        "yes\n"
    );
    assert_eq!(
        output("IFELSE \"true [PRINT 1] [PRINT 2] PRINT IFELSE \"false [1] [2]"),
        "1\n2\n"
    );
    assert_eq!(
        error("IF \"maybe [PRINT 1]"),
        "IF doesn't like maybe as input"
    );
    assert_eq!(
        error("REPEAT -1 [PRINT 1]"),
        "REPEAT doesn't like -1 as input"
    );
    assert_eq!(output("REPEAT 0 [PRINT 1] PRINT \"none"), "none\n");
    assert_eq!(
        output("TO F :n\n IF :n > 2 [OUTPUT \"big]\n OUTPUT \"small\nEND\nPRINT F 1 PRINT F 3"),
        "small\nbig\n"
    );
}
