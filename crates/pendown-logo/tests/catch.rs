//! CATCH, THROW, ERROR and ERRACT (phase 2 issue 05).

mod common;

use common::*;

#[test]
fn catch_and_throw() {
    assert_eq!(output("CATCH \"done [PRINT 1 THROW \"done PRINT 2] PRINT 3"), "1\n3\n");
    assert_eq!(output("SHOW CATCH \"x [(THROW \"X 42)]"), "42\n");
    assert_eq!(output("SHOW CATCH \"x [5]"), "5\n");
    // From inside a procedure.
    let src = "TO F :n\nIF :n = 3 [(THROW \"found :n)]\nF :n + 1\nEND\nSHOW CATCH \"found [F 0]";
    assert_eq!(output(src), "3\n");
    assert_eq!(error("THROW \"nowhere"), "Can't find a CATCH for NOWHERE");
    assert_eq!(error("CATCH \"a [THROW \"b]"), "Can't find a CATCH for B");
    // THROW "TOPLEVEL ends the program without an error.
    assert_eq!(output("PRINT 1 THROW \"toplevel PRINT 2"), "1\n");
}

#[test]
fn catching_errors() {
    assert_eq!(output("CATCH \"error [FD \"x] PRINT \"after"), "after\n");
    assert_eq!(output("CATCH \"error [FD \"x] SHOW ERROR SHOW ERROR"), "[0 FD doesn't like x as input [] 1]\n[]\n");
    let src = "TO F\nFD \"x\nEND\nCATCH \"error [F] SHOW ERROR";
    assert_eq!(output(src), "[0 FD doesn't like x as input F 2]\n");
    // A user error from THROW "ERROR.
    assert_eq!(output("CATCH \"error [(THROW \"error [bad thing])] SHOW ERROR"), "[0 bad thing [] 1]\n");
    assert_eq!(output("SHOW ERROR"), "[]\n");
}

#[test]
fn throw_error_points_at_the_call() {
    let src = "TO CHECK :n\nIF :n < 0 [(THROW \"error [CHECK needs a positive number])]\nEND\nCHECK -1";
    let r = run(src);
    let e = r.error();
    assert_eq!(e.message, "CHECK needs a positive number");
    assert_eq!(&src[e.span.unwrap().range()], "CHECK -1");
    assert_eq!(error("THROW \"error"), "THROW \"ERROR");
}

#[test]
fn erract() {
    let src = "TO F :size\nMAKE \"erract [PRINT :size]\nFD \"x\nEND\nF 7";
    let r = run(src);
    assert_eq!(r.printed(), "7\n");
    assert_eq!(r.error().message, "FD doesn't like x as input");
    // Not for errors that are caught.
    assert_eq!(output("MAKE \"erract [PRINT \"oops] CATCH \"error [FD \"x] PRINT \"fine"), "fine\n");
    // At the top level too.
    assert_eq!(run("MAKE \"erract [PRINT \"oops] FD \"x").printed(), "oops\n");
}
