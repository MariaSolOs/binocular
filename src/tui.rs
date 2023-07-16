use anyhow::{Context, Result};
use crossterm::terminal;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::Stdout;

/// Wrapper around the terminal user interface.
/// Responsible for its setup and shutdown.
pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    /// Sets up the terminal user interface.
    pub fn setup() -> Result<Self> {
        // Enable raw mode.
        terminal::enable_raw_mode().context("failed to enable raw mode")?;

        // Configure terminal properties.
        let mut stdout = std::io::stdout();
        crossterm::execute!(stdout, terminal::EnterAlternateScreen)
            .context("failed to enter alternate screen")?;

        // Initialize the terminal.
        let terminal =
            Terminal::new(CrosstermBackend::new(stdout)).context("failed to create terminal")?;

        Ok(Self { terminal })
    }

    pub fn shutdown(&mut self) -> Result<()> {
        // Disable raw mode.
        terminal::disable_raw_mode().context("failed to disable raw mode")?;

        // Restore terminal properties.
        crossterm::execute!(self.terminal.backend_mut(), terminal::LeaveAlternateScreen)
            .context("failed to leave alternate screen")?;

        Ok(())
    }
}
