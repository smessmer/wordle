use wordle_game::WORD_LENGTH;

/// State for the current text input
#[derive(Debug, Default, Clone)]
pub struct InputState {
    buffer: String,
}

impl InputState {
    /// Create a new empty input state
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Add a character to the input (if not full)
    pub fn push(&mut self, c: char) {
        if self.buffer.chars().count() < WORD_LENGTH && c.is_alphabetic() {
            self.buffer.push(c.to_lowercase().next().unwrap_or(c));
        }
    }

    /// Remove the last character
    pub fn pop(&mut self) {
        self.buffer.pop();
    }

    /// Clear the input
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get the current input as a string
    pub fn as_str(&self) -> &str {
        &self.buffer
    }

    /// Check if the input is complete (WORD_LENGTH letters)
    pub fn is_complete(&self) -> bool {
        self.buffer.chars().count() == WORD_LENGTH
    }
}
