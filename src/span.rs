use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}-{}", self.line, self.start, self.end)
    }
}

impl Span {
    pub const ZERO: Span = Span {
        line: 0,
        start: 0,
        end: 0,
    };

    pub fn merge(&self, other: &Span) -> Span {
        let line = self.line.min(other.line);
        let start = self.start.min(other.start);
        let end = self.end.max(other.end);

        Span { line, start, end }
    }
}
