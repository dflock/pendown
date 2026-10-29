//! Templates and the higher-order commands (phase 2 issue 06).

mod common;

use common::*;

#[track_caller]
fn shows(cases: &[(&str, &str)]) {
    for (expr, expected) in cases {
        assert_eq!(
            output(&format!("SHOW {expr}")),
            format!("{expected}\n"),
            "{expr}"
        );
    }
}

#[test]
fn the_four_forms() {
    shows(&[
        ("MAP \"FIRST [abc def]", "[a d]"),
        ("MAP [? * 2] [1 2 3]", "[2 4 6]"),
        ("MAP [[x] :x * 10] [1 2]", "[10 20]"),
        ("MAP [[x] [OUTPUT :x + 1]] [1 2]", "[2 3]"),
        ("(MAP [?1 + ?2] [1 2] [10 20])", "[11 22]"),
        ("(MAP [(? 1) * (? 2)] [2 3] [4 5])", "[8 15]"),
        ("(MAP \"SUM [1 2] [3 4])", "[4 6]"),
        ("(MAP [[a b] :a - :b] [5 5] [1 2])", "[4 3]"),
    ]);
    // A user procedure by name.
    assert_eq!(
        output("TO DOUBLE :x\nOUTPUT :x * 2\nEND\nSHOW MAP \"DOUBLE [1 2]"),
        "[2 4]\n"
    );
}

#[test]
fn apply_and_invoke() {
    shows(&[
        ("APPLY \"SUM [1 2 3]", "6"),
        ("APPLY [? + 1] [5]", "6"),
        ("APPLY [[a b] LIST :b :a] [1 2]", "[2 1]"),
        ("INVOKE \"WORD \"a", "a"),
        ("(INVOKE \"WORD \"a \"b \"c)", "abc"),
        ("(INVOKE [?1 * ?2] 3 4)", "12"),
    ]);
    assert_eq!(output("APPLY \"PRINT [hello world]"), "hello world\n");
    assert_eq!(output("APPLY \"REPEAT [2 [PRINT \"hi]]"), "hi\nhi\n");
}

#[test]
fn foreach() {
    assert_eq!(output("FOREACH [a b c] [PRINT ?]"), "a\nb\nc\n");
    assert_eq!(output("FOREACH \"abc [TYPE ?] PRINT \""), "abc\n");
    assert_eq!(output("FOREACH [a b c] [PRINT #]"), "1\n2\n3\n");
    assert_eq!(output("FOREACH [a b c] [SHOW ?REST]"), "[b c]\n[c]\n[]\n");
    assert_eq!(output("FOREACH \"abc [SHOW ?REST]"), "bc\nc\n\n");
    assert_eq!(
        output("(FOREACH [1 2] [a b] [PRINT WORD ?1 ?2])"),
        "1a\n2b\n"
    );
    assert_eq!(output("FOREACH [1 2] [[n] PRINT :n * 3]"), "3\n6\n");
    // STOP inside FOREACH leaves the procedure.
    assert_eq!(
        output("TO F\nFOREACH [1 2 3] [IF ? = 2 [STOP] PRINT ?]\nEND\nF"),
        "1\n"
    );
}

#[test]
fn map_se_filter_find_reduce() {
    shows(&[
        ("MAP [UPPERCASE ?] \"abc", "ABC"),
        ("MAP.SE [LIST ? ?] [a b]", "[a a b b]"),
        ("MAP.SE [[x] IFELSE :x > 1 [[]] [:x]] [1 2 3]", "[1]"),
        ("FILTER [? > 2] [1 2 3 4]", "[3 4]"),
        ("FILTER [MEMBERP ? \"aeiou] \"elephant", "eea"),
        ("FIND [? > 2] [1 2 3 4]", "3"),
        ("FIND [? > 9] [1 2]", "[]"),
        ("REDUCE \"SUM [1 2 3 4]", "10"),
        ("REDUCE \"DIFFERENCE [10 3 2]", "9"),
        ("REDUCE [IFELSE ?1 > ?2 [?1] [?2]] [3 9 2]", "9"),
        ("REDUCE \"WORD \"abc", "abc"),
        ("REDUCE \"SUM [5]", "5"),
        ("MAP [#] [a b c]", "[1 2 3]"),
    ]);
}

#[test]
fn template_errors() {
    assert_eq!(error("SHOW MAP \"NOSUCH [1]"), "I don't know how to NOSUCH");
    assert_eq!(
        error("SHOW MAP [PRINT ?] [1]"),
        "[PRINT ?] didn't output to MAP"
    );
    assert_eq!(
        error("SHOW (MAP \"SUM [1 2] [3])"),
        "MAP's data inputs must be the same length"
    );
    assert_eq!(
        error("SHOW FILTER [?] [1]"),
        "FILTER doesn't like 1 from the template"
    );
    assert_eq!(
        error("SHOW APPLY \"FORWARD [1 2]"),
        "Too many inputs to FORWARD"
    );
    assert_eq!(error("SHOW APPLY \"SUM 5"), "APPLY doesn't like 5 as input");
    assert_eq!(
        error("SHOW APPLY [[a b] :a] [1]"),
        "Not enough inputs to THE TEMPLATE"
    );
    assert_eq!(
        error("SHOW REDUCE \"SUM []"),
        "REDUCE doesn't like [] as input"
    );
    assert_eq!(error("PRINT ?"), "? can only be used inside a template");
    assert_eq!(error("SHOW MAP [?2] [1]"), "This template has no ?2");
    assert_eq!(error("FOREACH [1] [?]"), "You don't say what to do with 1");
}
