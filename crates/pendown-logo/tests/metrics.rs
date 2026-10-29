//! The interpreter's metrics (phase 3 issue 02).

use std::{
    sync::{Arc, atomic::Ordering, mpsc},
    thread,
    time::Duration,
};

use pendown_logo::{BatchSender, Control, Event, Interpreter, metrics};

#[test]
fn instructions_are_counted_for_each_job() {
    metrics::set_enabled(true);
    let control = Arc::new(Control::default());
    let mut it = Interpreter::new(Box::new(Vec::new()), control.clone());
    it.run_program("REPEAT 10000 [MAKE \"x 1]").unwrap();
    // REPEAT, its 10,000 passes and the MAKEs.
    let first = control.metrics.instructions.get();
    assert!((20_000..=20_002).contains(&first), "{first}");
    assert_eq!(control.metrics.running.count(), 1);

    // The next job counts from zero.
    it.run_console("MAKE \"y 2").unwrap();
    assert_eq!(control.metrics.instructions.get(), 1);
    assert_eq!(control.metrics.running.count(), 2);
}

#[test]
fn waits_are_measured() {
    metrics::set_enabled(true);
    let control = Arc::new(Control::default());
    control.lockstep.store(true, Ordering::Relaxed);
    let (tx, rx) = mpsc::sync_channel::<Vec<Event>>(1);
    let sink = BatchSender::new(tx, 1, Duration::from_millis(4)).measured(control.clone());
    let job = {
        let control = control.clone();
        thread::spawn(move || {
            Interpreter::new(Box::new(sink), control).run_program("FD 1\nFD 2\nFD 3")
        })
    };
    // A slow front end: it takes a while to read each batch, and to reply
    // to each Sync.
    for batch in rx {
        thread::sleep(Duration::from_millis(5));
        for event in batch {
            if let Event::Sync(sync) = event {
                thread::sleep(Duration::from_millis(5));
                sync.reply();
            }
        }
    }
    job.join().unwrap().unwrap();
    let m = &control.metrics;
    assert_eq!(m.lockstep.count(), 3, "a Sync at each line");
    assert!(
        m.lockstep.total() >= Duration::from_millis(15),
        "{:?}",
        m.lockstep.total()
    );
    assert!(
        m.channel.total() >= Duration::from_millis(10),
        "{:?}",
        m.channel.total()
    );
    assert!(m.running.total() >= m.lockstep.total() + m.channel.total());
}
