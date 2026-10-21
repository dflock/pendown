//! The animator with a real interpreter thread, but no window: frames are
//! driven by hand, with a made-up frame time.

use std::{
    thread,
    time::{Duration, Instant},
};

use pendown_logo::RunError;
use pendown::{
    animator::{Animator, Mode, Speed},
    run::{self, DEFAULT_CHANNEL, Run},
};

const FRAME: Duration = Duration::from_millis(16);

fn start(src: &str, speed: Speed) -> (Animator, Run) {
    run::start(src.to_string(), speed, DEFAULT_CHANNEL)
}

/// Plays one frame. If nothing was ready, gives the interpreter a moment,
/// so the tests don't depend on how fast it starts.
fn frame(animator: &mut Animator, dt: Duration) -> pendown::animator::Frame {
    for _ in 0..200 {
        let out = animator.frame(dt, usize::MAX);
        if out.events > 0 || !animator.wants_frames() || animator.in_progress() {
            return out;
        }
        thread::sleep(Duration::from_micros(500));
    }
    animator.frame(dt, usize::MAX)
}

fn highlighted<'a>(src: &'a str, animator: &Animator) -> &'a str {
    animator.highlight.map_or("", |span| &src[span.range()])
}

fn wait_for_exit(run: &Run) -> Duration {
    let start = Instant::now();
    while run.exited().is_none() {
        assert!(start.elapsed() < Duration::from_secs(2), "the interpreter didn't stop");
        thread::sleep(Duration::from_micros(100));
    }
    run.exited().unwrap().saturating_duration_since(start)
}

/// Question 4: the highlight moves instruction by instruction, in step
/// with the turtle.
#[test]
fn highlight_follows_the_turtle() {
    let src = "REPEAT 2 [FD 50 RT 90]";
    let (mut animator, _run) = start(src, Speed::Animated(100.0));
    let mut seen: Vec<(String, f64, f64)> = Vec::new();
    let mut frames = 0;
    while !animator.finished() && frames < 1000 {
        let out = frame(&mut animator, FRAME);
        frames += 1;
        let text = highlighted(src, &animator).to_string();
        if out.highlight_changed || seen.last().is_none_or(|(t, ..)| *t != text) {
            seen.push((text, animator.pos.y, animator.heading));
        }
    }
    let order: Vec<_> = seen.iter().map(|(t, ..)| t.as_str()).collect();
    assert_eq!(order, ["REPEAT 2 [FD 50 RT 90]", "FD 50", "RT 90", "FD 50", "RT 90"]);
    // 2 moves of 0.5 s, 2 turns of 0.3 s and REPEAT's dwell of 0.1 s.
    let expected = (2.0 * 0.5 + 2.0 * 0.3 + 0.1) / FRAME.as_secs_f64();
    assert!((frames as f64 - expected).abs() <= 3.0, "{frames} frames, expected about {expected:.0}");
}

#[test]
fn highlight_is_on_the_move_while_the_turtle_moves() {
    let src = "FD 100\nRT 90";
    let (mut animator, _run) = start(src, Speed::Animated(100.0));
    let mut moving_frames = 0;
    while !animator.finished() {
        frame(&mut animator, FRAME);
        if let Some((points, _)) = animator.partial(1.0) {
            assert_eq!(highlighted(src, &animator), "FD 100");
            let y = points.last().unwrap().y;
            assert!(y > 0.0 && y < 100.0);
            moving_frames += 1;
        }
    }
    assert!(moving_frames >= 60, "{moving_frames}");
}

/// Question 5: a speed change applies to the very next frame, even in the
/// middle of a move.
#[test]
fn speed_changes_mid_move() {
    let (mut animator, _run) = start("WINDOW FD 1000", Speed::Animated(100.0));
    let dt = Duration::from_millis(100);
    let mut ys = Vec::new();
    for i in 0..10 {
        if i == 5 {
            animator.set_speed(Speed::Animated(400.0));
        }
        frame(&mut animator, dt);
        ys.push(animator.pos.y);
    }
    let steps: Vec<_> = ys.windows(2).map(|w| (w[1] - w[0]).round()).collect();
    assert_eq!(steps, [10.0, 10.0, 10.0, 10.0, 40.0, 40.0, 40.0, 40.0, 40.0]);
    animator.set_speed(Speed::Instant);
    frame(&mut animator, dt);
    assert_eq!(animator.pos.y.round(), 1000.0, "Instant finishes the move at once");
}

const STEPPER: &str = "MAKE \"n 0
REPEAT 2 [
  MAKE \"n :n + 1
  FD 10
  PRINT :n
]
PRINT \"done";

/// Plays frames until the animator pauses, and returns what was printed.
fn until_paused(animator: &mut Animator) -> String {
    let mut printed = String::new();
    for _ in 0..2000 {
        let out = frame(animator, FRAME);
        printed.extend(out.printed);
        if animator.mode() == Mode::Paused || animator.finished() {
            break;
        }
    }
    printed
}

/// Question 3: Step stops at the start of each line, and the interpreter
/// has never started a line the highlight hasn't reached.
#[test]
fn step_runs_one_line_and_nothing_ahead() {
    let src = STEPPER;
    let (mut animator, run) = start(src, Speed::Animated(1000.0));
    animator.pause();
    let mut steps = Vec::new();
    while !animator.finished() && steps.len() < 20 {
        animator.step_line();
        let printed = until_paused(&mut animator);
        // Give the interpreter every chance to run ahead, if it could.
        thread::sleep(Duration::from_millis(20));
        assert_eq!(animator.lines_ahead(), 0, "at {:?}", highlighted(src, &animator));
        steps.push(format!("{}{}", highlighted(src, &animator), if printed.is_empty() { String::new() } else { format!(" | printed {printed:?}") }));
    }
    let expected = [
        "MAKE \"n 0",
        "REPEAT 2 [\n  MAKE \"n :n + 1\n  FD 10\n  PRINT :n\n]",
        "MAKE \"n :n + 1",
        "FD 10",
        "PRINT :n",
        "MAKE \"n :n + 1 | printed \"1\\n\"",
        "FD 10",
        "PRINT :n",
        "PRINT \"done | printed \"2\\n\"",
        "PRINT \"done | printed \"done\\n\"",
    ];
    assert_eq!(steps, expected);
    assert_eq!(run.result(), Some(Ok(())));
}

#[test]
fn step_instruction_goes_inside_a_line() {
    let src = "REPEAT 2 [FD 10 RT 90]\nPRINT \"done";
    let (mut animator, _run) = start(src, Speed::Animated(1000.0));
    animator.pause();
    let mut order = Vec::new();
    for _ in 0..7 {
        animator.step_instruction();
        until_paused(&mut animator);
        order.push(highlighted(src, &animator).to_string());
    }
    assert_eq!(order, ["REPEAT 2 [FD 10 RT 90]", "FD 10", "RT 90", "FD 10", "RT 90", "PRINT \"done", "PRINT \"done"]);
}

#[test]
fn step_line_runs_a_whole_repeat() {
    let src = "REPEAT 4 [FD 10 RT 90]\nPRINT \"done";
    let (mut animator, _run) = start(src, Speed::Animated(1000.0));
    animator.pause();
    animator.step_line();
    until_paused(&mut animator);
    assert_eq!(highlighted(src, &animator), "REPEAT 4 [FD 10 RT 90]");
    animator.step_line();
    until_paused(&mut animator);
    assert_eq!(highlighted(src, &animator), "PRINT \"done");
    assert_eq!(animator.heading.round(), 0.0, "all four turns played");
}

/// Pausing at a slow speed is exact too: the interpreter waits at the next
/// line start.
#[test]
fn pause_while_running_is_exact_at_animated_speeds() {
    let src = "REPEAT 1000 [\n  FD 1\n  RT 1\n]";
    let (mut animator, _run) = start(src, Speed::Animated(500.0));
    for _ in 0..30 {
        frame(&mut animator, FRAME);
    }
    animator.pause();
    thread::sleep(Duration::from_millis(50));
    let ahead = animator.lines_ahead();
    assert!((0..=1).contains(&ahead), "{ahead} lines ahead");
    // Step finishes the current line and stops at the next.
    animator.step_line();
    until_paused(&mut animator);
    thread::sleep(Duration::from_millis(20));
    assert_eq!(animator.lines_ahead(), 0);
}

/// At Instant the interpreter runs ahead freely, so a pause leaves it
/// ahead. The first Step plays what it already ran, then lockstep resumes.
#[test]
fn pause_from_instant_catches_up_on_the_first_step() {
    let src = "REPEAT 100000 [\n  FD 1\n  RT 1\n]";
    let (mut animator, _run) = start(src, Speed::Instant);
    frame(&mut animator, FRAME);
    animator.pause();
    thread::sleep(Duration::from_millis(50));
    let ahead = animator.lines_ahead();
    assert!(ahead > 10, "{ahead} lines ahead after pausing from Instant");
    eprintln!("paused from Instant {ahead} lines ahead");
    animator.set_speed(Speed::Animated(100_000.0));
    animator.step_line();
    until_paused(&mut animator);
    thread::sleep(Duration::from_millis(20));
    assert_eq!(animator.lines_ahead(), 0);
}

/// Question 1: Stop ends an endless program at once, wherever it's blocked.
#[test]
fn stop_while_running_at_instant() {
    let (mut animator, run) = start("FOREVER [FD 1 RT 1]", Speed::Instant);
    for _ in 0..10 {
        frame(&mut animator, FRAME);
    }
    animator.stop();
    let took = wait_for_exit(&run);
    assert_eq!(run.result(), Some(Err(RunError::Halted)));
    eprintln!("Stop at Instant: {took:?}");
}

#[test]
fn stop_while_blocked_on_a_full_channel() {
    // One line, so no Syncs: the interpreter blocks on the full channel.
    let (mut animator, run) = start("FOREVER [FD 1 RT 1]", Speed::Animated(10.0));
    frame(&mut animator, FRAME);
    thread::sleep(Duration::from_millis(100));
    assert!(run.exited().is_none());
    animator.stop();
    let took = wait_for_exit(&run);
    assert_eq!(run.result(), Some(Err(RunError::Halted)));
    eprintln!("Stop on a full channel: {took:?}");
}

#[test]
fn stop_while_waiting_at_a_held_line() {
    let (mut animator, run) = start(STEPPER, Speed::Animated(100.0));
    animator.pause();
    animator.step_line();
    until_paused(&mut animator);
    thread::sleep(Duration::from_millis(50));
    animator.stop();
    let took = wait_for_exit(&run);
    assert_eq!(run.result(), Some(Err(RunError::Halted)));
    eprintln!("Stop at a held line: {took:?}");
}

/// Question 2, without the drawing: Instant gets through a big fractal.
#[test]
fn instant_plays_a_big_fractal() {
    let src = "WINDOW
TO TREE :size :depth
  IF :depth = 0 [STOP]
  FD :size
  LT 30 TREE :size * 0.7 :depth - 1
  RT 60 TREE :size * 0.7 :depth - 1
  LT 30 BK :size
END
TREE 200 14";
    let (mut animator, run) = start(src, Speed::Instant);
    let started = Instant::now();
    let mut segments = 0;
    while !animator.finished() {
        segments += frame(&mut animator, FRAME).marks.len();
    }
    assert_eq!(run.result(), Some(Ok(())));
    assert_eq!(segments, 2 * (2usize.pow(14) - 1));
    eprintln!("{segments} segments in {:?}, not drawing them", started.elapsed());
}

#[test]
fn clean_and_hiding_the_turtle() {
    let (mut animator, _run) = start("FD 50 CLEAN FD 20 HT", Speed::Instant);
    let mut cleaned = false;
    let mut segments = Vec::new();
    while !animator.finished() {
        let out = frame(&mut animator, FRAME);
        if out.cleaned {
            cleaned = true;
            segments.clear();
        }
        segments.extend(out.marks);
    }
    assert!(cleaned);
    assert_eq!(segments.len(), 1, "only the move after CLEAN is left");
    assert!(!animator.shown);
}

#[test]
fn arcs_carry_the_turtle_along_them() {
    // A quarter circle of radius 100 is 157 steps: about 1.6 s at 100 a second.
    let (mut animator, _run) = start("ARC 90 100", Speed::Animated(100.0));
    let mut frames = 0;
    let mut halfway = None;
    while !animator.finished() && frames < 1000 {
        frame(&mut animator, FRAME);
        frames += 1;
        if frames == 48 {
            halfway = Some((animator.pos, animator.heading, animator.partial(0.1).map(|(p, _)| p.len())));
        }
    }
    assert!((frames as f64 - 157.0 / 100.0 / FRAME.as_secs_f64()).abs() <= 3.0, "{frames} frames");
    let (pos, heading, partial) = halfway.unwrap();
    // Half way round, the turtle is 45 degrees along, on the circle.
    assert!((heading - 45.0).abs() < 2.0, "heading {heading}");
    assert!((((pos.x - 100.0).powi(2) + pos.y.powi(2)).sqrt() - 100.0).abs() < 1e-6);
    assert!(partial.unwrap() > 2, "the arc so far is drawn as a curve");
    assert_eq!((animator.pos.x.round(), animator.pos.y.round(), animator.heading.round()), (100.0, 100.0, 90.0));
}

#[test]
fn shapes_sweep_round_and_pen_up_shapes_take_no_time() {
    let (mut animator, _run) = start("CIRCLE 50", Speed::Animated(100.0));
    let mut frames = 0;
    while !animator.finished() && frames < 1000 {
        frame(&mut animator, FRAME);
        frames += 1;
        if frames == 10 {
            let (points, _) = animator.partial(0.1).expect("sweeping");
            assert_eq!(points[0].y.round(), 50.0, "starting straight ahead");
            assert_eq!((animator.pos.x, animator.pos.y), (0.0, 0.0), "the turtle stays put");
        }
    }
    // 314 steps round at 100 a second.
    assert!((frames as f64 - std::f64::consts::PI / FRAME.as_secs_f64()).abs() <= 3.0, "{frames} frames");
    let (mut animator, _run) = start("PU CIRCLE 50 SQUARE 80", Speed::Animated(100.0));
    let mut frames = 0;
    while !animator.finished() && frames < 1000 {
        frame(&mut animator, FRAME);
        frames += 1;
    }
    assert!(frames < 20, "{frames} frames for shapes nobody can see");
}
