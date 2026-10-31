//! The animator with a real interpreter thread, but no window: frames are
//! driven by hand, with a made-up frame time.

use std::{
    thread,
    time::{Duration, Instant},
};

use pendown::{
    animator::{Animator, FADE, Mode, Speed, Thinking, Thought},
    run::{self, DEFAULT_CHANNEL, Run},
};
use pendown_logo::RunError;

const FRAME: Duration = Duration::from_millis(16);

fn start(src: &str, speed: Speed) -> (Animator, Run) {
    run::start(src.to_string(), speed, DEFAULT_CHANNEL)
}

/// Plays one frame. If nothing was ready, gives the interpreter a moment,
/// so the tests don't depend on how fast it starts. A frame that produced
/// anything is kept, even with no new events: the last frame of a shape
/// gives its mark, and the interpreter may not have finished yet when the
/// machine is busy, so throwing that frame away would lose the mark.
fn frame(animator: &mut Animator, dt: Duration) -> pendown::animator::Frame {
    for _ in 0..200 {
        let out = animator.frame(dt, usize::MAX);
        let produced = !out.marks.is_empty()
            || !out.printed.is_empty()
            || out.cleaned
            || out.background.is_some()
            || out.speed.is_some()
            || out.highlight_changed
            || out.watch_changed;
        if out.events > 0 || produced || !animator.wants_frames() || animator.in_progress() {
            return out;
        }
        thread::sleep(Duration::from_micros(500));
    }
    animator.frame(dt, usize::MAX)
}

/// Plays tiny frames until a move or turn is under way: its event can come
/// a frame or two after its highlight when the machine is busy, so a test
/// that measures it starts here.
fn until_started(animator: &mut Animator) {
    for _ in 0..1000 {
        if animator.in_progress() && animator.highlight.is_some() {
            return;
        }
        frame(animator, Duration::from_millis(1));
    }
    panic!("nothing started");
}

fn highlighted<'a>(src: &'a str, animator: &Animator) -> &'a str {
    animator.highlight.map_or("", |span| &src[span.range()])
}

fn wait_for_exit(run: &Run) -> Duration {
    let start = Instant::now();
    while run.exited().is_none() {
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "the interpreter didn't stop"
        );
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
    assert_eq!(
        order,
        ["REPEAT 2 [FD 50 RT 90]", "FD 50", "RT 90", "FD 50", "RT 90"]
    );
    // 2 moves of 0.5 s, 2 turns of 0.3 s and REPEAT's dwell of 0.1 s.
    let expected = (2.0 * 0.5 + 2.0 * 0.3 + 0.1) / FRAME.as_secs_f64();
    assert!(
        (frames as f64 - expected).abs() <= 3.0,
        "{frames} frames, expected about {expected:.0}"
    );
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
    // Until the line is being drawn, past WINDOW's moment on show.
    for _ in 0..1000 {
        if animator.partial(0.1).is_some() {
            break;
        }
        frame(&mut animator, Duration::from_millis(1));
    }
    let mut ys = Vec::new();
    for i in 0..10 {
        if i == 5 {
            animator.set_speed(Speed::Animated(400.0));
        }
        frame(&mut animator, dt);
        ys.push(animator.pos.y);
    }
    let steps: Vec<_> = ys.windows(2).map(|w| (w[1] - w[0]).round()).collect();
    assert_eq!(
        steps,
        [10.0, 10.0, 10.0, 10.0, 40.0, 40.0, 40.0, 40.0, 40.0]
    );
    animator.set_speed(Speed::Instant);
    frame(&mut animator, dt);
    assert_eq!(
        animator.pos.y.round(),
        1000.0,
        "Instant finishes the move at once"
    );
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

/// Question 3: Single Step stops at each instruction, and the interpreter
/// has never started a line the highlight hasn't reached. With one
/// instruction a line, that's a line at a time.
#[test]
fn single_step_runs_one_instruction_and_nothing_ahead() {
    let src = STEPPER;
    let (mut animator, run) = start(src, Speed::Animated(1000.0));
    animator.pause();
    let mut steps = Vec::new();
    while !animator.finished() && steps.len() < 20 {
        animator.single_step();
        let printed = until_paused(&mut animator);
        // Give the interpreter every chance to run ahead, if it could.
        thread::sleep(Duration::from_millis(20));
        assert_eq!(
            animator.lines_ahead(),
            0,
            "at {:?}",
            highlighted(src, &animator)
        );
        steps.push(format!(
            "{}{}",
            highlighted(src, &animator),
            if printed.is_empty() {
                String::new()
            } else {
                format!(" | printed {printed:?}")
            }
        ));
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
fn single_step_goes_inside_a_line() {
    let src = "REPEAT 2 [FD 10 RT 90]\nPRINT \"done";
    let (mut animator, _run) = start(src, Speed::Animated(1000.0));
    animator.pause();
    let mut order = Vec::new();
    for _ in 0..7 {
        animator.single_step();
        until_paused(&mut animator);
        order.push(highlighted(src, &animator).to_string());
    }
    assert_eq!(
        order,
        [
            "REPEAT 2 [FD 10 RT 90]",
            "FD 10",
            "RT 90",
            "FD 10",
            "RT 90",
            "PRINT \"done",
            "PRINT \"done"
        ]
    );
}

/// The panels' contents, in short: the counters, then the variables.
fn watched(animator: &Animator) -> String {
    let Some(w) = &animator.watch else {
        return String::new();
    };
    let mut parts: Vec<String> = w
        .counters
        .iter()
        .map(|c| format!("{} {}", c.name, c.count))
        .collect();
    for scope in &w.scopes {
        for v in &scope.variables {
            parts.push(format!("{}={}", v.name, v.value.as_deref().unwrap_or("-")));
        }
    }
    parts.join(" ")
}

/// Phase 4 issue 03: stepping shows what each instruction sees, as it's
/// highlighted.
#[test]
fn the_panels_keep_in_step_with_the_highlight() {
    let src = "MAKE \"x 0\nREPEAT 2 [MAKE \"x :x + 5 FD 10]\nPRINT :x";
    let (mut animator, _run) = start(src, Speed::Animated(1000.0));
    animator.pause();
    let mut seen = Vec::new();
    for _ in 0..8 {
        animator.single_step();
        until_paused(&mut animator);
        thread::sleep(Duration::from_millis(5));
        seen.push(format!(
            "{} | {}",
            highlighted(src, &animator),
            watched(&animator)
        ));
    }
    assert_eq!(
        seen,
        [
            "MAKE \"x 0 | ",
            "REPEAT 2 [MAKE \"x :x + 5 FD 10] | x=0",
            "MAKE \"x :x + 5 | REPEAT 1 x=0",
            "FD 10 | REPEAT 1 x=5",
            "MAKE \"x :x + 5 | REPEAT 2 x=5",
            "FD 10 | REPEAT 2 x=10",
            "PRINT :x | x=10",
            "PRINT :x | x=10",
        ]
    );
}

/// At animated speeds, running freely, a value the program changes shows
/// up with the instruction after the change.
#[test]
fn the_panels_follow_a_running_program() {
    let src = "MAKE \"x 1\nFD 20\nMAKE \"x 2\nFD 20";
    let (mut animator, _run) = start(src, Speed::Animated(200.0));
    let mut seen: Vec<String> = Vec::new();
    while !animator.finished() {
        frame(&mut animator, FRAME);
        let now = format!("{} | {}", highlighted(src, &animator), watched(&animator));
        if seen.last() != Some(&now) {
            seen.push(now);
        }
    }
    assert_eq!(
        seen,
        [
            "MAKE \"x 1 | ",
            "FD 20 | x=1",
            "MAKE \"x 2 | x=1",
            "FD 20 | x=2"
        ]
    );
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
    // Single Step plays up to the next instruction, which is the next line.
    animator.single_step();
    until_paused(&mut animator);
    thread::sleep(Duration::from_millis(20));
    assert_eq!(animator.lines_ahead(), 0);
}

/// Pause mid-move finishes the move at once, in the next frame, and pauses
/// with the highlight on the next instruction, before it runs.
#[test]
fn pause_finishes_the_move_at_once() {
    let src = "FD 100\nRT 90\nFD 100";
    let (mut animator, _run) = start(src, Speed::Animated(10.0));
    until_started(&mut animator);
    frame(&mut animator, FRAME);
    assert!(animator.pos.y < 50.0, "part-way along: {:?}", animator.pos);
    animator.pause();
    let out = frame(&mut animator, FRAME);
    assert_eq!(animator.mode(), Mode::Paused);
    assert_eq!(out.marks.len(), 1, "the whole line is drawn");
    assert_eq!(animator.pos.y.round(), 100.0);
    assert!(!animator.in_progress());
    assert_eq!(highlighted(src, &animator), "RT 90");
    assert_eq!(animator.heading, 0.0, "RT 90 hasn't run");
    // Paused, nothing moves, however long it's left.
    for _ in 0..20 {
        assert!(frame(&mut animator, FRAME).marks.is_empty());
    }
    assert_eq!(animator.heading, 0.0);
}

/// An arc too, and a turn: Kame is never left part-way.
#[test]
fn pause_finishes_an_arc_and_a_turn() {
    let src = "ARC 180 50\nRT 180\nFD 10";
    let (mut animator, _run) = start(src, Speed::Animated(10.0));
    until_started(&mut animator);
    animator.pause();
    let out = frame(&mut animator, FRAME);
    assert_eq!(out.marks.len(), 1, "the whole arc");
    assert_eq!(highlighted(src, &animator), "RT 180");
    animator.single_step();
    until_started(&mut animator);
    animator.pause();
    frame(&mut animator, FRAME);
    assert_eq!(highlighted(src, &animator), "FD 10");
    assert_eq!(
        animator.heading.round(),
        0.0,
        "the whole of RT 180 after the arc's 180"
    );
}

/// At Instant, Pause stops at the next instruction too.
#[test]
fn pause_at_instant_stops_before_an_instruction() {
    let src = "REPEAT 100000 [\n  FD 1\n  RT 1\n]";
    let (mut animator, _run) = start(src, Speed::Instant);
    frame(&mut animator, FRAME);
    animator.pause();
    frame(&mut animator, FRAME);
    assert_eq!(animator.mode(), Mode::Paused);
    assert!(!animator.in_progress());
    let at = highlighted(src, &animator);
    assert!(at == "FD 1" || at == "RT 1", "{at:?}");
}

/// At Instant the interpreter runs ahead freely, so a pause leaves it
/// ahead. Single Steps play what it already ran, then lockstep resumes.
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
    for _ in 0..=2 * ahead {
        animator.single_step();
        until_paused(&mut animator);
    }
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
fn stop_while_waiting_at_a_paused_line() {
    let (mut animator, run) = start(STEPPER, Speed::Animated(100.0));
    animator.pause();
    animator.single_step();
    until_paused(&mut animator);
    thread::sleep(Duration::from_millis(50));
    animator.stop();
    let took = wait_for_exit(&run);
    assert_eq!(run.result(), Some(Err(RunError::Halted)));
    eprintln!("Stop at a paused line: {took:?}");
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
    eprintln!(
        "{segments} segments in {:?}, not drawing them",
        started.elapsed()
    );
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
            halfway = Some((
                animator.pos,
                animator.heading,
                animator.partial(0.1).map(|(p, _)| p.len()),
            ));
        }
    }
    assert!(
        (frames as f64 - 157.0 / 100.0 / FRAME.as_secs_f64()).abs() <= 3.0,
        "{frames} frames"
    );
    let (pos, heading, partial) = halfway.unwrap();
    // Half way round, the turtle is 45 degrees along, on the circle.
    assert!((heading - 45.0).abs() < 2.0, "heading {heading}");
    assert!((((pos.x - 100.0).powi(2) + pos.y.powi(2)).sqrt() - 100.0).abs() < 1e-6);
    assert!(partial.unwrap() > 2, "the arc so far is drawn as a curve");
    assert_eq!(
        (
            animator.pos.x.round(),
            animator.pos.y.round(),
            animator.heading.round()
        ),
        (100.0, 100.0, 90.0)
    );
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
            assert_eq!(
                (animator.pos.x, animator.pos.y),
                (0.0, 0.0),
                "the turtle stays put"
            );
        }
    }
    // 314 steps round at 100 a second.
    assert!(
        (frames as f64 - std::f64::consts::PI / FRAME.as_secs_f64()).abs() <= 3.0,
        "{frames} frames"
    );
    let (mut animator, _run) = start("PU CIRCLE 50 SQUARE 80", Speed::Animated(100.0));
    let mut frames = 0;
    while !animator.finished() && frames < 1000 {
        frame(&mut animator, FRAME);
        frames += 1;
    }
    assert!(frames < 20, "{frames} frames for shapes nobody can see");
}

#[test]
fn filled_shapes_with_the_pen_up_take_the_time_of_their_outline() {
    let (mut animator, _run) = start("PU SETFC \"red CIRCLE 50", Speed::Animated(100.0));
    let (mut frames, mut marks) = (0, Vec::new());
    while !animator.finished() && frames < 1000 {
        let out = frame(&mut animator, FRAME);
        assert!(animator.partial(0.1).is_none(), "no outline to show");
        marks.extend(out.marks);
        frames += 1;
    }
    // 314 steps round at 100 a second, after PU and SETFC hold the
    // highlight for 10 steps each, since they move nothing.
    let expected = (std::f64::consts::PI + 0.2) / FRAME.as_secs_f64();
    assert!((frames as f64 - expected).abs() <= 3.0, "{frames} frames");
    assert_eq!(marks.len(), 1, "the fill appears at the end");
    assert_eq!(marks[0].fill(), Some([255, 0, 0, 255]));
}

#[test]
fn wait_takes_its_time_even_at_instant() {
    let (mut animator, _run) = start("FD 10 WAIT 30 FD 10", Speed::Instant);
    let mut frames = 0;
    let mut highlight_during_wait = String::new();
    while !animator.finished() && frames < 1000 {
        frame(&mut animator, FRAME);
        frames += 1;
        if frames == 15 {
            highlight_during_wait = highlighted("FD 10 WAIT 30 FD 10", &animator).to_string();
        }
    }
    // Half a second at 16 ms a frame.
    assert!((29..=34).contains(&frames), "{frames} frames");
    assert_eq!(highlight_during_wait, "WAIT 30");
}

#[test]
fn pause_and_stop_during_a_wait() {
    let (mut animator, run) = start("WAIT 60\nFD 10", Speed::Instant);
    for _ in 0..10 {
        frame(&mut animator, FRAME);
    }
    animator.pause();
    for _ in 0..200 {
        animator.frame(FRAME, usize::MAX);
    }
    // Pause ends the wait, and pauses before the next instruction.
    assert!(!animator.finished(), "paused before FD 10");
    assert_eq!(highlighted("WAIT 60\nFD 10", &animator), "FD 10");
    animator.stop();
    assert!(animator.finished(), "Stop ends the playback");
    // At Instant the interpreter didn't wait: it had already finished, and
    // only the animator was waiting.
    wait_for_exit(&run);
    assert_eq!(run.result(), Some(Ok(())));
}

#[test]
fn setspeed_changes_the_speed_where_it_runs() {
    let (mut animator, _run) = start("FD 10\nSETSPEED 3\nFD 10", Speed::Instant);
    let mut changed = None;
    while !animator.finished() {
        if let Some(speed) = frame(&mut animator, FRAME).speed {
            changed = Some(speed);
        }
    }
    assert_eq!(
        changed,
        Some(Speed::Animated(20.0)),
        "level 3 is 20 steps a second"
    );
    assert_eq!(animator.speed(), Speed::Animated(20.0));
}

#[test]
fn metrics_count_frames_and_events() {
    pendown_logo::metrics::set_enabled(true);
    let src = "REPEAT 2 [FD 50 RT 90]";
    let (mut animator, run) = start(src, Speed::Animated(100.0));
    let mut frames = 0;
    while !animator.finished() {
        frame(&mut animator, FRAME);
        frames += 1;
    }
    let m = &animator.metrics;
    assert!(
        m.frames.count() >= frames,
        "every frame is timed: {} of {frames}",
        m.frames.count()
    );
    // REPEAT, then two each of FD and RT with their Instruction and Move or
    // Turn, and a Sync at the start of the one line. Four Watches: at
    // REPEAT, at each FD as the counter moves on, and at the end.
    assert_eq!(m.events.get(), 14);
    let interp = &run.control.metrics;
    assert_eq!(interp.lockstep.count(), 1, "one line, one Sync");
    assert_eq!(
        interp.instructions.get(),
        7,
        "REPEAT, its 2 passes, and 2 each of FD and RT"
    );
}

/// Issue 08: the window skips frames that would look the same.
#[test]
fn next_change_is_when_the_picture_moves_a_pixel() {
    use pendown::animator::MAX_FRAME_GAP;
    let ms = |d: Duration| d.as_secs_f64() * 1e3;

    // At 20 steps a second and 0.5 pixels a step, the line grows a pixel
    // every 100 ms, the most the window waits.
    let (mut animator, _run) = start("FD 300", Speed::Animated(20.0));
    until_started(&mut animator);
    assert_eq!(animator.next_change(0.5, 18.0), MAX_FRAME_GAP);
    // At 2 pixels a step, every 25 ms.
    assert!((ms(animator.next_change(2.0, 18.0)) - 25.0).abs() < 0.01);

    // A turn moves the turtle's tip: 18 pixels long, turning 60 degrees a
    // second, it moves about 19 pixels a second.
    let (mut animator, _run) = start("RT 90", Speed::Animated(20.0));
    until_started(&mut animator);
    let gap = ms(animator.next_change(1.0, 18.0));
    assert!(
        (gap - 1000.0 / (18.0 * 60.0f64.to_radians())).abs() < 0.01,
        "{gap}"
    );

    // A hidden turtle turning shows nothing until the turn is done.
    let (mut animator, _run) = start("HT\nRT 3", Speed::Animated(20.0));
    while !matches!(animator.highlight, Some(span) if span.line == 1) || !animator.in_progress() {
        frame(&mut animator, FRAME);
    }
    let gap = ms(animator.next_change(1.0, 18.0));
    assert!(gap > 40.0 && gap <= 50.0, "the rest of a 50 ms turn: {gap}");

    // Instant wants every frame.
    let (mut animator, _run) = start("FD 300", Speed::Instant);
    frame(&mut animator, FRAME);
    assert_eq!(animator.next_change(0.5, 18.0), Duration::ZERO);
}

/// Budgets (phase 3 issue 06) for what doesn't depend on the machine.
/// Changing one is a deliberate change in its own commit, saying why.
mod budgets {
    use super::*;
    use pendown::animator::MAX_FRAME_GAP;
    use pendown_render::Drawing;

    /// The canvas's scale in the scripted session's window: 690 pixels for
    /// 1000 steps.
    const PX_PER_STEP: f64 = 0.69;
    const DISPLAY_FRAME: Duration = Duration::from_micros(16_667);

    /// Plays `src` as the window does, a frame at a time, waiting as long as
    /// `next_change` says. Returns the frames it took, and the tiles
    /// uploaded on frames where nothing was drawn.
    fn play(src: &str, speed: Speed) -> (u64, u64) {
        let mut drawing = Drawing::new();
        drawing.resize(690, 690);
        drawing.take_dirty_tiles();
        let (mut animator, _run) = start(src, speed);
        let (mut frames, mut wasted) = (0, 0);
        let mut dt = DISPLAY_FRAME;
        while !animator.finished() {
            let out = frame(&mut animator, dt);
            frames += 1;
            assert!(frames < 100_000, "{src} never finished");
            drawing.push(&out.marks);
            drawing.draw_pending();
            let tiles = drawing.take_dirty_tiles().len() as u64;
            if out.marks.is_empty() {
                wasted += tiles;
            }
            dt = animator.next_change(PX_PER_STEP, 18.0).max(DISPLAY_FRAME);
        }
        (frames, wasted)
    }

    #[test]
    fn a_slow_run_draws_a_frame_for_each_pixel_or_so() {
        // 400 steps at 20 steps a second is 20 s, 1,200 frames at 60 a
        // second. At 0.69 pixels a step the lines grow 14 pixels a second,
        // so about 280 frames, and the turns about another 120.
        let (frames, wasted) = play("REPEAT 4 [FD 100 RT 90]", Speed::Animated(20.0));
        assert!(frames <= 450, "{frames} frames");
        assert_eq!(wasted, 0, "tiles uploaded on frames that drew nothing");
    }

    #[test]
    fn a_wait_takes_ten_frames_a_second() {
        let (frames, _) = play("WAIT 120", Speed::Animated(20.0));
        let expected = (2.0 / MAX_FRAME_GAP.as_secs_f64()) as u64;
        assert!(frames <= expected + 5, "a 2 s WAIT took {frames} frames");
    }

    #[test]
    fn a_fast_run_draws_every_frame() {
        // At 2,560 steps a second each frame moves the turtle well over a
        // pixel: 400 steps in about 10 frames.
        let (frames, wasted) = play("REPEAT 4 [FD 100 RT 90]", Speed::Animated(2560.0));
        assert!((10..=40).contains(&frames), "{frames} frames");
        assert_eq!(wasted, 0);
    }

    #[test]
    fn paused_and_finished_runs_want_no_frames() {
        let (mut animator, _run) = start("FD 100\nFD 100", Speed::Animated(20.0));
        frame(&mut animator, FRAME);
        // Pause takes one more frame, to finish the move.
        animator.pause();
        assert!(animator.wants_frames());
        frame(&mut animator, FRAME);
        assert_eq!(animator.mode(), Mode::Paused);
        assert!(!animator.wants_frames());
        let out = animator.frame(FRAME, usize::MAX);
        assert_eq!(
            (out.events, out.marks.len()),
            (0, 0),
            "a paused frame takes nothing"
        );
        animator.resume();
        while !animator.finished() {
            frame(&mut animator, MAX_FRAME_GAP);
        }
        assert!(!animator.wants_frames());
        assert_eq!(animator.frame(FRAME, usize::MAX).events, 0);
    }
}

/// Plays frames until the program has finished, keeping what Kame thought
/// about in each.
fn thoughts(src: &str, speed: Speed) -> Vec<Thinking> {
    let (mut animator, _run) = start(src, speed);
    let mut seen = Vec::new();
    for _ in 0..1000 {
        if animator.finished() {
            break;
        }
        frame(&mut animator, FRAME);
        seen.push(animator.thinking());
    }
    assert!(animator.finished());
    seen
}

#[test]
fn kame_thinks_about_the_turn_as_it_turns() {
    let seen = thoughts("RT 90", Speed::Animated(100.0));
    let turned: Vec<f64> = seen
        .iter()
        .filter_map(|t| match t.current {
            Some(Thought::Turn { from, by, .. }) => {
                assert_eq!(from, 0.0);
                Some(by)
            }
            _ => None,
        })
        .collect();
    assert!(turned.len() > 5, "{turned:?}");
    assert!(
        turned.windows(2).all(|w| w[0] <= w[1]),
        "it grows: {turned:?}"
    );
    assert!(turned[0] > 0.0 && turned[0] < 90.0);
    assert_eq!(turned.last(), Some(&90.0), "and stays at the end");
}

#[test]
fn kame_thinks_about_the_move_so_far() {
    let seen = thoughts("RT 90 FD 50", Speed::Animated(200.0));
    let moves: Vec<(f64, f64)> = seen
        .iter()
        .filter_map(|t| match t.current {
            Some(Thought::Move {
                from, to, length, ..
            }) => {
                assert_eq!((from.x, from.y), (0.0, 0.0));
                Some((to.x, length))
            }
            _ => None,
        })
        .collect();
    assert!(moves.len() > 5, "{moves:?}");
    for (x, length) in &moves {
        assert!(
            (x - length).abs() < 1e-9,
            "to is where Kame has got: {moves:?}"
        );
    }
    assert_eq!(moves.last(), Some(&(50.0, 50.0)));
}

#[test]
fn the_last_thought_fades_as_the_next_starts() {
    let seen = thoughts("FD 50 RT 90 FD 50", Speed::Animated(100.0));
    let first_turn = seen
        .iter()
        .position(|t| matches!(t.current, Some(Thought::Turn { .. })))
        .unwrap();
    let Some((Thought::Move { length, .. }, left)) = seen[first_turn].fading else {
        panic!("the move fades as the turn starts: {:?}", seen[first_turn]);
    };
    assert_eq!(length, 50.0);
    assert!(left > 0.0 && left <= 1.0);
    // A third of a second later, it's gone.
    let after = (FADE / FRAME.as_secs_f64()).ceil() as usize + 1;
    assert!(
        !matches!(
            seen[first_turn + after].fading,
            Some((Thought::Move { .. }, _))
        ),
        "{:?}",
        seen[first_turn + after]
    );
    // The turn fades as the second move starts.
    let second_move = first_turn
        + seen[first_turn..]
            .iter()
            .position(|t| matches!(t.current, Some(Thought::Move { .. })))
            .unwrap();
    assert!(matches!(
        seen[second_move].fading,
        Some((Thought::Turn { by: 90.0, .. }, _))
    ));
}

#[test]
fn more_than_a_whole_turn_and_turning_left() {
    let seen = thoughts("RT 450", Speed::Animated(2000.0));
    assert!(matches!(
        seen.last().unwrap().current,
        Some(Thought::Turn {
            by: 450.0,
            done: true,
            ..
        })
    ));
    let seen = thoughts("LT 30", Speed::Animated(100.0));
    assert!(matches!(
        seen.last().unwrap().current,
        Some(Thought::Turn {
            by: -30.0,
            done: true,
            ..
        })
    ));
}

#[test]
fn nothing_to_think_about_at_instant() {
    let seen = thoughts("FD 50 RT 90 FD 50", Speed::Instant);
    assert!(seen.iter().all(|t| *t == Thinking::default()), "{seen:?}");
}

mod replay {
    //! Pause-and-edit issue 04: replaying a program out of sight to a number
    //! of instructions, then carrying on.

    use super::*;
    use pendown::{
        animator::Replay,
        run::{Job, Session},
    };
    use pendown_render::Mark;

    const SPIRAL: &str = "MAKE \"n 0
REPEAT 30 [
  MAKE \"n :n + 1
  FD :n + RANDOM 10
  RT 30 + RANDOM 30
]
PRINT :n";

    fn job(text: &str) -> Job {
        Job::Program {
            text: text.into(),
            seed: 5,
        }
    }

    /// What the canvas would show: the marks, Kame, the highlight and the
    /// panels.
    #[derive(Debug, PartialEq)]
    struct Seen {
        marks: Vec<Mark>,
        pos: (i64, i64),
        heading: i64,
        highlight: String,
        watched: String,
        shown: u64,
    }

    fn seen(src: &str, animator: &Animator, marks: Vec<Mark>) -> Seen {
        Seen {
            marks,
            pos: (
                (animator.pos.x * 1000.0).round() as i64,
                (animator.pos.y * 1000.0).round() as i64,
            ),
            heading: (animator.heading * 1000.0).round() as i64,
            highlight: highlighted(src, animator).to_string(),
            watched: watched(animator),
            shown: animator.instructions_shown,
        }
    }

    /// Plays frames until paused or finished, keeping the marks.
    fn play_to_pause(animator: &mut Animator, marks: &mut Vec<Mark>) -> usize {
        let mut frames = 0;
        while animator.mode() != Mode::Paused && !animator.finished() {
            marks.extend(frame(animator, FRAME).marks);
            frames += 1;
            assert!(frames < 100_000, "it never paused");
        }
        frames
    }

    /// Pauses at the start, then Single Steps until `n` instructions have
    /// been shown.
    fn stepped_to(src: &str, n: u64) -> Seen {
        let (mut animator, _run) =
            Session::new().start(job(src), Speed::Animated(5000.0), DEFAULT_CHANNEL);
        let mut marks = Vec::new();
        animator.pause();
        play_to_pause(&mut animator, &mut marks);
        while animator.instructions_shown < n {
            animator.single_step();
            play_to_pause(&mut animator, &mut marks);
        }
        thread::sleep(Duration::from_millis(5));
        seen(src, &animator, marks)
    }

    #[test]
    fn a_replay_ends_where_stepping_would() {
        for n in [1, 2, 5, 17, 40, 91] {
            let (mut animator, _run) = Session::new().replay(
                job(SPIRAL),
                Speed::Animated(10.0),
                DEFAULT_CHANNEL,
                Some(Replay {
                    to: n,
                    then: Mode::Paused,
                }),
            );
            let mut marks = Vec::new();
            play_to_pause(&mut animator, &mut marks);
            assert!(!animator.replaying_now() && !animator.ended_early());
            thread::sleep(Duration::from_millis(5));
            assert_eq!(animator.lines_ahead(), 0, "nothing run ahead at {n}");
            assert_eq!(
                seen(SPIRAL, &animator, marks),
                stepped_to(SPIRAL, n),
                "at {n}"
            );
        }
    }

    #[test]
    fn nothing_is_shown_until_it_gets_there() {
        let (mut animator, _run) = Session::new().replay(
            job(SPIRAL),
            Speed::Animated(10.0),
            DEFAULT_CHANNEL,
            Some(Replay {
                to: 60,
                then: Mode::Paused,
            }),
        );
        animator.instant_slice = Duration::from_micros(50);
        let mut last = None;
        while animator.replaying_now() {
            let out = frame(&mut animator, FRAME);
            assert!(animator.thinking().current.is_none());
            last = Some((out.highlight_changed, out.watch_changed));
            if animator.replaying_now() {
                assert!(!out.highlight_changed && !out.watch_changed);
            }
        }
        assert_eq!(last, Some((true, true)), "it all shows when it gets there");
    }

    #[test]
    fn then_it_carries_on_at_the_sliders_speed() {
        let src = "FD 10\nFD 10\nFD 100\nFD 10";
        let (mut animator, _run) = Session::new().replay(
            job(src),
            Speed::Animated(10.0),
            DEFAULT_CHANNEL,
            Some(Replay {
                to: 3,
                then: Mode::Running,
            }),
        );
        let mut marks = Vec::new();
        while animator.replaying_now() {
            marks.extend(frame(&mut animator, FRAME).marks);
        }
        assert_eq!(marks.len(), 2);
        assert_eq!(highlighted(src, &animator), "FD 100");
        until_started(&mut animator);
        frame(&mut animator, FRAME);
        assert!(animator.in_progress(), "FD 100 plays at 10 steps a second");
        assert!(animator.pos.y > 20.0 && animator.pos.y < 30.0);
    }

    #[test]
    fn or_takes_a_single_step() {
        let src = "FD 10\nRT 90\nFD 10\nFD 10";
        let (mut animator, _run) = Session::new().replay(
            job(src),
            Speed::Animated(1000.0),
            DEFAULT_CHANNEL,
            Some(Replay {
                to: 2,
                then: Mode::StepInstruction,
            }),
        );
        let mut marks = Vec::new();
        while animator.replaying_now() {
            marks.extend(frame(&mut animator, FRAME).marks);
        }
        assert_eq!(highlighted(src, &animator), "RT 90");
        play_to_pause(&mut animator, &mut marks);
        assert_eq!(highlighted(src, &animator), "FD 10");
        assert_eq!(animator.instructions_shown, 3);
        assert_eq!(animator.heading.round(), 90.0);
    }

    #[test]
    fn a_shorter_program_just_ends() {
        let src = "REPEAT 3 [FD 10 RT 120]";
        let (mut animator, run) = Session::new().replay(
            job(src),
            Speed::Animated(10.0),
            DEFAULT_CHANNEL,
            Some(Replay {
                to: 1000,
                then: Mode::Paused,
            }),
        );
        let mut marks = Vec::new();
        play_to_pause(&mut animator, &mut marks);
        assert!(animator.finished());
        assert!(animator.ended_early());
        assert_eq!(marks.len(), 3, "all of it, as playing it to the end");
        while run.result().is_none() {
            thread::sleep(Duration::from_micros(200));
        }
        assert_eq!(run.result(), Some(Ok(())));
    }

    /// At speed 1, a replay takes about as long as at Instant: it doesn't
    /// wait for the slider.
    #[test]
    fn a_replay_doesnt_wait_for_the_slider() {
        let src = "REPEAT 200 [\n  FD 10\n  RT 10\n  WAIT 1\n]";
        let frames = |speed| {
            let (mut animator, _run) = Session::new().replay(
                job(src),
                speed,
                DEFAULT_CHANNEL,
                Some(Replay {
                    to: 790,
                    then: Mode::Paused,
                }),
            );
            play_to_pause(&mut animator, &mut Vec::new())
        };
        let (slow, instant) = (frames(Speed::Animated(5.0)), frames(Speed::Instant));
        assert!(
            slow <= instant + 5 && slow < 30,
            "{slow} frames, {instant} at Instant"
        );
    }
}
