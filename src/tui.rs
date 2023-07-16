use anyhow::{Context, Result};
use crossterm::terminal;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};
use std::io::Stdout;
use tui_input::Input;

/// Wrapper around the terminal user interface.
/// Responsible for its setup and shutdown.
pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    /// Sets up the terminal user interface.
    pub fn setup() -> Result<Self> {
        // Enable raw mode.
        terminal::enable_raw_mode().context("Failed to enable raw mode")?;

        // Configure terminal properties.
        let mut stdout = std::io::stdout();
        crossterm::execute!(stdout, terminal::EnterAlternateScreen)
            .context("Failed to enter alternate screen")?;

        // Initialize the terminal.
        Terminal::new(CrosstermBackend::new(stdout))
            .map(|terminal| Self { terminal })
            .context("Failed to create terminal")
    }

    /// Shuts down the terminal user interface.
    /// Note that this function won't stop when encountering an error,
    /// instead it will print the error to `stderr` and continue.
    pub fn shutdown(&mut self) {
        // Disable raw mode.
        if let Err(err) = terminal::disable_raw_mode() {
            eprintln!("Failed to disable raw mode: {}", err);
        }

        // Restore terminal properties.
        if let Err(err) =
            crossterm::execute!(self.terminal.backend_mut(), terminal::LeaveAlternateScreen)
        {
            eprintln!("Failed to leave alternate screen: {}", err);
        }
    }

    /// Renders the terminal's widgets.
    pub fn render(&mut self, input: &Input) -> Result<()> {
        fn block(title: &str) -> Block {
            Block::default()
                .title(title)
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::LightCyan))
        }

        self.terminal
            .draw(|f| {
                // Define the layout.
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage(40),
                            Constraint::Percentage(53),
                            Constraint::Percentage(7),
                        ]
                        .as_ref(),
                    )
                    .margin(1)
                    .split(f.size());

                f.render_widget(block("Preview"), chunks[0]);
                f.render_widget(block("Results"), chunks[1]);
                f.render_widget(
                    Paragraph::new(input.value()).block(block("Input")),
                    chunks[2],
                );

                // Keep the cursor in sync with the input field.
                let width = chunks[0].width - 2;
                let scroll = input.visual_scroll(width as usize);
                f.set_cursor(
                    chunks[2].x + ((input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
                    chunks[2].y + 1,
                );
            })
            .map(|_| ())
            .context("Failed to draw terminal")
    }
}
