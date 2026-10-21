//! The tests from spike 03, one group per question in its issue:
//! `.scratch/phase-1-spikes/issues/03-interpreter-core.md`.

use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        Arc,
        atomic::Ordering,
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

use pendown_logo::{Error, Event, EventSink, Halted, Interpreter, Limits, RunError, Span, Stats};

#[derive(Clone, Default)]
struct Shared(Rc<RefCell<Vec<Event>>>);

impl EventSink for Shared {
    fn send(&mut self, event: Event) -> Result<(), Halted> {
        self.0.borrow_mut().push(event);
        Ok(())
    }
}

/// Stops the program after `limit` moves, like pressing Stop.
struct StopAfterMoves {
    moves: Rc<RefCell<u64>>,
    limit: u64,
}

impl EventSink for StopAfterMoves {
    fn send(&mut self, event: Event) -> Result<(), Halted> {
        if let Event::Move { .. } = event {
            let mut moves = self.moves.borrow_mut();
            *moves += 1;
            if *moves >= self.limit {
                return Err(Halted);
            }
        }
        Ok(())
    }
}

fn interpreter() -> (Interpreter, Shared) {
    let events = Shared::default();
    let it = Interpreter::new(Box::new(events.clone()), Arc::default());
    (it, events)
}

struct Run {
    result: Result<(), RunError>,
    events: Vec<Event>,
    stats: Stats,
}

impl Run {
    fn printed(&self) -> String {
        self.events
            .iter()
            .filter_map(|e| if let Event::Print(text) = e { Some(text.as_str()) } else { None })
            .collect()
    }

    fn moves(&self) -> usize {
        self.events.iter().filter(|e| matches!(e, Event::Move { .. })).count()
    }

    fn error(&self) -> &Error {
        match &self.result {
            Err(RunError::Runtime(e)) => e,
            other => panic!("expected a runtime error, got {other:?}"),
        }
    }
}

fn run_with(src: &str, limits: Limits) -> Run {
    let (mut it, events) = interpreter();
    it.limits = limits;
    let result = it.run_program(src);
    let events = events.0.borrow().clone();
    Run { result, events, stats: it.stats }
}

fn run(src: &str) -> Run {
    run_with(src, Limits { instruction_events: false, ..Limits::default() })
}

/// What `src` prints, which must run without errors.
fn output(src: &str) -> String {
    let r = run(src);
    assert_eq!(r.result, Ok(()), "{src}");
    r.printed()
}

/// Runs `f` on a thread with a stack of `bytes`, as the GUI will.
fn on_stack<T: Send + 'static>(bytes: usize, f: impl FnOnce() -> T + Send + 'static) -> T {
    thread::Builder::new().stack_size(bytes).spawn(f).unwrap().join().unwrap()
}

const MB: usize = 1024 * 1024;

/// A small stack and a low depth limit: if a call that should be a tail call
/// isn't one, these fail with "called itself too many times".
fn tail_limits() -> Limits {
    Limits { max_depth: 50, stack_bytes: Some(MB), instruction_events: false }
}

fn tail_run(src: &'static str) -> (Result<(), RunError>, String, usize) {
    on_stack(MB, move || {
        let r = run_with(src, tail_limits());
        (r.result.clone(), r.printed(), r.stats.max_depth)
    })
}

// 1. Endless tail recursion.

#[test]
fn spiral_runs_in_constant_space() {
    let (moves, result, stats) = on_stack(MB, || {
        let moves = Rc::new(RefCell::new(0));
        let sink = StopAfterMoves { moves: moves.clone(), limit: 1_000_000 };
        let mut it = Interpreter::new(Box::new(sink), Arc::default());
        it.limits = tail_limits();
        // The one-line form from the issue.
        let result = it.run_program("TO SPIRAL :n FD :n RT 90 SPIRAL :n + 1 END\nSPIRAL 1");
        let moves = *moves.borrow();
        (moves, result.clone(), it.stats)
    });
    assert_eq!(result, Err(RunError::Halted));
    assert_eq!(moves, 1_000_000);
    assert_eq!(stats.max_depth, 1, "every SPIRAL call was a tail call");
}

#[test]
fn tail_call_inside_if() {
    let (r, out, depth) = tail_run("TO DOWN :n\n IF :n > 0 [DOWN :n - 1]\nEND\nDOWN 1000000\nPRINT \"done");
    assert_eq!((r, out.as_str(), depth), (Ok(()), "done\n", 1));
}

#[test]
fn tail_call_inside_ifelse() {
    let (r, out, depth) = tail_run("TO DOWN :n\n IFELSE :n = 0 [PRINT \"done] [DOWN :n - 1]\nEND\nDOWN 1000000");
    assert_eq!((r, out.as_str(), depth), (Ok(()), "done\n", 1));
}

#[test]
fn tail_call_inside_three_input_if() {
    let (r, out, _) = tail_run("TO DOWN :n\n (IF :n = 0 [PRINT \"done] [DOWN :n - 1])\nEND\nDOWN 1000000");
    assert_eq!((r, out.as_str()), (Ok(()), "done\n"));
}

#[test]
fn mutual_tail_recursion() {
    let src = "TO PING :n\n IF :n > 0 [PONG :n - 1]\nEND\nTO PONG :n\n IF :n > 0 [PING :n - 1]\nEND\nPING 1000001\nPRINT \"done";
    let (r, out, depth) = tail_run(src);
    assert_eq!((r, out.as_str(), depth), (Ok(()), "done\n", 1));
}

#[test]
fn tail_call_after_output() {
    let src = "TO COUNTUP :n :acc\n IF :n = 0 [OUTPUT :acc]\n OUTPUT COUNTUP :n - 1 :acc + 1\nEND\nPRINT COUNTUP 1000000 0";
    let (r, out, depth) = tail_run(src);
    assert_eq!((r, out.as_str(), depth), (Ok(()), "1000000\n", 1));
}

#[test]
fn tail_call_inside_output_ifelse() {
    let src = "TO COUNTUP :n :acc\n OUTPUT IFELSE :n = 0 [:acc] [COUNTUP :n - 1 :acc + 1]\nEND\nPRINT COUNTUP 1000000 0";
    let (r, out, _) = tail_run(src);
    assert_eq!((r, out.as_str()), (Ok(()), "1000000\n"));
}

#[test]
fn calls_that_are_not_tail_calls_still_nest() {
    // Something runs after the call, so it can't be a tail call.
    let (r, _, _) = tail_run("TO G :n\n IF :n > 0 [G :n - 1 PRINT :n]\nEND\nG 100");
    assert!(matches!(r, Err(RunError::Runtime(e)) if e.message.starts_with("G called itself too many times")));
    // REPEAT's list is never in tail position.
    let (r, _, _) = tail_run("TO R :n\n IF :n > 0 [REPEAT 1 [R :n - 1]]\nEND\nR 100");
    assert!(matches!(r, Err(RunError::Runtime(e)) if e.message.starts_with("R called itself too many times")));
}

#[test]
fn tail_calls_keep_dynamic_scope() {
    // B is a tail call from A, so it shares A's frame, but still sees :x.
    let src = "TO A :x\n B 5\nEND\nTO B :y\n PRINT :x + :y\nEND\nMAKE \"x 100\nA 1\nPRINT :x";
    assert_eq!(output(src), "6\n100\n");
}

#[test]
fn tail_call_output_with_nowhere_to_go() {
    let r = run("TO F\n G\nEND\nTO G\n OUTPUT 3\nEND\nF");
    assert_eq!(r.error().message, "You don't say what to do with 3");
}

// 2. Deep recursion that isn't a tail call.

const TREE: &str = "
TO TREE :size :depth
  IF :depth = 0 [STOP]
  FD :size
  LT 30 TREE :size * 0.7 :depth - 1
  RT 60 TREE :size * 0.7 :depth - 1
  LT 30 BK :size
END
TREE 100 12
";

const FIB: &str = "
TO FIB :n
  IF :n < 2 [OUTPUT :n]
  OUTPUT (FIB :n - 1) + (FIB :n - 2)
END
PRINT FIB 25
";

fn deep_limits() -> Limits {
    Limits { stack_bytes: Some(256 * MB), instruction_events: false, ..Limits::default() }
}

#[test]
fn tree_fractal_12_deep() {
    let (moves, depth) = on_stack(256 * MB, || {
        let r = run_with(TREE, deep_limits());
        assert_eq!(r.result, Ok(()));
        (r.moves(), r.stats.max_depth)
    });
    // FD and BK in each of the 4095 calls that draw.
    assert_eq!(moves, 2 * (2usize.pow(12) - 1));
    assert_eq!(depth, 13);
}

#[test]
fn fibonacci_25() {
    let out = on_stack(256 * MB, || run_with(FIB, deep_limits()).printed());
    assert_eq!(out, "75025\n");
}

#[test]
fn past_the_depth_limit_is_a_friendly_error() {
    let src = "TO DOWN :n\n DOWN :n + 1\n PRINT :n\nEND\nDOWN 1";
    let (message, depth, calls) = on_stack(256 * MB, move || {
        let r = run_with(src, deep_limits());
        let e = r.error().clone();
        (e.message, r.stats.max_depth, e.calls.len())
    });
    assert_eq!(message, "DOWN called itself too many times: 10001 calls deep. Does it have a way to STOP?");
    assert_eq!(depth, 10_001);
    assert_eq!(calls, 10_000);
}

#[test]
fn running_out_of_stack_is_the_same_friendly_error() {
    // A small stack and no real depth limit: the stack guard stops it.
    let src = "TO DOWN :n\n DOWN :n + 1\n PRINT :n\nEND\nDOWN 1";
    let message = on_stack(4 * MB, move || {
        let limits = Limits { max_depth: usize::MAX, stack_bytes: Some(4 * MB), instruction_events: false };
        run_with(src, limits).error().message.clone()
    });
    assert!(message.starts_with("DOWN called itself too many times"), "{message}");
}

#[test]
fn mutual_recursion_past_the_limit() {
    let src = "TO A :n\n B :n + 1\n PRINT :n\nEND\nTO B :n\n A :n + 1\n PRINT :n\nEND\nA 1";
    let message = on_stack(256 * MB, move || run_with(src, deep_limits()).error().message.clone());
    assert_eq!(message, "Too many procedures were running inside each other (10001 deep) when A was called");
}

// 3. Calls before definitions.

#[test]
fn call_above_its_definition() {
    let r = run("SQUARE 10\nTO SQUARE :n\n  REPEAT 4 [FD :n RT 90]\nEND");
    assert_eq!((r.result.clone(), r.moves()), (Ok(()), 4));
}

#[test]
fn definitions_see_each_other_whatever_the_order() {
    let src = "PRINT TWICE 3\nTO TWICE :n\n OUTPUT DOUBLE :n\nEND\nTO DOUBLE :n OUTPUT :n * 2 END";
    assert_eq!(output(src), "6\n");
}

#[test]
fn arity_from_a_later_definition_groups_inputs() {
    // ADD takes two inputs, so `ADD 1 2 3` is ADD 1 2, then a stray 3.
    let (mut it, _) = interpreter();
    let sexp = it.parse_to_sexp("PRINT ADD 1 2\nTO ADD :a :b OUTPUT :a + :b END");
    assert_eq!(sexp, Ok(vec!["(print (add 1 2))".to_string()]));
}

#[test]
fn parse_errors_are_collected_and_nothing_runs() {
    let r = run("PRINT \"before\nFD\nFOO 3\nTO SQUARE :n\nREPEAT 4 [FD :n RT 90]");
    let Err(RunError::Parse(errors)) = r.result else { panic!("{:?}", r.result) };
    let messages: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
    assert_eq!(messages, ["SQUARE has no END", "Not enough inputs to FD", "I don't know how to FOO"]);
    assert!(r.events.is_empty());
}

#[test]
fn run_starts_from_a_clean_slate_and_the_console_keeps_it() {
    let (mut it, events) = interpreter();
    it.run_program("TO SQUARE :n REPEAT 4 [FD :n RT 90] END\nMAKE \"size 20").unwrap();
    it.run_console("SQUARE :size").unwrap();
    assert_eq!(events.0.borrow().iter().filter(|e| matches!(e, Event::Move { .. })).count(), 4);
    it.run_program("PRINT 1").unwrap();
    let Err(RunError::Parse(errors)) = it.run_console("SQUARE 10") else { panic!() };
    assert_eq!(errors[0].message, "I don't know how to SQUARE");
}

#[test]
fn bad_procedure_names() {
    let r = run("TO FD :n END");
    let Err(RunError::Parse(errors)) = r.result else { panic!() };
    assert_eq!(errors[0].message, "FD is already a Pendown command, so pick another name");
}

// 4. The minus sign.

fn sexp(src: &str) -> Vec<String> {
    let (mut it, _) = interpreter();
    it.parse_to_sexp(src).unwrap_or_else(|e| panic!("{src}: {e:?}"))
}

#[test]
fn minus_sign() {
    let cases = [
        ("FD -5", "(forward (- 5))"),
        // A minus where an input is expected is unary, as in UCBLogo.
        ("FD - 5", "(forward (- 5))"),
        ("PRINT :a - 5", "(print (difference :a 5))"),
        ("PRINT :a-5", "(print (difference :a 5))"),
        // Two inputs: :a and -5.
        ("SUM :a -5", "(sum :a (- 5))"),
        ("(PRINT :a -5)", "(print :a (- 5))"),
        // Unary minus binds tighter than infix, MINUS takes a whole expression.
        ("PRINT - 3 + 4", "(print (sum (- 3) 4))"),
        ("PRINT MINUS 3 + 4", "(print (minus (sum 3 4)))"),
        ("PRINT 2*-3", "(print (product 2 (- 3)))"),
        ("PRINT -:a * 2", "(print (product (- :a) 2))"),
        ("PRINT 1e-3 + 1", "(print (sum 1e-3 1))"),
    ];
    for (src, expected) in cases {
        assert_eq!(sexp(src), [expected], "{src}");
    }
    // Without parentheses, PRINT takes one input, so -5 is a new instruction.
    assert_eq!(sexp("PRINT :a -5"), ["(print :a)", "(- 5)"]);
}

#[test]
fn minus_sign_values() {
    assert_eq!(output("PRINT - 3 + 4"), "1\n");
    assert_eq!(output("PRINT MINUS 3 + 4"), "-7\n");
    assert_eq!(output("MAKE \"a 10 PRINT :a-5 PRINT :a - 5 PRINT SUM :a -5"), "5\n5\n5\n");
}

#[test]
fn minus_sign_errors() {
    let r = run("MAKE \"a 10\nPRINT SUM :a - 5");
    let Err(RunError::Parse(errors)) = r.result else { panic!() };
    assert_eq!(errors[0].message, "Not enough inputs to SUM");
}

// 5. Spans.

fn texts(src: &str, events: &[Event]) -> Vec<(String, bool)> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Instruction { span, starts_line } => Some((src[span.range()].to_string(), *starts_line)),
            _ => None,
        })
        .collect()
}

fn traced(src: &str) -> Run {
    run_with(src, Limits::default())
}

#[test]
fn spans_inside_repeat_and_procedures() {
    let src = "TO SQUARE :n\n  REPEAT 2 [FD :n RT 90]\nEND\nSQUARE 10 PRINT \"hi";
    let r = traced(src);
    assert_eq!(r.result, Ok(()));
    let expected = [
        ("SQUARE 10", true),
        ("REPEAT 2 [FD :n RT 90]", true),
        ("FD :n", false),
        ("RT 90", false),
        ("FD :n", false),
        ("RT 90", false),
        // Back on line 4 after SQUARE's body, so Step pauses here too.
        ("PRINT \"hi", true),
    ];
    let expected: Vec<_> = expected.iter().map(|(t, s)| (t.to_string(), *s)).collect();
    assert_eq!(texts(src, &r.events), expected);
}

#[test]
fn multi_line_lists_start_lines() {
    let src = "REPEAT 2 [\n  FD 10\n  RT 90\n]";
    let r = traced(src);
    let starts: Vec<_> = texts(src, &r.events).into_iter().map(|(t, s)| format!("{t}:{s}")).collect();
    assert_eq!(starts, ["REPEAT 2 [\n  FD 10\n  RT 90\n]:true", "FD 10:true", "RT 90:true", "FD 10:true", "RT 90:true"]);
}

#[test]
fn spans_of_one_line_definitions_and_infix() {
    let src = "TO DOUBLE :n OUTPUT :n * 2 END\nPRINT DOUBLE 3 + 1";
    let r = traced(src);
    let texts: Vec<_> = texts(src, &r.events).into_iter().map(|(t, _)| t).collect();
    assert_eq!(texts, ["PRINT DOUBLE 3 + 1", "OUTPUT :n * 2"]);
    assert_eq!(r.printed(), "8\n");
}

#[test]
fn runtime_error_spans_and_call_chain() {
    let src = "TO A\n B\n PRINT 1\nEND\nTO B\n FD \"far\n PRINT 2\nEND\nA";
    let r = run(src);
    let e = r.error();
    assert_eq!(e.message, "FD doesn't like far as input");
    assert_eq!(&src[e.span.unwrap().range()], "\"far");
    let chain: Vec<_> = e.calls.iter().map(|c| (c.procedure.as_str(), &src[c.span.range()], c.span.line)).collect();
    assert_eq!(chain, [("B", "B", 1), ("A", "A", 8)]);
    assert_eq!(e.to_string(), "FD doesn't like far as input, in B (line 2), called from A (line 9)");
}

#[test]
fn parse_error_spans() {
    let src = "FD 10\nRT\nREPEAT 4 [FD 10";
    let r = run(src);
    let Err(RunError::Parse(errors)) = r.result else { panic!() };
    // The reader fails first, on the unclosed bracket, before anything parses.
    assert_eq!(errors.len(), 1);
    assert_eq!(&src[errors[0].span.unwrap().range()], "[");
    let src = "FD 10\nRT\nPRINT (1 + 2";
    let Err(RunError::Parse(errors)) = run(src).result else { panic!() };
    let found: Vec<_> = errors.iter().map(|e| (e.message.as_str(), &src[e.span.unwrap().range()])).collect();
    assert_eq!(found, [("Not enough inputs to RT", "RT"), ("This ( has no matching )", "(")]);
}

#[test]
fn errors_in_lists_point_into_the_list() {
    let src = "REPEAT 2 [FD 10 FOO]";
    let e = run(src).error().clone();
    assert_eq!((e.message.as_str(), &src[e.span.unwrap().range()]), ("I don't know how to FOO", "FOO"));
}

#[test]
fn definition_spans() {
    let (mut it, _) = interpreter();
    let src = "PRINT 1\nTO SQUARE :n\nREPEAT 4 [FD :n RT 90]\nEND";
    it.run_program(src).unwrap();
    let span: Span = it.definition("square").unwrap();
    assert_eq!(&src[span.range()], "TO SQUARE :n");
}

// 6. Numbers.

#[test]
fn printing_numbers() {
    let cases = [
        ("4/2", "2"),
        ("3/2", "1.5"),
        ("0.1 + 0.2", "0.3"),
        ("1/3", "0.333333333333333"),
        ("2/3", "0.666666666666667"),
        ("-7/2", "-3.5"),
        ("123456789012345", "123456789012345"),
        ("2 * 4503599627370496", "9.00719925474099e+15"),
        ("1e20 * 1", "1e+20"),
        ("1/100000", "1e-05"),
        ("0.0001", "0.0001"),
        ("0 * -1", "0"),
        ("1.50", "1.50"),
        ("1.50 + 0", "1.5"),
        ("WORD 1.50 \"", "1.50"),
        ("WORD 1 2", "12"),
        ("(WORD 1 2) + 1", "13"),
    ];
    for (expr, expected) in cases {
        assert_eq!(output(&format!("PRINT {expr}")), format!("{expected}\n"), "{expr}");
    }
}

#[test]
fn equality() {
    let cases = [
        ("2 = 2.0", "true"),
        ("\"2.0 = 2", "true"),
        ("\"ABC = \"abc", "true"),
        ("\"abc = 3", "false"),
        // Numbers compare rounded to 15 significant digits, as they print.
        ("0.1 + 0.2 = 0.3", "true"),
        ("0.1 + 0.2 < 0.3", "false"),
        ("0.1 + 0.2 > 0.3", "false"),
        ("0.1 + 0.2 <= 0.3", "true"),
        ("0.1 + 0.2 <> 0.3", "false"),
        ("0.3 < 0.300000000000001", "true"),
        ("1 < 2", "true"),
        ("3 <> 4", "true"),
        ("EQUALP 1 1", "true"),
    ];
    for (expr, expected) in cases {
        assert_eq!(output(&format!("PRINT {expr}")), format!("{expected}\n"), "{expr}");
    }
}

#[test]
fn stepping_by_a_tenth_reaches_one() {
    let src = "MAKE \"x 0\nREPEAT 10 [MAKE \"x :x + 0.1]\nPRINT :x\nPRINT :x = 1";
    assert_eq!(output(src), "1\ntrue\n");
}

#[test]
fn one_line_and_multi_line_definitions_mix() {
    let src = "TO DOUBLE :n OUTPUT :n * 2 END\nTO SQUARE :n\n  REPEAT 4 [FD DOUBLE :n RT 90]\nEND\nSQUARE 5 PRINT DOUBLE 4";
    let r = run(src);
    assert_eq!((r.result.clone(), r.moves(), r.printed().as_str()), (Ok(()), 4, "8\n"));
}

#[test]
fn number_errors() {
    let cases = [
        ("PRINT 1 / 0", "Can't divide by zero"),
        ("PRINT 1e300 * 1e300", "That number is too big"),
        ("REPEAT 2.5 [FD 1]", "REPEAT doesn't like 2.5 as input"),
        ("PRINT \"abc + 1", "+ doesn't like abc as input"),
        ("PRINT - \"abc", "- doesn't like abc as input"),
    ];
    for (src, expected) in cases {
        assert_eq!(run(src).error().message, expected, "{src}");
    }
}

// 7. Stop.

#[test]
fn stop_ends_forever_promptly() {
    let control: Arc<pendown_logo::Control> = Arc::default();
    let (tx, rx) = mpsc::sync_channel(1024);
    let flag = control.clone();
    let interp = thread::spawn(move || {
        let mut it = Interpreter::new(Box::new(tx), flag);
        it.run_program("FOREVER [FD 1 RT 1]").clone()
    });
    let drain = thread::spawn(move || rx.into_iter().count());
    thread::sleep(Duration::from_millis(200));
    let start = Instant::now();
    control.cancel.store(true, Ordering::Relaxed);
    let result = interp.join().unwrap();
    let latency = start.elapsed();
    let events = drain.join().unwrap();
    assert_eq!(result, Err(RunError::Halted));
    assert!(events > 1000, "it ran: {events} events");
    assert!(latency < Duration::from_millis(50), "{latency:?}");
    eprintln!("stopped FOREVER in {latency:?} after {events} events");
}

#[test]
fn stop_wakes_a_blocked_interpreter() {
    // Nobody reads the channel, as when the animator is paused, so the
    // interpreter blocks once it's full. Dropping the receiver wakes it.
    let (tx, rx) = mpsc::sync_channel(16);
    let interp = thread::spawn(move || {
        let mut it = Interpreter::new(Box::new(tx), Arc::default());
        it.run_program("FOREVER [FD 1 RT 1]").clone()
    });
    thread::sleep(Duration::from_millis(100));
    assert!(!interp.is_finished());
    let start = Instant::now();
    drop(rx);
    let result = interp.join().unwrap();
    assert_eq!(result, Err(RunError::Halted));
    assert!(start.elapsed() < Duration::from_millis(50), "{:?}", start.elapsed());
}

#[test]
fn stop_is_not_an_error_at_top_level() {
    assert_eq!(run("STOP").error().message, "STOP can only be used inside a procedure");
}

// Display spelling (ADR 0004), added with phase 1.

#[test]
fn the_first_definition_decides_the_spelling() {
    // Called as `square` before it's defined as `Square`.
    let r = run("square 1\nTO Square :n\n FD \"far\n PRINT 1\nEND");
    assert_eq!(r.error().calls[0].procedure, "Square");

    let (mut it, _) = interpreter();
    it.run_program("PRINT 1\nMAKE \"Size 3\nPRINT :SIZE").unwrap();
    assert_eq!(it.parse_to_sexp("PRINT :size"), Ok(vec!["(print :Size)".to_string()]));

    // Run starts again: the next definition decides.
    it.run_program("MAKE \"SIZE 4").unwrap();
    assert_eq!(it.parse_to_sexp("PRINT :size"), Ok(vec!["(print :SIZE)".to_string()]));
}
