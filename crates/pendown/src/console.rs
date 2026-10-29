//! The console's input line (ADR 0006): what Enter does, history, and
//! definitions typed at the console.

/// What happens when the user presses Enter.
#[derive(Debug, PartialEq)]
pub enum Submit {
    /// Run this entry.
    Run(String),
    /// The entry isn't finished: a `TO` is waiting for its `END`.
    More,
}

#[derive(Default)]
pub struct ConsoleInput {
    /// Lines typed so far of an unfinished definition.
    pending: Vec<String>,
    history: Vec<String>,
    /// Where Up and Down have got to in `history`.
    recalling: Option<usize>,
}

/// Whether `text` starts a definition that hasn't ended: a line starting
/// with `TO`, and no `END` after it.
fn unfinished(text: &str) -> bool {
    let mut open = false;
    for line in text.lines() {
        let mut words = line.split_whitespace();
        if words.next().is_some_and(|w| w.eq_ignore_ascii_case("to")) {
            open = true;
        }
        if line
            .split_whitespace()
            .any(|w| w.eq_ignore_ascii_case("end"))
        {
            open = false;
        }
    }
    open
}

/// Whether an entry defines procedures, which the next Run will clear.
pub fn defines(text: &str) -> bool {
    text.lines().any(|line| {
        line.split_whitespace()
            .next()
            .is_some_and(|w| w.eq_ignore_ascii_case("to"))
    })
}

impl ConsoleInput {
    pub fn submit(&mut self, line: &str) -> Submit {
        self.recalling = None;
        self.pending.push(line.to_string());
        let text = self.pending.join("\n");
        if unfinished(&text) {
            return Submit::More;
        }
        self.pending.clear();
        if !text.trim().is_empty() && self.history.last() != Some(&text) {
            self.history.push(text.clone());
        }
        Submit::Run(text)
    }

    /// Whether a definition is part-way through being typed.
    pub fn continuing(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Up: the previous entry.
    pub fn older(&mut self) -> Option<&str> {
        let i = match self.recalling {
            None => self.history.len().checked_sub(1)?,
            Some(i) => i.saturating_sub(1),
        };
        self.recalling = Some(i);
        Some(&self.history[i])
    }

    /// Down: the next entry, or an empty line after the newest.
    pub fn newer(&mut self) -> Option<&str> {
        let i = self.recalling? + 1;
        if i >= self.history.len() {
            self.recalling = None;
            return Some("");
        }
        self.recalling = Some(i);
        Some(&self.history[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_line_entries_run_straight_away() {
        let mut c = ConsoleInput::default();
        assert_eq!(c.submit("FD 50"), Submit::Run("FD 50".into()));
        assert_eq!(
            c.submit("TO DOUBLE :n OUTPUT :n * 2 END"),
            Submit::Run("TO DOUBLE :n OUTPUT :n * 2 END".into())
        );
    }

    #[test]
    fn a_definition_waits_for_its_end() {
        let mut c = ConsoleInput::default();
        assert_eq!(c.submit("to box :n"), Submit::More);
        assert!(c.continuing());
        assert_eq!(c.submit("  repeat 4 [fd :n rt 90]"), Submit::More);
        assert_eq!(
            c.submit("end"),
            Submit::Run("to box :n\n  repeat 4 [fd :n rt 90]\nend".into())
        );
        assert!(!c.continuing());
    }

    #[test]
    fn history() {
        let mut c = ConsoleInput::default();
        assert_eq!(c.older(), None, "nothing yet");
        c.submit("FD 10");
        c.submit("RT 90");
        c.submit("RT 90");
        assert_eq!(c.older(), Some("RT 90"), "repeats are kept once");
        assert_eq!(c.older(), Some("FD 10"));
        assert_eq!(c.older(), Some("FD 10"), "stays at the oldest");
        assert_eq!(c.newer(), Some("RT 90"));
        assert_eq!(c.newer(), Some(""), "past the newest, an empty line");
        assert_eq!(c.newer(), None);
    }

    #[test]
    fn spotting_definitions() {
        assert!(defines("TO BOX :n\nEND"));
        assert!(defines("PRINT 1\nto f\nend"));
        assert!(!defines("BOX 50"));
        assert!(!defines("PRINT \"to"));
    }
}
