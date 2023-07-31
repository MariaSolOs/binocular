use anyhow::{bail, Context, Result};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::ListItem,
};
use std::{collections::HashMap, io::ErrorKind, iter};
use tokio::{process::Command, sync::mpsc::Sender};

use super::{Picker, PickerItem};

/// Number of context lines kept before and after a matched line.
const CTX_LINES: u16 = 4;

/// A `grep` match.
pub struct GrepItem {
    filename: String,
    line_number: u16,
    matched_line: String,
    context: String,
}

impl GrepItem {
    /// Creates a new `grep` item builder.
    fn builder(
        filename: impl Into<String>,
        line_number: u16,
        matched_line: impl Into<String>,
    ) -> GrepItemBuilder {
        GrepItemBuilder {
            filename: filename.into(),
            line_number,
            matched_line: matched_line.into(),
            pre_context: Vec::with_capacity(CTX_LINES.into()),
            post_context: Vec::with_capacity(CTX_LINES.into()),
        }
    }
}

impl PickerItem for GrepItem {
    fn as_list_item(&self) -> ListItem {
        ListItem::new(vec![Line::from(vec![
            Span::styled(&self.filename, Style::default().fg(Color::LightMagenta)),
            Span::styled(
                format!(" [{}]", self.line_number),
                Style::default().fg(Color::LightMagenta),
            ),
            Span::raw(&self.matched_line),
        ])])
    }

    fn preview(&self) -> String {
        self.context.to_owned()
    }
}

/// A builder for [GrepItem]s.
struct GrepItemBuilder {
    filename: String,
    line_number: u16,
    matched_line: String,
    pre_context: Vec<String>,
    post_context: Vec<String>,
}

impl GrepItemBuilder {
    /// Adds context before the matched line to the [GrepItem].
    fn add_pre_context(mut self, ctx: &HashMap<u16, &str>) -> Self {
        for line in self.line_number.saturating_sub(CTX_LINES)..self.line_number {
            if let Some(ctx_line) = ctx.get(&line) {
                self.pre_context.push(ctx_line.to_string());
            }
        }

        self
    }

    /// Adds context after the matched line to the [GrepItem].
    fn add_post_context(mut self, ctx: &HashMap<u16, &str>) -> Self {
        for line in self.line_number + 1..=self.line_number + CTX_LINES {
            if let Some(ctx_line) = ctx.get(&line) {
                self.post_context.push(ctx_line.to_string());
            }
        }

        self
    }

    /// Builds the [GrepItem].
    fn build(self) -> GrepItem {
        let context = self
            .pre_context
            .into_iter()
            .chain(iter::once(self.matched_line.clone()))
            .chain(self.post_context.into_iter())
            .collect::<Vec<_>>()
            .join("\n");

        GrepItem {
            filename: self.filename,
            line_number: self.line_number,
            matched_line: self.matched_line,
            context,
        }
    }
}

pub struct GrepPicker;

impl Picker<GrepItem> for GrepPicker {
    fn name(&self) -> &'static str {
        "Live Grep"
    }

    fn preview_title(&self) -> &'static str {
        "Grep Preview"
    }

    fn handle_input_change(&self, input: String, sender: Sender<Vec<GrepItem>>) {
        tokio::spawn(async move {
            let results = if input.is_empty() {
                Vec::new()
            } else {
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
                        let mut builder: Option<GrepItemBuilder> = None;
                        let mut results = Vec::new();
                        for output_line in output {
                            if output_line.starts_with(|c: char| c.is_ascii_digit()) {
                                match output_line
                                    .trim_start_matches(|c: char| c.is_ascii_digit())
                                    .chars()
                                    .next()
                                {
                                    Some(c @ ('-' | ':')) => {
                                        let (line_number, line) =
                                            output_line.split_once(c).context(
                                                "output line should contain the matched character",
                                            )?;
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
                                                results
                                                    .push(builder.add_post_context(&ctx).build());
                                            }

                                            // The current context is the pre-context for this item.
                                            builder = Some(
                                                GrepItem::builder(file, line_number, line)
                                                    .add_pre_context(&ctx),
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
                        results
                    }
                }
            };

            // Send the results to the application.
            sender
                .send(results)
                .await
                .context("Failed to send grep results")
        });
    }

    fn handle_selection(&self, item: &GrepItem) -> Result<()> {
        // Open the `grep` match in VS Code.
        Command::new(if cfg!(windows) {
            "code-insiders.cmd"
        } else {
            "code-insiders"
        })
        .arg("--goto")
        .arg(format!("{}:{}", item.filename, item.line_number))
        .spawn()
        .context("Failed to open file in VS Code")
        .map(|_| ())
    }
}
