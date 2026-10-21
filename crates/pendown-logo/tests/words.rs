//! The rest of words and lists (phase 2 issue 02).

mod common;

use common::*;

#[track_caller]
fn shows(cases: &[(&str, &str)]) {
    for (expr, expected) in cases {
        assert_eq!(output(&format!("SHOW {expr}")), format!("{expected}\n"), "{expr}");
    }
}

#[test]
fn constructors() {
    shows(&[
        ("FPUT 1 [2 3]", "[1 2 3]"),
        ("FPUT [1] []", "[[1]]"),
        ("FPUT \"a \"bc", "abc"),
        ("LPUT 3 [1 2]", "[1 2 3]"),
        ("LPUT \"c \"ab", "abc"),
        ("COMBINE \"a \"bc", "abc"),
        ("COMBINE \"a [b c]", "[a b c]"),
        ("REVERSE [1 2 3]", "[3 2 1]"),
        ("REVERSE \"abc", "cba"),
        ("REVERSE []", "[]"),
    ]);
}

#[test]
fn selectors() {
    shows(&[
        ("FIRSTS [[1 2] [3 4] abc]", "[1 3 a]"),
        ("BUTFIRSTS [[1 2] [3 4]]", "[[2] [4]]"),
        ("BFS []", "[]"),
        ("REMOVE 2 [1 2 3 2]", "[1 3]"),
        ("REMOVE \"l \"hello", "heo"),
        ("REMDUP [a b a c b]", "[a c b]"),
        ("REMDUP \"banana", "bna"),
        ("QUOTED \"abc", "\"abc"),
        ("QUOTED [a b]", "[a b]"),
        ("MEMBERP [a b] [[a b] c]", "true"),
    ]);
    let r = output("REPEAT 20 [TYPE PICK [a b c]]");
    assert_eq!(r.len(), 20);
    assert!(r.chars().all(|c| "abc".contains(c)), "{r}");
}

#[test]
fn stacks_and_queues() {
    assert_eq!(output("MAKE \"s [] PUSH \"s 1 PUSH \"s 2 SHOW :s SHOW POP \"s SHOW :s"), "[2 1]\n2\n[1]\n");
    assert_eq!(output("MAKE \"q [] QUEUE \"q 1 QUEUE \"q 2 SHOW :q SHOW DEQUEUE \"q SHOW :q"), "[1 2]\n1\n[2]\n");
    assert_eq!(error("PUSH \"nothing 1"), "nothing has no value");
    assert_eq!(error("MAKE \"s 5 PUSH \"s 1"), "PUSH doesn't like 5 as the value of s");
    assert_eq!(error("MAKE \"s [] SHOW POP \"s"), "s is empty");
}

#[test]
fn predicates() {
    shows(&[
        ("WORDP \"a", "true"),
        ("WORDP 5", "true"),
        ("WORD? [a]", "false"),
        ("LISTP [a]", "true"),
        ("LIST? \"a", "false"),
        ("NUMBERP 5", "true"),
        ("NUMBERP \"5", "true"),
        ("NUMBER? \"five", "false"),
        ("BEFOREP \"apple \"banana", "true"),
        ("BEFOREP 3 12", "false"),
        ("BEFORE? \"B \"a", "false"),
        ("MEMBERP 2 [1 2 3]", "true"),
        ("MEMBER? 4 [1 2 3]", "false"),
        ("MEMBERP \"e \"hello", "true"),
        ("MEMBERP \"el \"hello", "false"),
        ("SUBSTRINGP \"ell \"hello", "true"),
        ("SUBSTRING? \"x \"hello", "false"),
        ("SUBSTRINGP [a] [a b]", "false"),
        ("MEMBERP \"A [a b]", "true"),
    ]);
}

#[test]
fn queries() {
    shows(&[
        ("MEMBER \"c [a b c d]", "[c d]"),
        ("MEMBER \"x [a b]", "[]"),
        ("MEMBER \"l \"hello", "llo"),
        ("MEMBER \"x \"hello", ""),
        ("ASCII \"A", "65"),
        ("ASCII 5", "53"),
        ("CHAR 97", "a"),
        ("LOWERCASE \"HeLLo", "hello"),
        ("UPPERCASE \"hello", "HELLO"),
    ]);
}

#[test]
fn errors() {
    assert_eq!(error("SHOW FPUT \"ab \"cd"), "FPUT doesn't like ab as input");
    assert_eq!(error("SHOW FIRSTS [[1] []]"), "FIRSTS doesn't like [[1] []] as input");
    assert_eq!(error("SHOW PICK []"), "PICK doesn't like [] as input");
    assert_eq!(error("SHOW ASCII \"ab"), "ASCII doesn't like ab as input");
    assert_eq!(error("SHOW CHAR -1"), "CHAR doesn't like -1 as input");
    assert_eq!(error("SHOW MEMBER 1 {1 2}"), "MEMBER doesn't like {1 2} as input");
}
