//! What the counters and variables panels show (phase 4 issue 02).

mod common;

use std::sync::{Arc, atomic::Ordering};

use common::Shared;
use pendown_logo::{
    Control, Event, EventSink, Halted, Interpreter, Watch,
    watch::{ScopeKind, VALUE_LIMIT},
};

/// Keeps the events and replies to every `Sync`, as a front end in
/// lockstep would.
struct Replying(Shared);

impl EventSink for Replying {
    fn send(&mut self, event: Event) -> Result<(), Halted> {
        if let Event::Sync(sync) = &event {
            sync.clone().reply();
        }
        self.0.send(event)
    }
}

/// A short form of a `Watch`: the counters, with `*` on the one `REPCOUNT`
/// outputs, then each scope's variables, with `~` on hidden ones.
fn describe(w: &Watch) -> String {
    let mut parts: Vec<String> = w
        .counters
        .iter()
        .map(|c| {
            let of = c.of.map_or(String::new(), |of| format!(" of {of}"));
            format!("{} {}{of}{}", c.name, c.count, if c.repcount { "*" } else { "" })
        })
        .collect();
    for scope in &w.scopes {
        let vars: Vec<String> = scope
            .variables
            .iter()
            .map(|v| format!("{}{}={}", if v.hidden { "~" } else { "" }, v.name, v.value.as_deref().unwrap_or("-")))
            .collect();
        let name = match scope.kind {
            ScopeKind::Globals => "globals",
            _ => &scope.name,
        };
        parts.push(format!("{name}: {}", vars.join(" ")));
    }
    parts.join(" | ")
}

/// Runs `src` in lockstep, and returns its events.
fn events(src: &str) -> Vec<Event> {
    let events = Shared::default();
    let control = Arc::new(Control::default());
    control.lockstep.store(true, Ordering::Relaxed);
    let mut it = Interpreter::new(Box::new(Replying(events.clone())), control);
    it.run_program(src).unwrap();
    events.0.take()
}

/// Each `Watch` sent, with the text of the instruction it was sent for,
/// or "end" for the one at the end of the run.
fn watches(src: &str) -> Vec<(String, String)> {
    let mut pending = None;
    let mut out = Vec::new();
    for event in events(src) {
        match event {
            Event::Watch(w) => pending = Some(describe(&w)),
            Event::Instruction { span, .. } => {
                if let Some(w) = pending.take() {
                    out.push((src[span.start as usize..span.end as usize].to_string(), w));
                }
            }
            _ => {}
        }
    }
    out.extend(pending.map(|w| ("end".to_string(), w)));
    out
}

/// What the panels show while `instruction` is highlighted for the `nth`
/// time, counting from 1: the last `Watch` before it.
fn at(src: &str, instruction: &str, nth: usize) -> String {
    let mut seen = 0;
    let mut shown = String::new();
    for event in events(src) {
        match event {
            Event::Watch(w) => shown = describe(&w),
            Event::Instruction { span, .. } if &src[span.start as usize..span.end as usize] == instruction => {
                seen += 1;
                if seen == nth {
                    return shown;
                }
            }
            _ => {}
        }
    }
    panic!("{instruction} was highlighted {seen} times, not {nth}")
}

/// What the run left.
fn last(src: &str) -> String {
    watches(src).last().expect("a Watch at the end").1.clone()
}

#[test]
fn globals_are_sorted_and_keep_their_spelling() {
    assert_eq!(last("MAKE \"Zebra 1\nMAKE \"apple [a b]\nMAKE \"zebra 2"), "globals: apple=[a b] Zebra=2");
}

#[test]
fn a_value_is_shown_before_the_instruction_that_changes_it_runs() {
    let src = "MAKE \"x 1\nMAKE \"x 2\nFD :x";
    assert_eq!(at(src, "MAKE \"x 2", 1), "globals: x=1");
    assert_eq!(at(src, "FD :x", 1), "globals: x=2");
}

#[test]
fn an_unchanged_watch_isnt_sent_again() {
    let all = watches("MAKE \"x 1\nFD 1\nFD 2\nFD 3");
    let after: Vec<_> = all.iter().map(|(h, _)| h.as_str()).collect();
    // The first instruction sees nothing, and the next sees x, which
    // nothing changes after that.
    assert_eq!(after, ["MAKE \"x 1", "FD 1"]);
}

#[test]
fn special_variables_are_hidden_at_their_starting_values() {
    assert_eq!(last("MAKE \"a 1"), "globals: a=1");
    assert_eq!(last("MAKE \"caseignoredp \"false"), "globals: caseignoredp=false");
}

#[test]
fn procedures_show_their_inputs_and_locals_in_the_call_chain() {
    let src = "TO OUTER :size\n LOCAL \"n\n INNER :size * 2\n FD 1\nEND\nTO INNER :Size\n FD :size\nEND\nMAKE \"size 1\nOUTER 10";
    // One name, spelt as its first definition spells it.
    assert_eq!(at(src, "FD :size", 1), "globals: ~size=1 | OUTER: ~size=10 n=- | INNER: size=20");
    // Nothing is left of the frames when the run ends.
    assert_eq!(last(src), "globals: size=1");
}

#[test]
fn a_procedure_with_no_inputs_is_still_in_the_chain() {
    assert_eq!(at("TO SQ\n FD 10\nEND\nSQ", "FD 10", 1), "globals:  | SQ: ");
}

#[test]
fn a_tail_call_replaces_the_frame() {
    // As ADR 0007 says: the caller never resumes.
    let src = "TO OUTER :a\n INNER 2\nEND\nTO INNER :b\n FD :b\nEND\nOUTER 1";
    assert_eq!(at(src, "FD :b", 1), "globals:  | INNER: a=1 b=2");
    let src = "TO DOWN :n\n IF :n = 0 [STOP]\n DOWN :n - 1\nEND\nDOWN 3";
    assert_eq!(at(src, "STOP", 1), "globals:  | DOWN: n=0");
}

#[test]
fn counters_are_outermost_first_and_repcount_is_marked() {
    let src = "REPEAT 3 [REPEAT 2 [FD 1]]";
    assert_eq!(at(src, "FD 1", 1), "REPEAT 1 of 3 | REPEAT 1 of 2* | globals: ");
    assert_eq!(at(src, "FD 1", 4), "REPEAT 2 of 3 | REPEAT 2 of 2* | globals: ");
    assert_eq!(last(src), "globals: ");
}

#[test]
fn a_one_line_loop_sends_each_pass() {
    let src = "REPEAT 3 [FD 1]";
    let passes: Vec<String> = (1..=3).map(|n| at(src, "FD 1", n)).collect();
    assert_eq!(passes, ["REPEAT 1 of 3* | globals: ", "REPEAT 2 of 3* | globals: ", "REPEAT 3 of 3* | globals: "]);
}

#[test]
fn forever_has_no_total() {
    let src = "TO GO\n FOREVER [IF REPCOUNT = 2 [STOP] FD 1]\nEND\nGO";
    assert_eq!(at(src, "FD 1", 1), "FOREVER 1* | globals:  | GO: ");
}

#[test]
fn for_has_a_counter_and_a_scope_but_repcount_skips_it() {
    let src = "REPEAT 2 [FOR [i 10 30 10] [FD :i]]";
    assert_eq!(at(src, "FD :i", 3), "REPEAT 1 of 2* | FOR 3 of 3 | globals:  | FOR: i=30");
    assert_eq!(at("FOR [i 5 1] [FD :i]", "FD :i", 2), "FOR 2 of 5 | globals:  | FOR: i=4");
}

#[test]
fn templates_count_their_members_and_lambdas_have_scopes() {
    let src = "FOREACH [a b c] [[w] PRINT :w]";
    assert_eq!(at(src, "PRINT :w", 2), "FOREACH 2 of 3* | globals:  | FOREACH: w=b");
}

#[test]
fn an_array_changed_in_place_is_sent_again() {
    let src = "MAKE \"a {1 2}\nSETITEM 1 :a 9\nFD 1";
    assert_eq!(at(src, "FD 1", 1), "globals: a={9 2}");
}

#[test]
fn long_values_are_shortened() {
    let all = last("MAKE \"xs ISEQ 1 300000\nMAKE \"deep [[[[[[1]]]]]]");
    assert_eq!(
        all,
        "globals: deep=[[[[...]]]] xs=[1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 ...]"
    );
    let long = last(&format!("MAKE \"w \"{}", "x".repeat(1000)));
    assert_eq!(long.len(), "globals: w=".len() + VALUE_LIMIT + 3);
}

#[test]
fn instant_sends_a_watch_only_when_asked() {
    let events = Shared::default();
    let control = Arc::new(Control::default());
    let mut it = Interpreter::new(Box::new(events.clone()), control.clone());
    it.run_program("MAKE \"x 1\nMAKE \"x 2").unwrap();
    let count = |events: &Shared| events.0.borrow().iter().filter(|e| matches!(e, Event::Watch(_))).count();
    assert_eq!(count(&events), 1, "only at the end");

    control.watch.store(true, Ordering::Relaxed);
    it.run_console("MAKE \"y 3\nMAKE \"y 4").unwrap();
    assert_eq!(count(&events), 3, "one asked for, and one at the end");
    assert!(!control.watch.load(Ordering::Relaxed), "the request is answered");
}

#[test]
fn the_cli_never_pays_for_watches() {
    let r = common::run("MAKE \"x 1 REPEAT 3 [FD 1]");
    assert!(!r.events.iter().any(|e| matches!(e, Event::Watch(_))));
}
