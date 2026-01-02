use std::fmt;

/// Errors that can occur in game logic
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    /// Word list could not be loaded
    WordListLoadError(String),
    /// Word pool is empty
    EmptyWordPool,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::WordListLoadError(msg) => write!(f, "Failed to load word list: {}", msg),
            GameError::EmptyWordPool => write!(f, "Word pool is empty"),
        }
    }
}

impl std::error::Error for GameError {}

impl From<std::io::Error> for GameError {
    fn from(err: std::io::Error) -> Self {
        GameError::WordListLoadError(err.to_string())
    }
}
