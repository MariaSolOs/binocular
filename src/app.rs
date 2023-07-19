use anyhow::{bail, Context, Result};
use crossterm::event::{self, Event, KeyCode};
use ratatui::widgets::ListState;
use std::{collections::HashMap, io::ErrorKind, process::Command, time::Duration};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{
    rg_item::{RgItem, RgItemBuilder, CTX_LINES},
    tui::Tui,
};

// TODO: Document.

const TIMEOUT: u64 = 250;

pub struct App {
    input: Input,
    results: Vec<RgItem>,
    state: ListState,
}

impl App {
    pub fn new() -> Self {
        Self {
            input: Input::default(),
            results: Vec::new(),
            state: ListState::default(),
        }
    }

    pub fn run(&mut self, tui: &mut Tui) -> Result<()> {
        let timeout = Duration::from_millis(TIMEOUT);

        loop {
            tui.render(
                &self.input,
                self.results
                    .clone()
                    .into_iter()
                    .map(RgItem::into_list_item)
                    .collect(),
                &mut self.state,
            )
            .context("Failed to render application window")?;

            if event::poll(timeout).context("Failed to poll next terminal event")? {
                if let Event::Key(key) = event::read().context("Failed to read terminal event")? {
                    match key.code {
                        // Exit the application.
                        KeyCode::Esc => return Ok(()),
                        KeyCode::Up => {
                            self.state.select(Some(self.state.selected().map_or(0, |i| {
                                if i == 0 {
                                    self.results.len() - 1
                                } else {
                                    i - 1
                                }
                            })));
                        }
                        KeyCode::Down => {
                            self.state.select(Some(self.state.selected().map_or(0, |i| {
                                if i >= self.results.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            })));
                        }
                        _ => {
                            self.input.handle_event(&Event::Key(key));
                            self.execute_rg().context("Failed to execute ripgrep")?;
                        }
                    }
                }
            }
        }
    }

    fn execute_rg(&mut self) -> Result<()> {
        if self.input.value().is_empty() {
            // Easy case. Just clear the results.
            self.set_results(Vec::new());
            return Ok(());
        }

        match Command::new("rg")
            .arg(self.input.value())
            .arg("--color=never")
            .arg("--heading")
            .arg("--line-number")
            .arg("--smart-case")
            .arg("--no-context-separator")
            .arg(format!("--context={}", CTX_LINES))
            .output()
        {
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    bail!("ripgrep is not installed")
                } else {
                    bail!("Failed to run ripgrep: {}", err)
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
                let mut ctx = HashMap::with_capacity(8);
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

                                if c == '-' {
                                    // We have a context line.
                                    ctx.insert(line_number, line);
                                } else {
                                    // We have a match.
                                    if let Some(builder) = builder {
                                        // The current context is the post-context for the previous item
                                        // (if any).
                                        results.push(builder.add_post_context(&ctx).build());
                                    }

                                    // The current context is the pre-context for this item.
                                    builder = Some(
                                        RgItem::builder(file, line_number, line)
                                            .add_pre_context(&ctx),
                                    );
                                }
                            }
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

                // Update the current results.
                self.set_results(results);
            }
        }

        Ok(())
    }

    fn set_results(&mut self, results: Vec<RgItem>) {
        self.results = results;
        // Reset the stored offset.
        self.state = ListState::default().with_selected(if self.results.is_empty() {
            None
        } else {
            Some(0)
        });
    }
}
