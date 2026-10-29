//! Arithmetic and logic (phase 1 issue 10).

mod common;

use std::sync::Arc;

use common::*;
use pendown_logo::{Event, Interpreter};

#[test]
fn the_basics() {
    let cases = [
        ("SUM 2 3", "5"),
        ("(SUM 1 2 3 4)", "10"),
        ("DIFFERENCE 10 4", "6"),
        ("PRODUCT 6 7", "42"),
        ("(PRODUCT 2 3 4)", "24"),
        ("QUOTIENT 7 2", "3.5"),
        ("QUOTIENT 4 2", "2"),
        ("(QUOTIENT 4)", "0.25"),
        ("MINUS 5", "-5"),
        ("INT 3.7", "3"),
        ("INT -3.7", "-3"),
        ("ROUND 3.5", "4"),
        ("ROUND -3.5", "-4"),
        ("ROUND 2.4", "2"),
        ("SQRT 16", "4"),
        ("SQRT 2", "1.4142135623731"),
        ("POWER 2 10", "1024"),
        ("POWER 4 0.5", "2"),
        ("POWER -2 3", "-8"),
    ];
    for (expr, expected) in cases {
        assert_eq!(
            output(&format!("PRINT {expr}")),
            format!("{expected}\n"),
            "{expr}"
        );
    }
}

#[test]
fn remainder_and_modulo_signs() {
    // REMAINDER takes the sign of the first input, MODULO of the second.
    let cases = [
        ("REMAINDER 7 3", "1"),
        ("REMAINDER -7 3", "-1"),
        ("REMAINDER 7 -3", "1"),
        ("MODULO 7 3", "1"),
        ("MODULO -7 3", "2"),
        ("MODULO 7 -3", "-2"),
        ("MODULO -7 -3", "-1"),
    ];
    for (expr, expected) in cases {
        assert_eq!(
            output(&format!("PRINT {expr}")),
            format!("{expected}\n"),
            "{expr}"
        );
    }
}

#[test]
fn trig_in_degrees() {
    let cases = [
        ("SIN 30", "0.5"),
        ("SIN 90", "1"),
        ("COS 90", "0"),
        ("COS 180", "-1"),
        ("COS 60", "0.5"),
        ("SIN 45", "0.707106781186547"),
        ("ARCTAN 1", "45"),
        ("(ARCTAN 1 1)", "45"),
        ("(ARCTAN 0 1)", "90"),
        ("(ARCTAN 0 -1)", "-90"),
        ("(ARCTAN -1 0)", "180"),
    ];
    for (expr, expected) in cases {
        assert_eq!(
            output(&format!("PRINT {expr}")),
            format!("{expected}\n"),
            "{expr}"
        );
    }
}

#[test]
fn comparisons_and_logic() {
    let cases = [
        ("LESSP 1 2", "true"),
        ("GREATERP 1 2", "false"),
        ("LESSEQUALP 2 2", "true"),
        ("GREATEREQUALP 1 2", "false"),
        ("1 <= 1", "true"),
        ("2 >= 3", "false"),
        ("AND \"true \"true", "true"),
        ("AND \"true \"false", "false"),
        ("(AND \"true \"true \"false)", "false"),
        ("OR \"false \"false", "false"),
        ("(OR \"false \"false \"TRUE)", "true"),
        ("NOT \"false", "true"),
        ("NOT 1 = 2", "true"),
        ("AND 1 < 2 2 < 3", "true"),
    ];
    for (expr, expected) in cases {
        assert_eq!(
            output(&format!("PRINT {expr}")),
            format!("{expected}\n"),
            "{expr}"
        );
    }
}

#[test]
fn logic_runs_list_inputs_only_when_needed() {
    // The second list would divide by zero, but AND stops at the first FALSE.
    assert_eq!(output("PRINT AND [1 > 2] [1 / 0 = 1]"), "false\n");
    assert_eq!(output("PRINT OR [1 < 2] [1 / 0 = 1]"), "true\n");
    assert_eq!(output("PRINT NOT [3 = 3]"), "false\n");
    assert_eq!(
        error("PRINT AND [1 < 2] [1 / 0 = 1]"),
        "Can't divide by zero"
    );
}

#[test]
fn errors() {
    assert_eq!(
        error("PRINT REMAINDER 7.5 2"),
        "REMAINDER doesn't like 7.5 as input"
    );
    assert_eq!(error("PRINT MODULO 7 0"), "Can't divide by zero");
    assert_eq!(error("PRINT SQRT -1"), "SQRT doesn't like -1 as input");
    assert_eq!(
        error("PRINT POWER -8 0.5"),
        "POWER doesn't like 0.5 as input"
    );
    assert_eq!(error("PRINT POWER 10 400"), "That number is too big");
    assert_eq!(
        error("PRINT AND \"yes \"true"),
        "AND doesn't like yes as input"
    );
    assert_eq!(error("PRINT NOT 5"), "NOT doesn't like 5 as input");
    assert_eq!(error("PRINT RANDOM 0"), "RANDOM doesn't like 0 as input");
    assert_eq!(
        error("PRINT RANDOM 2.5"),
        "RANDOM doesn't like 2.5 as input"
    );
    assert_eq!(
        error("PRINT (RANDOM 5 1)"),
        "RANDOM doesn't like 1 as input"
    );
}

fn printed(it: &mut Interpreter, events: &Shared, src: &str) -> String {
    events.0.borrow_mut().clear();
    it.run_program(src).unwrap();
    events
        .0
        .borrow()
        .iter()
        .filter_map(|e| {
            if let Event::Print(t) = e {
                Some(t.clone())
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn random() {
    let events = Shared::default();
    let mut it = Interpreter::new(Box::new(events.clone()), Arc::default());
    it.set_seed(Some(1));
    let program = "REPEAT 20 [PRINT RANDOM 10]";
    let first = printed(&mut it, &events, program);
    assert_eq!(
        printed(&mut it, &events, program),
        first,
        "the same seed draws the same picture"
    );
    let numbers: Vec<i64> = first
        .split_whitespace()
        .map(|n| n.parse().unwrap())
        .collect();
    assert!(numbers.iter().all(|n| (0..10).contains(n)), "{numbers:?}");
    let ranged = printed(&mut it, &events, "REPEAT 50 [PRINT (RANDOM 3 5)]");
    let ranged: Vec<i64> = ranged
        .split_whitespace()
        .map(|n| n.parse().unwrap())
        .collect();
    assert!(
        ranged.iter().all(|n| (3..=5).contains(n)) && ranged.contains(&3) && ranged.contains(&5),
        "{ranged:?}"
    );
    // Without a seed, each Run is different.
    it.set_seed(None);
    let runs: Vec<_> = (0..3).map(|_| printed(&mut it, &events, program)).collect();
    assert!(runs[0] != runs[1] || runs[1] != runs[2]);
}

#[test]
fn phase_2_arithmetic() {
    let cases = [
        ("EXP 0", "1"),
        ("LN EXP 2", "2"),
        ("LOG10 1000", "3"),
        ("RADSIN 0", "0"),
        ("RADCOS 0", "1"),
        ("RADARCTAN 1", "0.785398163397448"),
        ("(RADARCTAN 0 1)", "1.5707963267949"),
        ("ISEQ 3 7", "[3 4 5 6 7]"),
        ("ISEQ 7 3", "[7 6 5 4 3]"),
        ("RSEQ 3 5 9", "[3 3.25 3.5 3.75 4 4.25 4.5 4.75 5]"),
        ("RSEQ 3 5 5", "[3 3.5 4 4.5 5]"),
        ("RSEQ 1 2 1", "[1]"),
        ("FORM 3.14159 8 2", "    3.14"),
        ("FORM 2 0 0", "2"),
        ("FORM -1.5 1 3", "-1.500"),
    ];
    for (expr, expected) in cases {
        assert_eq!(
            output(&format!("SHOW {expr}")),
            format!("{expected}\n"),
            "{expr}"
        );
    }
}

#[test]
fn rerandom_repeats() {
    let a = output("RERANDOM REPEAT 10 [TYPE RANDOM 100 TYPE \" ]");
    let b = output("RERANDOM REPEAT 10 [TYPE RANDOM 100 TYPE \" ]");
    assert_eq!(a, b);
    let c = output("(RERANDOM 7) REPEAT 10 [TYPE RANDOM 100 TYPE \" ]");
    assert_ne!(a, c);
}

#[test]
fn phase_2_arithmetic_errors() {
    assert_eq!(error("SHOW LN 0"), "LN doesn't like 0 as input");
    assert_eq!(error("SHOW LOG10 -1"), "LOG10 doesn't like -1 as input");
    assert_eq!(error("SHOW EXP 1000"), "That number is too big");
    assert_eq!(error("SHOW ISEQ 1 1.5"), "ISEQ doesn't like 1.5 as input");
    assert_eq!(
        error("SHOW ISEQ 1 1e12"),
        "ISEQ would make more than 10000000 members"
    );
    assert_eq!(error("SHOW RSEQ 1 2 0"), "RSEQ doesn't like 0 as input");
    assert_eq!(
        error("SHOW FORM 1 -1 \"%x"),
        "FORM's printf formats aren't supported: give a width of 0 or more"
    );
    assert_eq!(
        error("SHOW ARRAY 1e9"),
        "ARRAY would make more than 10000000 members"
    );
}
