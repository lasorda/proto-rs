use crate::position::Position;

/// Comment represents one or more comment text lines, either C-style or C++ style.
#[derive(Debug, Clone)]
pub struct Comment {
    pub position: Position,
    /// Lines are comment text lines without prefixes //, ///, /* or suffix */
    pub lines: Vec<String>,
    /// True for /* ... */ style comments.
    pub c_style: bool,
    /// True if the comment starts with /// (triple slash).
    pub extra_slash: bool,
}

impl Comment {
    /// Create a new Comment from a raw comment literal (including // or /* */ markers).
    pub fn new(pos: Position, lit: &str) -> Self {
        let extra_slash = lit.starts_with("///");
        let is_c_style = lit.starts_with("/*") && lit.ends_with("*/");
        let lines = if is_c_style {
            let without_markers = lit.trim_start_matches('/').trim_start_matches('*');
            let without_markers = without_markers.trim_end_matches('/').trim_end_matches('*');
            without_markers
                .split('\n')
                .map(|s| s.to_string())
                .collect()
        } else {
            let trimmed = lit.trim_start_matches('/');
            trimmed.split('\n').map(|s| s.to_string()).collect()
        };
        Comment {
            position: pos,
            lines,
            c_style: is_c_style,
            extra_slash,
        }
    }

    /// Merge appends all lines from another comment.
    pub fn merge(&mut self, other: &Comment) {
        self.lines.extend(other.lines.iter().cloned());
        self.c_style = self.c_style || other.c_style;
    }

    /// Returns true if this comment has text on the given line number.
    pub fn has_text_on_line(&self, line: usize) -> bool {
        if self.lines.is_empty() {
            return false;
        }
        self.position.line <= line && line < self.position.line + self.lines.len()
    }

    /// Returns the first line, or empty string if no lines.
    pub fn message(&self) -> &str {
        self.lines.first().map(|s| s.as_str()).unwrap_or("")
    }
}
