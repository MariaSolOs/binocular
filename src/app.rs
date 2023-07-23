use anyhow::{bail, Context, Result};
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use std::{collections::HashMap, io::ErrorKind};
use tokio::{
    process::Command,
    sync::mpsc::{self, Receiver, Sender},
};
use tokio_stream::StreamExt;
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{
    rg_item::{RgItem, RgItemBuilder, CTX_LINES},
    tui::Tui,
};

// TODO: Tune this?
const CHANNEL_CAPACITY: usize = 100;

/// The application state. Abstraction over what's displayed
/// in the TUI.
pub struct App {
    input: Input,
    results: Vec<RgItem>,
    state: ListState,
    show_help: bool,
    tx: Sender<Vec<RgItem>>,
    rx: Receiver<Vec<RgItem>>,
}

impl App {
    /// Initializes a new application.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(CHANNEL_CAPACITY);
        Self {
            input: Input::default(),
            results: Vec::new(),
            state: ListState::default(),
            show_help: false,
            tx,
            rx,
        }
    }

    /// Runs the application loop.
    pub async fn run(&mut self, tui: &mut Tui) -> Result<()> {
        let mut reader = EventStream::new();

        loop {
            // Render the terminal UI.
            tui.render(
                &self.input,
                self.results.iter().map(RgItem::as_list_item).collect(),
                &{ self.selected_item().map_or("", |item| item.context()) }.to_string(),
                &mut self.state,
                self.show_help,
            )
            .context("Failed to render application window")?;

            tokio::select! {
                Some(event) = reader.next() => {
                    if let Event::Key(key) = event.context("Failed to read terminal event")? {
                        if key.code == KeyCode::Esc {
                            // Exit the application.
                            break;
                        }

                        self.handle_key_event(key)?;
                    }
                }
                Some(results) = self.rx.recv() => self.handle_results(results),
                else => break
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        // Note that only some actions are enabled when showing the help dialog.
        match (key.code, self.show_help) {
            // Select the previous item from the results list.
            (KeyCode::Up, false) => {
                self.state.select(Some(self.state.selected().map_or(0, |i| {
                    if i == 0 {
                        self.results.len() - 1
                    } else {
                        i - 1
                    }
                })));
            }
            // Select the next item from the results list.
            (KeyCode::Down, false) => {
                self.state.select(Some(self.state.selected().map_or(0, |i| {
                    if i >= self.results.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                })));
            }
            (KeyCode::Enter, false) => {
                // Open the selected item in VS Code.
                if let Some(item) = self.selected_item() {
                    Command::new(if cfg!(windows) {
                        "code-insiders.cmd"
                    } else {
                        "code-insiders"
                    })
                    .arg("--goto")
                    .arg(format!("{}:{}", item.filename(), item.line_number()))
                    .spawn()
                    .context("Failed to open file in VS Code")?;
                }
            }
            (KeyCode::Char('?'), _) => {
                // Toggle the help window.
                self.show_help = !self.show_help;
            }
            // Handle any other key event as search input.
            (_, show_help) => {
                if !show_help {
                    self.input.handle_event(&Event::Key(key));

                    // Spawn a new ripgrep task.
                    let input = self.input.value().to_owned();
                    let tx = self.tx.clone();
                    tokio::spawn(async move {
                        let rg_items = execute_rg(&input)
                            .await
                            .context("Failed to execute ripgrep")?;

                        tx.send(rg_items)
                            .await
                            .context("Failed to send ripgrep results")
                    });
                }
            }
        }

        Ok(())
    }

    /// Sets the current search results and resets the list offset.
    fn handle_results(&mut self, results: Vec<RgItem>) {
        self.results = results;
        self.state = ListState::default().with_selected(if self.results.is_empty() {
            None
        } else {
            Some(0)
        });
    }

    /// Returns the currently selected [RgItem] (if any).
    fn selected_item(&self) -> Option<&RgItem> {
        if self.results.is_empty() {
            None
        } else {
            Some(&self.results[self.state.selected().unwrap_or(0)])
        }
    }
}

// Executes `ripgrep` with the given search input.
async fn execute_rg(input: &str) -> Result<Vec<RgItem>> {
    // Easy case.
    if input.is_empty() {
        return Ok(Vec::new());
    }

    match Command::new(if cfg!(windows) { "rg.exe" } else { "rg" })
        .arg(input)
        .arg("--color=never")
        .arg("--heading")
        .arg("--line-number")
        .arg("--smart-case")
        .arg("--no-context-separator")
        .arg(format!("--context={}", CTX_LINES))
        .output()
        .await
    {
        Err(err) => {
            if err.kind() == ErrorKind::NotFound {
                bail!("ripgrep is not installed");
            } else {
                bail!("Failed to run ripgrep: {}", err);
            }
        }
        Ok(output) => {
            // Split the results.
            let output = String::from_utf8_lossy(&output.stdout);
            let mut output = output.split('\n');

            // Parse each item, keeping track of the context lines around each match.
            let mut file = output
                .next()
                .context("first output line should be a file name")?;
            let mut ctx = HashMap::with_capacity(CTX_LINES as usize * 2);
            let mut builder: Option<RgItemBuilder> = None;
            let mut results = Vec::new();
            for output_line in output {
                if output_line.starts_with(|c: char| c.is_ascii_digit()) {
                    match output_line
                        .trim_start_matches(|c: char| c.is_ascii_digit())
                        .chars()
                        .next()
                    {
                        Some(c @ ('-' | ':')) => {
                            let (line_number, line) = output_line
                                .split_once(c)
                                .context("output line should contain the matched character")?;
                            let line_number = line_number
                                .parse::<u16>()
                                .context("output line should start with digits")?;

                            // Add the line to the context.
                            ctx.insert(line_number, line);

                            if c == ':' {
                                // We have a match.
                                if let Some(builder) = builder {
                                    // The current context is the post-context for the previous item
                                    // (if any).
                                    results.push(builder.add_post_context(&ctx).build());
                                }

                                // The current context is the pre-context for this item.
                                builder = Some(
                                    RgItem::builder(file, line_number, line).add_pre_context(&ctx),
                                );
                            }
                        }
                        // This is technically impossible because we're matching ripgrep's
                        // format, but we'll handle it anyway.
                        _ => bail!(
                            "expected a context or a matching line but found: {}",
                            output_line
                        ),
                    }
                } else if !output_line.is_empty() {
                    // Must be a line with the file name.
                    file = output_line;
                } else {
                    // Changing files, so clear the context.
                    ctx.clear();
                }
            }

            // Add the last item.
            if let Some(builder) = builder {
                results.push(builder.add_post_context(&ctx).build());
            }

            Ok(results)
        }
    }
}
