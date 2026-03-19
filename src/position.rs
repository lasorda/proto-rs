use std::fmt;

/// Position describes a source position including file, line, and column.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Position {
    pub filename: String,
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.filename.is_empty() {
            write!(f, "<input>:{}:{}", self.line, self.column)
        } else {
            write!(f, "{}:{}:{}", self.filename, self.line, self.column)
        }
    }
}
