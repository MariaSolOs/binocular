use std::time::Duration;

use anyhow::Result;
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
            tui.render(&self.input)?;

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
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
