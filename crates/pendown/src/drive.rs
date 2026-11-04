//! Drive Kame: the arrow keys move and turn Kame by steps, each press a
//! console entry, and presses of the same key in a row joining into one.
//! The parts that don't need a window.

use std::collections::VecDeque;

/// The steps a move can take, and a turn, with the ones driving starts with.
pub const MOVE_STEPS: [f64; 5] = [1.0, 5.0, 10.0, 25.0, 50.0];
pub const TURN_STEPS: [f64; 6] = [1.0, 5.0, 15.0, 30.0, 45.0, 90.0];
pub const FIRST_MOVE_STEP: f64 = 10.0;
pub const FIRST_TURN_STEP: f64 = 15.0;

/// How fast Kame plays while driving, whatever the speed slider says:
/// `FD 10` takes a tenth of a second.
pub const DRIVE_SPEED: f64 = 100.0;

/// An arrow key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Forward,
    Back,
    Left,
    Right,
}

impl Key {
    pub fn command(&self) -> &'static str {
        match self {
            Key::Forward => "FD",
            Key::Back => "BK",
            Key::Left => "LT",
            Key::Right => "RT",
        }
    }

    pub fn turns(&self) -> bool {
        matches!(self, Key::Left | Key::Right)
    }
}

/// The steps for moves and turns.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Steps {
    pub moves: f64,
    pub turns: f64,
}

impl Default for Steps {
    fn default() -> Steps {
        Steps {
            moves: FIRST_MOVE_STEP,
            turns: FIRST_TURN_STEP,
        }
    }
}

impl Steps {
    pub fn of(&self, key: Key) -> f64 {
        if key.turns() { self.turns } else { self.moves }
    }
}

/// The entry presses are joining into: which key, and how far so far.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Joined {
    pub key: Key,
    pub total: f64,
}

/// A number as Logo text: whole if it is.
pub fn number(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{n:.0}")
    } else {
        format!("{n}")
    }
}

impl Joined {
    /// The entry as it reads in the console: `FD 30`.
    pub fn text(&self) -> String {
        format!("{} {}", self.key.command(), number(self.total))
    }
}

/// What a press does.
#[derive(Clone, Debug, PartialEq)]
pub enum Press {
    /// Starts a new entry, which runs this.
    New { joined: Joined, run: String },
    /// Carries on the open entry, which now reads `joined.text()`: this
    /// press runs `run` without being an entry of its own.
    More { joined: Joined, run: String },
}

/// What pressing `key` does, with `open` the entry presses are joining
/// into, if any: the same key carries it on; any other starts a new one.
pub fn press(open: Option<Joined>, key: Key, steps: Steps) -> Press {
    let step = steps.of(key);
    let run = format!("{} {}", key.command(), number(step));
    match open {
        Some(joined) if joined.key == key => Press::More {
            joined: Joined {
                key,
                total: joined.total + step,
            },
            run,
        },
        _ => Press::New {
            joined: Joined { key, total: step },
            run,
        },
    }
}

/// What a key asks driving to do.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Input {
    Key(Key),
    /// Space: the pen up or down.
    Pen,
    /// Backspace or Ctrl+Z.
    Undo,
}

/// The keys down and the presses waiting to play. A key that comes down again
/// with no key-up between is the keyboard repeating it, which can be far
/// faster than Kame moves, so repeats never build up: at most one waits, and
/// letting go of the key takes it back. Separate presses all wait their turn.
/// This goes by key-ups as not every platform says which presses are repeats.
#[derive(Debug, Default)]
pub struct Presses {
    /// The names of the keys down.
    held: Vec<String>,
    queue: VecDeque<Input>,
    /// The repeat waiting, and the name of its key.
    again: Option<(String, Input)>,
}

impl Presses {
    /// `key` came down, asking for `input`, with a press still playing if
    /// `busy`. Returns what to play now, if anything.
    pub fn down(&mut self, key: &str, input: Input, busy: bool) -> Option<Input> {
        let repeat = self.held.iter().any(|k| k == key);
        if !repeat {
            self.held.push(key.to_string());
        }
        if repeat && matches!(input, Input::Pen | Input::Undo) {
            // Holding Space would only flip the pen over and over, and
            // holding Undo would soon take back everything.
            return None;
        }
        if !busy {
            return Some(input);
        }
        if !repeat {
            // The keyboard repeats only the last key down, so a repeat of an
            // earlier key that's still waiting is out of date.
            self.again = None;
            self.queue.push_back(input);
        } else if self.queue.is_empty() {
            self.again = Some((key.to_string(), input));
        }
        None
    }

    /// `key` came up.
    pub fn up(&mut self, key: &str) {
        self.held.retain(|k| k != key);
        if self.again.as_ref().is_some_and(|(k, _)| k == key) {
            self.again = None;
        }
    }

    /// The next press to play, now that the last has finished.
    pub fn next(&mut self) -> Option<Input> {
        self.queue
            .pop_front()
            .or_else(|| self.again.take().map(|(_, input)| input))
    }

    /// How many presses are waiting to play.
    pub fn waiting(&self) -> usize {
        self.queue.len() + usize::from(self.again.is_some())
    }
}

/// What Space runs: lifts the pen if it's down, and puts it down if not.
pub fn pen(up: bool) -> &'static str {
    if up { "PD" } else { "PU" }
}

/// What driving says about Kame under it: the entry, and Kame's heading.
pub fn caption(joined: Joined, heading: f64) -> String {
    let heading = heading.rem_euclid(360.0).round() % 360.0;
    format!("{} · heading {}", joined.text(), number(heading))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presses_of_the_same_key_join_up() {
        let steps = Steps::default();
        let first = press(None, Key::Forward, steps);
        let Press::New { joined, run } = first else {
            panic!("{first:?}")
        };
        assert_eq!(
            (joined.text(), run.as_str()),
            ("FD 10".to_string(), "FD 10")
        );
        let second = press(Some(joined), Key::Forward, steps);
        let Press::More { joined, run } = second else {
            panic!("{second:?}")
        };
        assert_eq!(
            (joined.text(), run.as_str()),
            ("FD 20".to_string(), "FD 10")
        );
        // Another key starts a new entry, even the opposite one.
        assert_eq!(
            press(Some(joined), Key::Back, steps),
            Press::New {
                joined: Joined {
                    key: Key::Back,
                    total: 10.0
                },
                run: "BK 10".into()
            }
        );
    }

    #[test]
    fn turns_use_the_turn_step() {
        let steps = Steps {
            moves: 50.0,
            turns: 90.0,
        };
        let Press::New { joined, .. } = press(None, Key::Right, steps) else {
            panic!()
        };
        let Press::More { joined, run } = press(Some(joined), Key::Right, steps) else {
            panic!()
        };
        assert_eq!(
            (joined.text(), run),
            ("RT 180".to_string(), "RT 90".to_string())
        );
        assert_eq!(steps.of(Key::Back), 50.0);
    }

    const UP: Input = Input::Key(Key::Forward);
    const RIGHT: Input = Input::Key(Key::Right);

    #[test]
    fn holding_a_key_never_builds_up_a_queue() {
        let mut presses = Presses::default();
        assert_eq!(presses.down("up", UP, false), Some(UP));
        for _ in 0..100 {
            assert_eq!(presses.down("up", UP, true), None);
        }
        assert_eq!(presses.waiting(), 1);
        // Still held when that press ends: the next plays straight away.
        assert_eq!(presses.next(), Some(UP));
        assert_eq!(presses.next(), None);
        // A repeat with nothing playing plays.
        assert_eq!(presses.down("up", UP, false), Some(UP));
    }

    #[test]
    fn letting_go_takes_back_the_repeat_waiting() {
        let mut presses = Presses::default();
        presses.down("up", UP, false);
        presses.down("up", UP, true);
        presses.up("up");
        assert_eq!(presses.waiting(), 0);
        assert_eq!(presses.next(), None);
    }

    #[test]
    fn separate_presses_all_wait_their_turn() {
        let mut presses = Presses::default();
        presses.down("up", UP, false);
        presses.up("up");
        for _ in 0..3 {
            assert_eq!(presses.down("up", UP, true), None);
            // Letting go doesn't take back a press.
            presses.up("up");
        }
        assert_eq!(presses.waiting(), 3);
        assert_eq!(
            [presses.next(), presses.next(), presses.next()],
            [Some(UP); 3]
        );
        assert_eq!(presses.next(), None);
    }

    #[test]
    fn a_press_goes_before_any_repeat() {
        let mut presses = Presses::default();
        presses.down("up", UP, false);
        presses.down("up", UP, true);
        // Another key: Up's repeat is out of date.
        presses.down("right", RIGHT, true);
        assert_eq!(presses.waiting(), 1);
        // Right's own repeat doesn't jump in behind its press.
        presses.down("right", RIGHT, true);
        assert_eq!(presses.waiting(), 1);
        assert_eq!(presses.next(), Some(RIGHT));
        // Letting go of Up doesn't touch Right's repeat.
        presses.down("right", RIGHT, true);
        presses.up("up");
        assert_eq!(presses.next(), Some(RIGHT));
    }

    #[test]
    fn holding_space_flips_the_pen_once() {
        let mut presses = Presses::default();
        assert_eq!(presses.down("space", Input::Pen, false), Some(Input::Pen));
        assert_eq!(presses.down("space", Input::Pen, false), None);
        assert_eq!(presses.down("space", Input::Pen, true), None);
        assert_eq!(presses.waiting(), 0);
        presses.up("space");
        assert_eq!(presses.down("space", Input::Pen, false), Some(Input::Pen));
    }

    #[test]
    fn holding_undo_takes_back_one_entry() {
        let mut presses = Presses::default();
        assert_eq!(
            presses.down("backspace", Input::Undo, false),
            Some(Input::Undo)
        );
        assert_eq!(presses.down("backspace", Input::Undo, false), None);
        assert_eq!(presses.down("backspace", Input::Undo, true), None);
        assert_eq!(presses.waiting(), 0);
        presses.up("backspace");
        assert_eq!(
            presses.down("backspace", Input::Undo, false),
            Some(Input::Undo)
        );
    }

    #[test]
    fn a_lost_key_up_costs_one_press_at_most() {
        let mut presses = Presses::default();
        presses.down("up", UP, false);
        // Its key-up never came. Tapping it again looks like a repeat, which
        // still plays with nothing else playing.
        assert_eq!(presses.down("up", UP, false), Some(UP));
        // Tapped while a press plays, it's lost when its key-up comes...
        assert_eq!(presses.down("up", UP, true), None);
        presses.up("up");
        assert_eq!(presses.waiting(), 0);
        // ...and that key-up puts things right.
        assert_eq!(presses.down("up", UP, true), None);
        presses.up("up");
        assert_eq!(presses.waiting(), 1);
    }

    #[test]
    fn words_for_the_console_and_the_canvas() {
        assert_eq!(number(2.5), "2.5");
        assert_eq!(number(45.0), "45");
        assert_eq!(pen(false), "PU");
        assert_eq!(pen(true), "PD");
        let joined = Joined {
            key: Key::Right,
            total: 45.0,
        };
        assert_eq!(caption(joined, 135.0), "RT 45 · heading 135");
        assert_eq!(caption(joined, -45.0), "RT 45 · heading 315");
        assert_eq!(caption(joined, 359.7), "RT 45 · heading 0");
    }
}
