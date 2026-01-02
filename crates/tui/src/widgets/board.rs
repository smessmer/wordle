use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};
use wordle_game::{Game, LetterFeedback, MAX_GUESSES, WORD_LENGTH};

use crate::theme::Theme;

/// Widget for rendering the Wordle game board
pub struct BoardWidget<'a> {
    game: &'a Game,
    current_input: &'a str,
    theme: &'a Theme,
}

impl<'a> BoardWidget<'a> {
    pub fn new(game: &'a Game, current_input: &'a str, theme: &'a Theme) -> Self {
        Self {
            game,
            current_input,
            theme,
        }
    }

    fn feedback_to_bg_color(&self, feedback: LetterFeedback) -> ratatui::style::Color {
        match feedback {
            LetterFeedback::Correct => self.theme.correct,
            LetterFeedback::WrongPosition => self.theme.wrong_position,
            LetterFeedback::NotInWord => self.theme.not_in_word,
        }
    }
}

impl Widget for BoardWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Each cell is 3 chars wide, 1 char tall, with 1 char spacing
        let cell_width = 3;
        let cell_spacing = 1;
        let total_width = WORD_LENGTH as u16 * (cell_width + cell_spacing) - cell_spacing;
        let total_height = MAX_GUESSES as u16;

        // Center the board in the area
        let start_x = area.x + (area.width.saturating_sub(total_width)) / 2;
        let start_y = area.y + (area.height.saturating_sub(total_height)) / 2;

        let guesses = self.game.guesses();

        for row in 0..MAX_GUESSES {
            for col in 0..WORD_LENGTH {
                let x = start_x + col as u16 * (cell_width + cell_spacing);
                let y = start_y + row as u16;

                if x + cell_width > area.x + area.width || y >= area.y + area.height {
                    continue;
                }

                let (letter, style) = if row < guesses.len() {
                    // Completed guess row
                    let feedback = &guesses[row];
                    let letter = feedback.word().letter(col).char();
                    let fb = feedback.feedback()[col];
                    let bg = self.feedback_to_bg_color(fb);
                    let style = Style::default()
                        .fg(self.theme.text)
                        .bg(bg)
                        .add_modifier(Modifier::BOLD);
                    (Some(letter), style)
                } else if row == guesses.len() {
                    // Current input row
                    let input_chars: Vec<char> = self.current_input.chars().collect();
                    let letter = input_chars.get(col).copied();
                    let style = Style::default()
                        .fg(self.theme.text)
                        .bg(self.theme.empty)
                        .add_modifier(Modifier::BOLD);
                    (letter, style)
                } else {
                    // Empty row
                    let style = Style::default().fg(self.theme.border).bg(self.theme.empty);
                    (None, style)
                };

                // Draw the cell background
                for i in 0..cell_width {
                    buf[(x + i, y)].set_style(style);
                }

                // Draw the letter (centered in the cell)
                if let Some(ch) = letter {
                    buf[(x + 1, y)]
                        .set_char(ch.to_uppercase().next().unwrap_or(ch))
                        .set_style(style);
                }
            }
        }
    }
}
