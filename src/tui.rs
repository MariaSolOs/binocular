use anyhow::{Context, Result};
use crossterm::terminal;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::io::{self, Stdout};
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
        let mut stdout = io::stdout();
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
    pub fn shutdown() {
        // Disable raw mode.
        if let Err(err) = terminal::disable_raw_mode() {
            eprintln!("Failed to disable raw mode: {}", err);
        }

        // Restore terminal properties.
        if let Err(err) = crossterm::execute!(io::stdout(), terminal::LeaveAlternateScreen) {
            eprintln!("Failed to leave alternate screen: {}", err);
        }
    }

    /// Renders the terminal's widgets.
    pub(crate) fn render(
        &mut self,
        input: &Input,
        results: Vec<ListItem>,
        preview: String,
        state: &mut ListState,
        show_help: bool,
    ) -> Result<()> {
        fn block(title: &str) -> Block {
            Block::default()
                .title(title)
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::LightCyan))
        }

        fn help_line<'a>(key: &'a str, desc: &'a str) -> Line<'a> {
            Line::from(vec![
                Span::styled(
                    format!("  {:<15}", key),
                    Style::default()
                        .fg(Color::LightCyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(desc),
            ])
        }

        self.terminal
            .draw(|f| {
                // Define the layout.
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Length(10),
                            Constraint::Min(20),
                            Constraint::Length(3),
                            Constraint::Length(1),
                        ]
                        .as_ref(),
                    )
                    .margin(1)
                    .split(f.size());

                f.render_widget(Paragraph::new(preview).block(block("Preview")), chunks[0]);

                f.render_stateful_widget(
                    List::new(results)
                        .block(block("Results"))
                        .highlight_symbol(">> ")
                        .highlight_style(Style::default().fg(Color::Yellow)),
                    chunks[1],
                    state,
                );

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

                // Help label.
                f.render_widget(
                    Paragraph::new("Help (?)")
                        .style(Style::default().fg(Color::LightCyan))
                        .alignment(Alignment::Right),
                    chunks[3],
                );

                if show_help {
                    // Show the help dialog.
                    let layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [
                                Constraint::Percentage(35),
                                Constraint::Max(8),
                                Constraint::Percentage(35),
                            ]
                            .as_ref(),
                        )
                        .split(f.size());
                    let chunk = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [
                                Constraint::Percentage(40),
                                Constraint::Min(40),
                                Constraint::Percentage(40),
                            ]
                            .as_ref(),
                        )
                        .split(layout[1])[1];
                    f.render_widget(Clear, chunk);
                    f.render_widget(
                        Paragraph::new(vec![
                            help_line("<esc>", "Quit"),
                            help_line("<up>", "Previous result"),
                            help_line("<down>", "Next result"),
                            help_line("<enter>", "Select result"),
                            help_line("?", "Toggle help"),
                        ])
                        .block(block("Help")),
                        chunk,
                    );
                }
            })
            .map(|_| ())
            .context("Failed to draw terminal")
    }
}
