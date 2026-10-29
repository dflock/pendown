/// Which text a span points into (ADR 0006).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SourceId {
    /// The editor buffer, for this run.
    Editor,
    /// A numbered console entry.
    Console(u32),
}

/// A byte range in a source, and the line it starts on, counting from 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub source: SourceId,
    pub start: u32,
    pub end: u32,
    pub line: u32,
}

impl Span {
    /// The span from the start of `self` to the end of `last`.
    pub fn to(self, last: Span) -> Span {
        Span {
            end: self.end.max(last.end),
            ..self
        }
    }

    pub fn range(self) -> std::ops::Range<usize> {
        self.start as usize..self.end as usize
    }
}
