use anyhow::{Context, Result};
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::StreamExt;
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{
    pickers::{Picker, PickerItem},
    tui::Tui,
};

// TODO: Tune this?
const CHANNEL_CAPACITY: usize = 100;

/// The application state. Abstraction over what's displayed
/// in the TUI.
pub struct App<I, P>
where
    I: PickerItem,
    P: Picker<I>,
{
    picker: P,
    input: Input,
    results: Vec<I>,
    state: ListState,
    show_help: bool,
}

impl<I, P> App<I, P>
where
    I: PickerItem,
    P: Picker<I>,
{
    /// Initializes a new application.
    pub fn new(picker: P) -> Self {
        Self {
            picker,
            input: Input::default(),
            results: Vec::new(),
            state: ListState::default(),
            show_help: false,
        }
    }

    /// Runs the application loop.
    pub async fn run(&mut self, tui: &mut Tui<'_>) -> Result<()> {
        let mut reader = EventStream::new();
        let (tx, mut rx) = mpsc::channel(CHANNEL_CAPACITY);

        loop {
            // Render the terminal UI.
            tui.render(
                &self.input,
                &self.results,
                &mut self.state,
                self.show_help,
                self.picker.preview_title(),
                self.picker.name(),
            )
            .context("Failed to render application window")?;

            tokio::select! {
                Some(event) = reader.next() => {
                    if let Event::Key(key) = event.context("Failed to read terminal event")? {
                        if key.code == KeyCode::Esc {
                            // Exit the application.
                            break;
                        }

                        self.handle_key_event(key, tx.clone()).context("Failed to handle key event")?;
                    }
                }
                // Received something from the picker, update the results.
                Some(results) = rx.recv() => self.handle_results(results),
                else => break
            }
        }

        Ok(())
    }

    /// Updates the UI based on the key press.
    fn handle_key_event(&mut self, key: KeyEvent, tx: Sender<Vec<I>>) -> Result<()> {
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
                // Handle the selection.
                if let Some(item) = self.results.get(self.state.selected().unwrap_or(0)) {
                    self.picker
                        .handle_selection(item)
                        .context("Failed to process selected item")?;
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
                    self.picker
                        .handle_input_change(self.input.value().to_owned(), tx);
                }
            }
        }

        Ok(())
    }

    /// Sets the current search results and resets the list offset.
    fn handle_results(&mut self, results: Vec<I>) {
        self.results = results;
        self.state = ListState::default().with_selected(if self.results.is_empty() {
            None
        } else {
            Some(0)
        });
    }
}
