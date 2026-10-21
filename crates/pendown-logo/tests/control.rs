//! RUN, the loops, CASE and COND (phase 2 issue 04).

mod common;

use common::*;

#[test]
fn run_and_runresult() {
    assert_eq!(output("RUN [PRINT 1 PRINT 2]"), "1\n2\n");
    assert_eq!(output("SHOW RUN [3 + 4]"), "7\n");
    assert_eq!(output("RUN \"HOME SHOW POS"), "[0 0]\n");
    assert_eq!(output("RUN (LIST \"PRINT 5)"), "5\n");
    assert_eq!(output("SHOW RUNRESULT [5]"), "[5]\n");
    assert_eq!(output("SHOW RUNRESULT [HOME]"), "[]\n");
    assert_eq!(output("IGNORE 5 PRINT \"ok"), "ok\n");
    assert_eq!(output("TO F\nOUTPUT RUN [IFELSE 1 < 2 [\"yes] [\"no]]\nEND\nPRINT F"), "yes\n");
    // OUTPUT and STOP inside RUN leave the procedure.
    assert_eq!(output("TO F\nRUN [OUTPUT 3]\nPRINT \"never\nEND\nPRINT F"), "3\n");
    assert_eq!(error("RUN [3 + 4]"), "You don't say what to do with 7");
    assert_eq!(error("RUN [FOO]"), "I don't know how to FOO");
}

#[test]
fn for_loops() {
    assert_eq!(output("FOR [i 1 3] [PRINT :i]"), "1\n2\n3\n");
    assert_eq!(output("FOR [i 3 1] [PRINT :i]"), "3\n2\n1\n");
    assert_eq!(output("FOR [i 2 7 1.5] [PRINT :i]"), "2\n3.5\n5\n6.5\n");
    assert_eq!(output("FOR [i 1 0 1] [PRINT :i] PRINT \"done"), "done\n");
    assert_eq!(output("MAKE \"n 3 FOR [i 1 :n*2 2] [TYPE :i] PRINT \""), "135\n");
    assert_eq!(output("FOR [i 0 1 0.25] [TYPE WORD :i \",] PRINT \""), "0,0.25,0.5,0.75,1,\n");
    // The variable is local to the FOR.
    assert_eq!(output("MAKE \"i 99 FOR [i 1 2] [] PRINT :i"), "99\n");
    // Changing the variable changes the next value.
    assert_eq!(output("FOR [i 1 10] [PRINT :i MAKE \"i :i + 4]"), "1\n6\n");
    assert_eq!(output("TO F\nFOR [i 1 10] [IF :i = 3 [OUTPUT :i]]\nEND\nPRINT F"), "3\n");
    assert_eq!(error("FOR [i 1] [PRINT :i]"), "FOR doesn't like [i 1] as input");
    assert_eq!(error("FOR [i 1 \"x] [PRINT :i]"), "FOR doesn't like [i 1 \"x] as input");
}

#[test]
fn while_and_until() {
    assert_eq!(output("MAKE \"x 0 WHILE [:x < 3] [MAKE \"x :x + 1 PRINT :x]"), "1\n2\n3\n");
    assert_eq!(output("MAKE \"x 5 WHILE [:x < 3] [PRINT :x] PRINT \"none"), "none\n");
    assert_eq!(output("MAKE \"x 5 DO.WHILE [PRINT :x] [:x < 3]"), "5\n");
    assert_eq!(output("MAKE \"x 0 UNTIL [:x = 2] [MAKE \"x :x + 1 PRINT :x]"), "1\n2\n");
    assert_eq!(output("MAKE \"x 9 DO.UNTIL [PRINT :x] [\"true]"), "9\n");
    assert_eq!(error("WHILE [5] [PRINT 1]"), "WHILE doesn't like 5 as a condition");
    assert_eq!(error("WHILE [HOME] [PRINT 1]"), "[HOME] didn't output TRUE or FALSE to WHILE");
}

#[test]
fn case_and_cond() {
    let vowel = "TO VOWELP :letter\nOUTPUT CASE :letter [[[a e i o u] \"true] [ELSE \"false]]\nEND\n";
    assert_eq!(output(&format!("{vowel}SHOW VOWELP \"e SHOW VOWELP \"x")), "true\nfalse\n");
    assert_eq!(output("CASE 2 [[[1] PRINT \"one] [[2 3] PRINT \"two]]"), "two\n");
    assert_eq!(output("CASE 9 [[[1] PRINT \"one]] PRINT \"none"), "none\n");
    assert_eq!(output("MAKE \"n 5 COND [[[:n < 0] PRINT \"neg] [[:n > 0] PRINT \"pos] [ELSE PRINT \"zero]]"), "pos\n");
    assert_eq!(output("SHOW COND [[[1 > 2] 1] [else 2]]"), "2\n");
    assert_eq!(error("CASE 1 [x]"), "CASE doesn't like [x] as input");
    assert_eq!(error("COND [[[5] 1]]"), "COND doesn't like 5 as a condition");
}

#[test]
fn empty_loops_can_still_be_stopped() {
    use std::sync::Arc;
    use pendown_logo::{Interpreter, Limits, RunError};
    for src in ["FOREVER []", "REPEAT 1e15 []", "FOR [i 1 1e15] []", "WHILE [\"true] []"] {
        let mut it = Interpreter::new(Box::new(Vec::new()), Arc::default());
        it.limits = Limits { instruction_events: false, max_instructions: Some(1000), ..Limits::default() };
        match it.run_program(src) {
            Err(RunError::Runtime(e)) => assert!(e.message.starts_with("Stopped after 1000 instructions"), "{src}: {e:?}"),
            other => panic!("{src}: {other:?}"),
        }
    }
}
