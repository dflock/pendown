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

/// The editor's program, with a seed of its own.
fn program(text: &str) -> Job {
    Job::Program {
        text: text.into(),
        seed: 1,
    }
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
        program("TO BOX :n\n REPEAT 4 [FD :n RT 90]\nEND\nMAKE \"size 30"),
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
    finish(&session, program("PRINT 1"));
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
        program("FOREVER [FD 1 RT 1]"),
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

#[test]
fn undo_runs_on_the_interpreter_thread() {
    let session = Session::new();
    finish(&session, program(""));
    finish(&session, Job::Console("FD 40 RT 90".into()));
    let (run, marks) = finish(&session, Job::Undo);
    assert_eq!((run.result(), marks), (Some(Ok(())), 0));
    let (mut animator, run) = session.start(
        Job::Console("PRINT (LIST POS HEADING)".into()),
        Speed::Instant,
        DEFAULT_CHANNEL,
    );
    let mut printed = Vec::new();
    while !animator.finished() {
        printed.extend(
            animator
                .frame(Duration::from_millis(16), usize::MAX)
                .printed,
        );
        thread::sleep(Duration::from_micros(200));
    }
    while run.result().is_none() {
        thread::sleep(Duration::from_micros(200));
    }
    assert_eq!(printed.concat(), "[0 0] 0\n");
}

/// Pause-and-edit issue 03: the GUI picks the seed and sends it with the
/// job, so running a program again with its seed draws the same picture,
/// and another seed draws another.
#[test]
fn the_same_seed_draws_the_same_picture() {
    let src = "REPEAT 20 [FD RANDOM 100 RT RANDOM 360]";
    let session = Session::new();
    let marks = |seed| {
        let job = Job::Program {
            text: src.into(),
            seed,
        };
        let (mut animator, run) = session.start(job, Speed::Instant, DEFAULT_CHANNEL);
        let mut marks = Vec::new();
        while !animator.finished() {
            marks.extend(animator.frame(Duration::from_millis(16), usize::MAX).marks);
            thread::sleep(Duration::from_micros(200));
        }
        assert_eq!(run.seed, Some(seed));
        marks
    };
    let first = marks(7);
    assert_eq!(first.len(), 20);
    assert_eq!(marks(7), first, "the same seed, the same picture");
    assert_ne!(marks(8), first, "another seed, another picture");
}
