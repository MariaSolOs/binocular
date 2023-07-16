use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::tui::Tui;

// TODO: Document.

pub struct App {
    input: Input,
}

impl App {
    pub fn new() -> Self {
        Self {
            input: Input::default(),
        }
    }

    pub fn run(&mut self, tui: &mut Tui) -> Result<()> {
        let timeout = Duration::from_millis(250);

        loop {
            tui.render(&self.input)
                .context("failed to render application window")?;

            if event::poll(timeout).context("failed to poll next terminal event")? {
                if let Event::Key(key) = event::read().context("failed to read terminal event")? {
                    match key.code {
                        // Exit the application.
                        KeyCode::Esc => return Ok(()),
                        _ => {
                            self.input.handle_event(&Event::Key(key));
                        }
                    }
                }
            }
        }
    }
}
