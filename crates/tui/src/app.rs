use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Paragraph},
    Frame,
};
use wordle_game::{Game, GameState, GuessResult, WordPool};

use crate::input::InputState;
use crate::theme::Theme;
use crate::widgets::{BoardWidget, KeyboardState, KeyboardWidget};

/// Main application state
pub struct App {
    game: Game,
    word_pool: WordPool,
    input: InputState,
    keyboard_state: KeyboardState,
    message: Option<String>,
    should_quit: bool,
    theme: Theme,
}

impl App {
    /// Create a new app with the given word pool
    pub fn new(word_pool: WordPool) -> Self {
        let game = Game::new(word_pool.clone());
        Self {
            game,
            word_pool,
            input: InputState::new(),
            keyboard_state: KeyboardState::new(),
            message: None,
            should_quit: false,
            theme: Theme::default(),
        }
    }

    /// Check if the app should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Handle an input event
    pub fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            self.handle_key(key);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // Clear message on any key press
        self.message = None;

        // Handle quit shortcuts
        if key.code == KeyCode::Esc
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            self.should_quit = true;
            return;
        }

        match self.game.state() {
            GameState::Playing => self.handle_playing_key(key),
            GameState::Won { .. } | GameState::Lost => self.handle_game_over_key(key),
        }
    }

    fn handle_playing_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) if c.is_alphabetic() => {
                self.input.push(c);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Enter => {
                if self.input.is_complete() {
                    self.submit_guess();
                } else {
                    self.message = Some("Not enough letters".to_string());
                }
            }
            _ => {}
        }
    }

    fn handle_game_over_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Enter {
            self.new_game();
        }
    }

    fn submit_guess(&mut self) {
        let input = self.input.as_str().to_string();
        match self.game.guess(&input) {
            GuessResult::Accepted(feedback) => {
                self.keyboard_state.update(&feedback);
                self.input.clear();
            }
            GuessResult::NotInWordList => {
                self.message = Some("Not in word list".to_string());
            }
            GuessResult::InvalidInput => {
                self.message = Some("Invalid input".to_string());
            }
            GuessResult::GameOver => {
                self.message = Some("Game is over".to_string());
            }
        }
    }

    fn new_game(&mut self) {
        self.game = Game::new(self.word_pool.clone());
        self.input.clear();
        self.keyboard_state.clear();
        self.message = None;
    }

    /// Render the app to the frame
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Clear background
        let block = Block::default().style(Style::default().bg(self.theme.background));
        frame.render_widget(block, area);

        // Layout: title, board, message, keyboard, help
        let chunks = Layout::vertical([
            Constraint::Length(2),  // Title
            Constraint::Length(8),  // Board (6 rows + padding)
            Constraint::Length(2),  // Message
            Constraint::Length(5),  // Keyboard (3 rows + padding)
            Constraint::Min(1),     // Help text
        ])
        .split(area);

        self.render_title(frame, chunks[0]);
        self.render_board(frame, chunks[1]);
        self.render_message(frame, chunks[2]);
        self.render_keyboard(frame, chunks[3]);
        self.render_help(frame, chunks[4]);
    }

    fn render_title(&self, frame: &mut Frame, area: Rect) {
        let title = Paragraph::new("WORDLE")
            .style(
                Style::default()
                    .fg(self.theme.text)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(title, area);
    }

    fn render_board(&self, frame: &mut Frame, area: Rect) {
        let board = BoardWidget::new(&self.game, self.input.as_str(), &self.theme);
        frame.render_widget(board, area);
    }

    fn render_message(&self, frame: &mut Frame, area: Rect) {
        let text = match self.game.state() {
            GameState::Won { guesses_used } => {
                format!("You won in {} guess{}! Press Enter to play again.",
                    guesses_used,
                    if guesses_used == 1 { "" } else { "es" }
                )
            }
            GameState::Lost => {
                format!(
                    "Game over! The word was {}. Press Enter to play again.",
                    self.game.secret().map(|w| w.to_string().to_uppercase()).unwrap_or_default()
                )
            }
            GameState::Playing => {
                self.message.clone().unwrap_or_default()
            }
        };

        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(self.theme.text))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(paragraph, area);
    }

    fn render_keyboard(&self, frame: &mut Frame, area: Rect) {
        let keyboard = KeyboardWidget::new(&self.keyboard_state, &self.theme);
        frame.render_widget(keyboard, area);
    }

    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let help = Paragraph::new("Type letters to guess | Backspace to delete | Enter to submit | Esc to quit")
            .style(Style::default().fg(self.theme.not_in_word))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(help, area);
    }
}
