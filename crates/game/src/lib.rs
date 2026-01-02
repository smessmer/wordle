pub mod constants;
pub mod error;
pub mod feedback;
pub mod game;
pub mod letter;
pub mod word_pool;
pub mod wordlists;

// Re-exports for convenience
pub use constants::{MAX_GUESSES, WORD_LENGTH};
pub use error::GameError;
pub use feedback::{GuessFeedback, LetterFeedback};
pub use game::{Game, GameConfig, GameState, GuessResult};
pub use letter::{Letter, Word};
pub use word_pool::{load_german_wordlist, WordPool};
