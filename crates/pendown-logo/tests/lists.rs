//! Words, lists and printing (phase 1 issue 11).

mod common;

use common::*;

#[test]
fn building_lists() {
    let cases = [
        ("LIST 1 2", "[1 2]"),
        ("(LIST 1 2 3)", "[1 2 3]"),
        ("(LIST)", "[]"),
        ("LIST [1 2] 3", "[[1 2] 3]"),
        ("SENTENCE [1 2] 3", "[1 2 3]"),
        ("SE [a b] [c d]", "[a b c d]"),
        ("(SENTENCE \"a [b [c]] \"d)", "[a b [c] d]"),
        ("WORD \"ab \"cd", "abcd"),
    ];
    for (expr, expected) in cases {
        assert_eq!(output(&format!("SHOW {expr}")), format!("{expected}\n"), "{expr}");
    }
}

#[test]
fn selectors_on_lists() {
    let cases = [
        ("FIRST [a b c]", "a"),
        ("LAST [a b c]", "c"),
        ("BUTFIRST [a b c]", "[b c]"),
        ("BF [a]", "[]"),
        ("BUTLAST [a b c]", "[a b]"),
        ("BL [a b c]", "[a b]"),
        ("ITEM 2 [a b c]", "b"),
        ("FIRST [[1 2] 3]", "[1 2]"),
        ("COUNT [a b c]", "3"),
        ("COUNT []", "0"),
        ("EMPTYP []", "true"),
        ("EMPTY? [a]", "false"),
    ];
    for (expr, expected) in cases {
        assert_eq!(output(&format!("SHOW {expr}")), format!("{expected}\n"), "{expr}");
    }
}

#[test]
fn selectors_on_words_and_numbers() {
    // Numbers are words (ADR 0002).
    let cases = [
        ("FIRST \"hello", "h"),
        ("LAST \"hello", "o"),
        ("BF \"hello", "ello"),
        ("BL \"hello", "hell"),
        ("ITEM 2 \"hello", "e"),
        ("COUNT \"hello", "5"),
        ("FIRST 345", "3"),
        ("COUNT 345", "3"),
        ("BF 345", "45"),
        ("(BF 345) + 1", "46"),
        ("COUNT 2.5", "3"),
        ("EMPTYP \"", "true"),
        ("EMPTYP BF \"a", "true"),
        ("COUNT \"héllo", "5"),
    ];
    for (expr, expected) in cases {
        assert_eq!(output(&format!("SHOW {expr}")), format!("{expected}\n"), "{expr}");
    }
}

#[test]
fn printing() {
    assert_eq!(output("PRINT [a [b c] d]"), "a [b c] d\n", "PRINT drops the outer brackets");
    assert_eq!(output("SHOW [a [b c] d]"), "[a [b c] d]\n", "SHOW keeps them");
    assert_eq!(output("(PRINT 1 [2 3] \"four)"), "1 2 3 four\n");
    assert_eq!(output("(SHOW 1 [2 3])"), "1 [2 3]\n");
    assert_eq!(output("TYPE \"a (TYPE \"b [c d]) PRINT \"e"), "abc de\n", "TYPE adds no spaces or newline");
    assert_eq!(output("(PRINT)"), "\n");
    assert_eq!(output("PRINT []"), "\n");
}

#[test]
fn positions_are_lists() {
    assert_eq!(output("SETPOS LIST 30 40 SHOW POS PRINT FIRST POS PRINT LAST POS"), "[30 40]\n30\n40\n");
    assert_eq!(output("SETPOS SE 10 20 SHOW POS"), "[10 20]\n");
    assert_eq!(output("MAKE \"p POS FD 50 SETPOS :p SHOW POS"), "[0 0]\n");
}

#[test]
fn errors() {
    assert_eq!(error("PRINT FIRST []"), "FIRST doesn't like [] as input");
    assert_eq!(error("PRINT LAST \""), "LAST doesn't like the empty word as input");
    assert_eq!(error("PRINT BF []"), "BF doesn't like [] as input", "named as it was typed");
    assert_eq!(error("PRINT ITEM 4 [a b c]"), "ITEM doesn't like 4 as input");
    assert_eq!(error("PRINT ITEM 0 \"abc"), "ITEM doesn't like 0 as input");
    assert_eq!(error("PRINT ITEM 1.5 [a b]"), "ITEM doesn't like 1.5 as input");
}
