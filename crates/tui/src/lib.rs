mod app;
mod input;
mod theme;
mod widgets;

use std::io::{self, stdout, Stdout};
use std::time::Duration;

use crossterm::{
    event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use wordle_game::load_german_wordlist;

use app::App;

type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Run the Wordle TUI application
pub fn run() -> io::Result<()> {
    // Load wordlist
    let word_pool = load_german_wordlist()?;

    // Setup terminal
    let mut terminal = setup_terminal()?;

    // Create app
    let mut app = App::new(word_pool);

    // Run main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    restore_terminal(&mut terminal)?;

    result
}

fn setup_terminal() -> io::Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Tui) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn run_app(terminal: &mut Tui, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| app.render(frame))?;

        // Poll for events with a timeout
        if event::poll(Duration::from_millis(100))? {
            let event = event::read()?;
            app.handle_event(event);
        }

        if app.should_quit() {
            return Ok(());
        }
    }
}
