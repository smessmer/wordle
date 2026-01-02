use ratatui::style::Color;

/// Wordle color scheme
#[derive(Debug, Clone)]
pub struct Theme {
    /// Correct letter in correct position (green)
    pub correct: Color,
    /// Correct letter in wrong position (yellow)
    pub wrong_position: Color,
    /// Letter not in word (gray)
    pub not_in_word: Color,
    /// Empty cell (dark gray)
    pub empty: Color,
    /// Text color
    pub text: Color,
    /// Background color
    pub background: Color,
    /// Border color
    pub border: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            correct: Color::Rgb(106, 170, 100),       // Wordle green #6aaa64
            wrong_position: Color::Rgb(201, 180, 88), // Wordle yellow #c9b458
            not_in_word: Color::Rgb(120, 124, 126),   // Wordle gray #787c7e
            empty: Color::Rgb(58, 58, 60),            // Dark gray #3a3a3c
            text: Color::White,
            background: Color::Rgb(18, 18, 19),       // Near black #121213
            border: Color::Rgb(58, 58, 60),           // Same as empty
        }
    }
}
