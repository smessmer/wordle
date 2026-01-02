use crate::constants::MAX_GUESSES;
use crate::feedback::GuessFeedback;
use crate::letter::Word;
use crate::word_pool::WordPool;

/// Configuration for a game
#[derive(Debug, Clone)]
pub struct GameConfig {
    /// Maximum number of guesses allowed
    pub max_guesses: usize,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            max_guesses: MAX_GUESSES,
        }
    }
}

/// Current state of the game
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameState {
    /// Game in progress
    Playing,
    /// Player won
    Won { guesses_used: usize },
    /// Player lost (exhausted all guesses)
    Lost,
}

/// Result of a guess attempt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuessResult {
    /// Guess accepted, here's the feedback
    Accepted(GuessFeedback),
    /// Word not in dictionary
    NotInWordList,
    /// Game already over
    GameOver,
    /// Invalid input (not 5 letters, non-alphabetic)
    InvalidInput,
}

/// The main game struct
#[derive(Debug, Clone)]
pub struct Game {
    secret: Word,
    guesses: Vec<GuessFeedback>,
    config: GameConfig,
    word_pool: WordPool,
}

impl Game {
    /// Create a new game with a random secret word
    pub fn new(word_pool: WordPool) -> Self {
        Self::with_config(word_pool, GameConfig::default())
    }

    /// Create with custom config
    pub fn with_config(word_pool: WordPool, config: GameConfig) -> Self {
        let secret = word_pool.random().clone();
        Self {
            secret,
            guesses: Vec::new(),
            config,
            word_pool,
        }
    }

    /// Create with specific secret (for testing)
    pub fn with_secret(word_pool: WordPool, secret: Word) -> Self {
        Self {
            secret,
            guesses: Vec::new(),
            config: GameConfig::default(),
            word_pool,
        }
    }

    /// Make a guess (string input for convenience)
    pub fn guess(&mut self, input: &str) -> GuessResult {
        match Word::parse(input) {
            Some(word) => self.guess_word(&word),
            None => GuessResult::InvalidInput,
        }
    }

    /// Make a guess with a pre-parsed Word
    pub fn guess_word(&mut self, word: &Word) -> GuessResult {
        // Check if game is already over
        if self.state() != GameState::Playing {
            return GuessResult::GameOver;
        }

        // Check if word is in the word list
        if !self.word_pool.contains(word) {
            return GuessResult::NotInWordList;
        }

        // Evaluate the guess
        let feedback = GuessFeedback::evaluate(word, &self.secret);
        self.guesses.push(feedback.clone());

        GuessResult::Accepted(feedback)
    }

    /// Current game state
    pub fn state(&self) -> GameState {
        // Check if the last guess was correct
        if self.guesses.last().is_some_and(|last| last.is_win()) {
            return GameState::Won {
                guesses_used: self.guesses.len(),
            };
        }

        // Check if we've used all guesses
        if self.guesses.len() >= self.config.max_guesses {
            return GameState::Lost;
        }

        GameState::Playing
    }

    /// All guesses made so far
    pub fn guesses(&self) -> &[GuessFeedback] {
        &self.guesses
    }

    /// Number of guesses remaining
    pub fn guesses_remaining(&self) -> usize {
        self.config.max_guesses.saturating_sub(self.guesses.len())
    }

    /// Current guess number (1-based, for display)
    pub fn current_guess_number(&self) -> usize {
        self.guesses.len() + 1
    }

    /// Get the secret word (only available after game ends)
    pub fn secret(&self) -> Option<&Word> {
        match self.state() {
            GameState::Playing => None,
            _ => Some(&self.secret),
        }
    }

    /// Check if a word is in the valid word list
    pub fn is_valid_word(&self, word: &Word) -> bool {
        self.word_pool.contains(word)
    }

    /// Get max guesses allowed
    pub fn max_guesses(&self) -> usize {
        self.config.max_guesses
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_pool() -> WordPool {
        WordPool::from_strings(vec![
            "hello".to_string(),
            "world".to_string(),
            "crane".to_string(),
            "slate".to_string(),
            "audio".to_string(),
        ])
    }

    #[test]
    fn test_win_first_guess() {
        let pool = test_pool();
        let mut game = Game::with_secret(pool, Word::parse("hello").unwrap());

        let result = game.guess("hello");
        assert!(matches!(result, GuessResult::Accepted(f) if f.is_win()));
        assert_eq!(game.state(), GameState::Won { guesses_used: 1 });
    }

    #[test]
    fn test_win_after_multiple_guesses() {
        let pool = test_pool();
        let mut game = Game::with_secret(pool, Word::parse("hello").unwrap());

        game.guess("world");
        game.guess("crane");
        let result = game.guess("hello");

        assert!(matches!(result, GuessResult::Accepted(f) if f.is_win()));
        assert_eq!(game.state(), GameState::Won { guesses_used: 3 });
    }

    #[test]
    fn test_lose_after_max_guesses() {
        let pool = test_pool();
        let mut game = Game::with_secret(pool, Word::parse("hello").unwrap());

        for _ in 0..MAX_GUESSES {
            game.guess("world");
        }

        assert_eq!(game.state(), GameState::Lost);
        assert_eq!(game.secret(), Some(&Word::parse("hello").unwrap()));
    }

    #[test]
    fn test_invalid_word() {
        let pool = test_pool();
        let mut game = Game::with_secret(pool, Word::parse("hello").unwrap());

        let result = game.guess("hi");
        assert_eq!(result, GuessResult::InvalidInput);

        let result = game.guess("12345");
        assert_eq!(result, GuessResult::InvalidInput);
    }

    #[test]
    fn test_word_not_in_list() {
        let pool = test_pool();
        let mut game = Game::with_secret(pool, Word::parse("hello").unwrap());

        let result = game.guess("zzzzz");
        assert_eq!(result, GuessResult::NotInWordList);
    }

    #[test]
    fn test_game_over_prevents_more_guesses() {
        let pool = test_pool();
        let mut game = Game::with_secret(pool, Word::parse("hello").unwrap());

        game.guess("hello"); // Win
        let result = game.guess("world");
        assert_eq!(result, GuessResult::GameOver);
    }

    #[test]
    fn test_guesses_remaining() {
        let pool = test_pool();
        let mut game = Game::with_secret(pool, Word::parse("hello").unwrap());

        assert_eq!(game.guesses_remaining(), MAX_GUESSES);
        game.guess("world");
        assert_eq!(game.guesses_remaining(), MAX_GUESSES - 1);
    }
}
