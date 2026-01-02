use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};
use std::collections::HashMap;
use wordle_game::{GuessFeedback, LetterFeedback};

use crate::theme::Theme;

/// Tracks the best feedback state for each letter
#[derive(Debug, Clone, Default)]
pub struct KeyboardState {
    letter_states: HashMap<char, LetterFeedback>,
}

impl KeyboardState {
    /// Create a new keyboard state
    pub fn new() -> Self {
        Self {
            letter_states: HashMap::new(),
        }
    }

    /// Update states based on a new guess feedback.
    /// Letters upgrade: NotInWord -> WrongPosition -> Correct
    pub fn update(&mut self, feedback: &GuessFeedback) {
        for (letter, fb) in feedback.iter() {
            let c = letter.char();
            let current = self.letter_states.get(&c).copied();
            let new_state = match (current, fb) {
                (None, fb) => fb,
                (Some(LetterFeedback::NotInWord), fb) => fb,
                (Some(LetterFeedback::WrongPosition), LetterFeedback::Correct) => {
                    LetterFeedback::Correct
                }
                (Some(current), _) => current,
            };
            self.letter_states.insert(c, new_state);
        }
    }

    /// Get the state of a letter
    pub fn get(&self, letter: char) -> Option<LetterFeedback> {
        self.letter_states.get(&letter.to_lowercase().next().unwrap_or(letter)).copied()
    }

    /// Clear all states (for new game)
    pub fn clear(&mut self) {
        self.letter_states.clear();
    }
}

/// Widget for rendering the virtual keyboard
pub struct KeyboardWidget<'a> {
    state: &'a KeyboardState,
    theme: &'a Theme,
}

impl<'a> KeyboardWidget<'a> {
    pub fn new(state: &'a KeyboardState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }
}

impl Widget for KeyboardWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // QWERTZ keyboard layout (German)
        let rows = [
            "qwertzuiop",
            "asdfghjkl",
            "yxcvbnm",
        ];

        let key_width = 3;
        let key_spacing = 1;

        let start_y = area.y;

        for (row_idx, row) in rows.iter().enumerate() {
            let row_width = row.len() as u16 * (key_width + key_spacing) - key_spacing;
            let row_x = area.x + (area.width.saturating_sub(row_width)) / 2;
            let y = start_y + row_idx as u16;

            if y >= area.y + area.height {
                continue;
            }

            for (col_idx, ch) in row.chars().enumerate() {
                let x = row_x + col_idx as u16 * (key_width + key_spacing);

                if x + key_width > area.x + area.width {
                    continue;
                }

                let bg_color = match self.state.get(ch) {
                    Some(LetterFeedback::Correct) => self.theme.correct,
                    Some(LetterFeedback::WrongPosition) => self.theme.wrong_position,
                    Some(LetterFeedback::NotInWord) => self.theme.not_in_word,
                    None => self.theme.empty,
                };

                let style = Style::default()
                    .fg(self.theme.text)
                    .bg(bg_color)
                    .add_modifier(Modifier::BOLD);

                // Draw key background
                for i in 0..key_width {
                    buf[(x + i, y)].set_style(style);
                }

                // Draw letter (centered)
                buf[(x + 1, y)]
                    .set_char(ch.to_uppercase().next().unwrap_or(ch))
                    .set_style(style);
            }
        }
    }
}
