//! Drive Kame: the arrow keys move and turn Kame by steps, each press a
//! console entry, and presses of the same key in a row joining into one.
//! The parts that don't need a window.

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
