//! One interpreter for the whole session (ADR 0006): the console runs
//! against whatever the last run left.

use std::{thread, time::Duration};

use pendown::{
    animator::{Animator, Speed},
    run::{DEFAULT_CHANNEL, Job, Run, Session},
};
use pendown_logo::RunError;

/// Plays a job to the end, and returns how many marks it drew.
fn play(animator: &mut Animator) -> usize {
    let mut marks = 0;
    for _ in 0..10_000 {
        if animator.finished() {
            return marks;
        }
        marks += animator
            .frame(Duration::from_millis(16), usize::MAX)
            .marks
            .len();
        thread::sleep(Duration::from_micros(200));
    }
    panic!("the job didn't finish");
}

fn finish(session: &Session, job: Job) -> (Run, usize) {
    let (mut animator, run) = session.start(job, Speed::Instant, DEFAULT_CHANNEL);
    let marks = play(&mut animator);
    while run.result().is_none() {
        thread::sleep(Duration::from_micros(200));
    }
    (run, marks)
}

#[test]
fn the_console_uses_what_the_last_run_left() {
    let session = Session::new();
    let (run, _) = finish(
        &session,
        Job::Program("TO BOX :n\n REPEAT 4 [FD :n RT 90]\nEND\nMAKE \"size 30".into()),
    );
    assert_eq!(run.result(), Some(Ok(())));
    let (run, marks) = finish(&session, Job::Console("BOX :size".into()));
    assert_eq!((run.result(), marks), (Some(Ok(())), 4));
    // A TO at the console works too...
    let (run, _) = finish(
        &session,
        Job::Console("TO TWICE :n OUTPUT :n * 2 END".into()),
    );
    assert_eq!(run.result(), Some(Ok(())));
    let (run, marks) = finish(&session, Job::Console("FD TWICE 10".into()));
    assert_eq!((run.result(), marks), (Some(Ok(())), 1));
    // ...until the next Run clears everything.
    finish(&session, Job::Program("PRINT 1".into()));
    let (run, _) = finish(&session, Job::Console("BOX 10".into()));
    let Some(Err(RunError::Parse(errors))) = run.result() else {
        panic!("{:?}", run.result())
    };
    assert_eq!(errors[0].message, "I don't know how to BOX");
}

#[test]
fn a_stopped_job_doesnt_stop_the_next() {
    let session = Session::new();
    let (mut animator, run) = session.start(
        Job::Program("FOREVER [FD 1 RT 1]".into()),
        Speed::Animated(10.0),
        DEFAULT_CHANNEL,
    );
    animator.frame(Duration::from_millis(16), usize::MAX);
    animator.stop();
    while run.result().is_none() {
        thread::sleep(Duration::from_micros(200));
    }
    assert_eq!(run.result(), Some(Err(RunError::Halted)));
    let (run, marks) = finish(&session, Job::Console("FD 10".into()));
    assert_eq!((run.result(), marks), (Some(Ok(())), 1));
}
