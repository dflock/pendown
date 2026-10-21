//! Special variables, NAME and GLOBAL (phase 2 issue 03).

mod common;

use common::*;

#[test]
fn caseignoredp() {
    assert_eq!(output("SHOW :CASEIGNOREDP SHOW \"a = \"A"), "true\ntrue\n");
    assert_eq!(output("MAKE \"caseignoredp \"false SHOW \"a = \"A SHOW MEMBERP \"A [a] SHOW BEFOREP \"B \"a"), "false\nfalse\ntrue\n");
    // Run starts it afresh.
    assert_eq!(output("SHOW \"a = \"A"), "true\n");
}

#[test]
fn print_limits() {
    assert_eq!(output("MAKE \"printdepthlimit 1 SHOW [1 [2 [3]]] PRINT [1 [2]]"), "[1 ...]\n1 ...\n");
    assert_eq!(output("MAKE \"printwidthlimit 2 SHOW [1 2 3 4] SHOW {a b c}"), "[1 2 ...]\n{a b ...}\n");
    assert_eq!(output("MAKE \"printwidthlimit \"none SHOW [1 2 3]"), "[1 2 3]\n");
}

#[test]
fn fullprintp() {
    assert_eq!(output("SHOW \"|| SHOW \""), "||\n\n");
    assert_eq!(output("MAKE \"fullprintp \"true SHOW \" SHOW (LIST \"a \")"), "||\n[a ||]\n");
}

#[test]
fn logoversion() {
    let version = output("SHOW :LOGOVERSION");
    assert!(version.trim().parse::<f64>().is_ok(), "{version}");
}

#[test]
fn name_and_global() {
    assert_eq!(output("NAME 5 \"x SHOW :x"), "5\n");
    assert_eq!(output("GLOBAL \"x (GLOBAL \"y \"z) GLOBAL [a b] MAKE \"x 1 SHOW :x"), "1\n");
    assert_eq!(error("GLOBAL \"x SHOW :x"), ":x has no value");
}
