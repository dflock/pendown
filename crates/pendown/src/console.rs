//! The console's input line (ADR 0006): what Enter does, history, the
//! entries since the last Run for Undo (ADR 0025), and definitions typed at
//! the console.

/// What happens when the user presses Enter.
#[derive(Debug, PartialEq)]
pub enum Submit {
    /// Run this entry.
    Run(String),
    /// The entry isn't finished: a `TO` is waiting for its `END`.
    More,
}

/// How a console entry ended.
#[derive(Clone, Debug, PartialEq)]
pub enum Ending {
    Ran,
    /// It stopped with this error.
    Error(String),
    /// Stop was pressed.
    Stopped,
}

/// An entry run at the console since the last Run.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub text: String,
    /// How it ended, once it has.
    pub ending: Option<Ending>,
    pub undone: bool,
}

#[derive(Default)]
pub struct ConsoleInput {
    /// Lines typed so far of an unfinished definition.
    pending: Vec<String>,
    history: Vec<String>,
    /// Where Up and Down have got to in `history`.
    recalling: Option<usize>,
    /// Every entry run since the last Run, oldest first. Unlike `history`
    /// it keeps repeats, since each one moved Kame: one for each the
    /// interpreter saved Kame's state for.
    entries: Vec<Entry>,
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
        if !text.trim().is_empty() {
            if self.history.last() != Some(&text) {
                self.history.push(text.clone());
            }
            self.entries.push(Entry {
                text: text.clone(),
                ending: None,
                undone: false,
            });
        }
        Submit::Run(text)
    }

    /// Records how the newest entry ended.
    pub fn finish(&mut self, ending: Ending) {
        if let Some(entry) = self.entries.last_mut() {
            entry.ending = Some(ending);
        }
    }

    /// The entry Undo would take back: the newest not undone yet.
    pub fn to_undo(&self) -> Option<&Entry> {
        self.entries.iter().rev().find(|e| !e.undone)
    }

    /// Marks the newest entry not undone yet as undone, and returns it.
    pub fn undo(&mut self) -> Option<&Entry> {
        let entry = self.entries.iter_mut().rev().find(|e| !e.undone)?;
        entry.undone = true;
        Some(entry)
    }

    /// The entries Teach Kame lists: those not undone, oldest first.
    pub fn teachable(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(|e| !e.undone)
    }

    /// Run starts again with nothing to undo or teach. The history stays.
    pub fn clear_entries(&mut self) {
        self.entries.clear();
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

    fn texts<'a>(entries: impl Iterator<Item = &'a Entry>) -> Vec<&'a str> {
        entries.map(|e| e.text.as_str()).collect()
    }

    #[test]
    fn entries_keep_repeats_and_how_they_ended() {
        let mut c = ConsoleInput::default();
        c.submit("FD 100");
        c.finish(Ending::Ran);
        c.submit("FD 100");
        c.finish(Ending::Error("oops".into()));
        c.submit("");
        c.submit("to box");
        assert!(
            c.teachable().all(|e| e.text == "FD 100"),
            "only whole entries"
        );
        c.submit("end");
        assert_eq!(texts(c.teachable()), ["FD 100", "FD 100", "to box\nend"]);
        let endings: Vec<_> = c.teachable().map(|e| e.ending.clone()).collect();
        assert_eq!(
            endings,
            [Some(Ending::Ran), Some(Ending::Error("oops".into())), None]
        );
    }

    #[test]
    fn undo_takes_back_the_newest_first() {
        let mut c = ConsoleInput::default();
        assert_eq!(c.undo(), None, "nothing to undo");
        for line in ["FD 100", "RT 90", "RT 45"] {
            c.submit(line);
            c.finish(Ending::Ran);
        }
        assert_eq!(c.to_undo().map(|e| e.text.as_str()), Some("RT 45"));
        assert_eq!(c.undo().map(|e| e.text.as_str()), Some("RT 45"));
        assert_eq!(texts(c.teachable()), ["FD 100", "RT 90"]);
        c.submit("FD 50");
        assert_eq!(texts(c.teachable()), ["FD 100", "RT 90", "FD 50"]);
        assert_eq!(c.undo().map(|e| e.text.as_str()), Some("FD 50"));
        assert_eq!(c.undo().map(|e| e.text.as_str()), Some("RT 90"));
        assert_eq!(c.undo().map(|e| e.text.as_str()), Some("FD 100"));
        assert_eq!(c.undo(), None);
        assert_eq!(
            c.older(),
            Some("FD 50"),
            "undone entries stay in the history"
        );
    }

    #[test]
    fn run_clears_the_entries() {
        let mut c = ConsoleInput::default();
        c.submit("FD 10");
        c.clear_entries();
        assert_eq!(c.to_undo(), None);
        assert_eq!(c.teachable().count(), 0);
        assert_eq!(c.older(), Some("FD 10"));
    }

    #[test]
    fn spotting_definitions() {
        assert!(defines("TO BOX :n\nEND"));
        assert!(defines("PRINT 1\nto f\nend"));
        assert!(!defines("BOX 50"));
        assert!(!defines("PRINT \"to"));
    }
}
