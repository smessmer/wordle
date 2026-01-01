use std::fmt;
use std::io;

/// Errors that can occur when working with UniqueStringSet.
#[derive(Debug)]
pub enum UniqueStringSetError {
    /// An I/O error occurred while reading or writing.
    Io(io::Error),
}

impl fmt::Display for UniqueStringSetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for UniqueStringSetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
        }
    }
}

impl From<io::Error> for UniqueStringSetError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// Result type alias for UniqueStringSet operations.
pub type Result<T> = std::result::Result<T, UniqueStringSetError>;
