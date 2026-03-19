use crate::position::Position;
use std::fmt;

/// Error type for proto parsing errors.
#[derive(Debug, Clone)]
pub struct ProtoError {
    pub position: Position,
    pub message: String,
}

impl fmt::Display for ProtoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.position, self.message)
    }
}

impl std::error::Error for ProtoError {}

/// Convenience Result alias.
pub type Result<T> = std::result::Result<T, ProtoError>;
