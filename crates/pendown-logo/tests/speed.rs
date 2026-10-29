//! `WAIT`, `SETSPEED` and `SPEED` (phase 1 issue 12).

mod common;

use std::{sync::Arc, time::Duration};

use common::*;
use pendown_logo::{Control, Event, Interpreter};

#[test]
fn wait_sends_its_duration() {
    let r = run("WAIT 30 WAIT 0 WAIT 1.5");
    let waits: Vec<_> = r
        .events
        .iter()
        .filter_map(|e| {
            if let Event::Wait(d) = e {
                Some(*d)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        waits,
        [Duration::from_millis(500), Duration::from_millis(25)],
        "WAIT 0 sends nothing"
    );
    assert_eq!(error("WAIT -1"), "WAIT doesn't like -1 as input");
}

#[test]
fn setspeed_and_speed() {
    let r = run("PRINT SPEED SETSPEED 3 PRINT SPEED");
    assert_eq!(r.printed(), "11\n3\n", "11 with no slider, as in the CLI");
    assert!(r.events.contains(&Event::SetSpeed(3)));
    for bad in ["0", "12", "2.5", "\"fast"] {
        assert_eq!(
            error(&format!("SETSPEED {bad}")),
            format!(
                "SETSPEED doesn't like {} as input",
                bad.trim_start_matches('"')
            )
        );
    }
}

#[test]
fn speed_reads_the_sliders_level() {
    let events = Shared::default();
    let control = Arc::new(Control::default());
    let mut it = Interpreter::new(Box::new(events.clone()), control.clone());
    control.speed.store(6, std::sync::atomic::Ordering::Relaxed);
    it.run_program("PRINT SPEED").unwrap();
    assert!(events.0.borrow().contains(&Event::Print("6\n".into())));
}
