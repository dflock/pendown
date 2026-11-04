//! Arrays (phase 2 issue 01).

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
fn typing_arrays_in() {
    shows(&[
        ("{a b c}", "{a b c}"),
        ("{a b c}@0", "{a b c}@0"),
        ("{}", "{}"),
        ("{1 [2 3] {4}}", "{1 [2 3] {4}}"),
        ("[a {b c}@-1]", "[a {b c}@-1]"),
        ("COUNT {a b c}", "3"),
        ("ITEM 2 {a b c}", "b"),
        ("ITEM 0 {a b c}@0", "a"),
        ("FIRST {a b c}@5", "5"),
        ("EMPTYP {}", "true"),
        ("ARRAYP {a}", "true"),
        ("ARRAY? [a]", "false"),
    ]);
    // PRINT keeps an array's braces.
    assert_eq!(output("PRINT {a b}"), "{a b}\n");
}

#[test]
fn making_and_changing_arrays() {
    shows(&[
        ("ARRAY 3", "{[] [] []}"),
        ("(ARRAY 2 0)", "{[] []}@0"),
        ("MDARRAY [2 3]", "{{[] [] []} {[] [] []}}"),
        ("(MDARRAY [2 2] 0)", "{{[] []}@0 {[] []}@0}@0"),
        ("LISTTOARRAY [a b]", "{a b}"),
        ("(LISTTOARRAY [a b] 0)", "{a b}@0"),
        ("ARRAYTOLIST {a b}@0", "[a b]"),
    ]);
    assert_eq!(
        output("MAKE \"a ARRAY 3 SETITEM 2 :a \"x SHOW :a"),
        "{[] x []}\n"
    );
    assert_eq!(
        output("MAKE \"m MDARRAY [2 2] MDSETITEM [2 1] :m 5 SHOW :m SHOW MDITEM [2 1] :m"),
        "{{[] []} {5 []}}\n5\n"
    );
}

#[test]
fn arrays_are_shared() {
    // SETITEM changes the array wherever it's used.
    assert_eq!(
        output("MAKE \"a {1 2} MAKE \"b :a SETITEM 1 :b 9 SHOW :a"),
        "{9 2}\n"
    );
    // Two arrays are equal only if they're the same array.
    shows(&[("{1 2} = {1 2}", "false"), ("MEMBERP 2 {1 2}", "true")]);
    assert_eq!(output("MAKE \"a {1} SHOW :a = :a"), "true\n");
}

#[test]
fn array_errors() {
    assert_eq!(error("SHOW ITEM 4 {a b c}"), "ITEM doesn't like 4 as input");
    assert_eq!(
        error("SHOW ITEM 3 {a b c}@0"),
        "ITEM doesn't like 3 as input"
    );
    assert_eq!(error("SHOW ARRAY 0"), "ARRAY doesn't like 0 as input");
    assert_eq!(
        error("SHOW BUTFIRST {a b}"),
        "BUTFIRST doesn't like {a b} as input"
    );
    assert_eq!(
        error("SETITEM 1 [a b] 3"),
        "SETITEM doesn't like [a b] as input"
    );
    assert_eq!(
        error("MAKE \"a {1 2} SETITEM 1 :a :a"),
        "SETITEM can't put an array inside itself"
    );
    assert_eq!(
        error("MAKE \"a {1 2} SETITEM 1 :a (LIST 3 :a)"),
        "SETITEM can't put an array inside itself"
    );
    assert_eq!(
        error("SHOW MDITEM [1 5] MDARRAY [2 2]"),
        "MDITEM doesn't like [1 5] as input"
    );
    assert_eq!(error("SHOW MDARRAY []"), "MDARRAY doesn't like [] as input");
}

#[test]
fn reading_errors() {
    let parse_error = |src: &str| match run(src).result {
        Err(pendown_logo::RunError::Parse(errors)) => errors[0].message.clone(),
        other => panic!("expected a parse error, got {other:?}"),
    };
    assert_eq!(parse_error("SHOW {a b"), "This { has no matching }");
    assert_eq!(parse_error("SHOW a}"), "This } has no matching {");
    assert_eq!(parse_error("SHOW [a}"), "This } has no matching {");
    assert_eq!(parse_error("SHOW {a]"), "This ] has no matching [");
    assert_eq!(
        parse_error("SHOW {a}@x"),
        "An array's origin must be a whole number, as in {a b c}@0"
    );
}
