use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::ListItem,
};
use std::{collections::HashMap, iter};

/// Number of context lines kept before and after a matched line.
pub(crate) const CTX_LINES: u16 = 4;

/// A match returned by `ripgrep`.
#[derive(Clone)]
pub(crate) struct RgItem {
    filename: String,
    line_number: u16,
    matched_line: String,
    context: String,
}

impl RgItem {
    /// Creates a new `ripgrep` item builder.
    pub(crate) fn builder(
        filename: impl Into<String>,
        line_number: u16,
        matched_line: impl Into<String>,
    ) -> RgItemBuilder {
        RgItemBuilder {
            filename: filename.into(),
            line_number,
            matched_line: matched_line.into(),
            pre_context: Vec::with_capacity(CTX_LINES.into()),
            post_context: Vec::with_capacity(CTX_LINES.into()),
        }
    }

    /// Returns the line number of the match.
    pub(crate) fn line_number(&self) -> u16 {
        self.line_number
    }

    /// Returns the file name of the match.
    pub(crate) fn filename(&self) -> &str {
        &self.filename
    }

    /// Returns the matched line together with its surrounding context.
    pub(crate) fn context(&self) -> &str {
        &self.context
    }

    /// Returns a list item representing the match.
    pub fn as_list_item(&self) -> ListItem {
        ListItem::new(vec![Line::from(vec![
            Span::styled(&self.filename, Style::default().fg(Color::LightMagenta)),
            Span::styled(
                format!(" [{}]", self.line_number),
                Style::default().fg(Color::LightMagenta),
            ),
            Span::raw(&self.matched_line),
        ])])
    }
}

/// A builder for [RgItem]s.
pub(crate) struct RgItemBuilder {
    filename: String,
    line_number: u16,
    matched_line: String,
    pre_context: Vec<String>,
    post_context: Vec<String>,
}

impl RgItemBuilder {
    /// Adds context before the matched line to the [RgItem].
    pub(crate) fn add_pre_context(mut self, ctx: &HashMap<u16, &str>) -> Self {
        for line in self.line_number.saturating_sub(CTX_LINES)..self.line_number {
            if let Some(ctx_line) = ctx.get(&line) {
                self.pre_context.push(ctx_line.to_string());
            }
        }

        self
    }

    /// Adds context after the matched line to the [RgItem].
    pub(crate) fn add_post_context(mut self, ctx: &HashMap<u16, &str>) -> Self {
        for line in self.line_number + 1..=self.line_number + CTX_LINES {
            if let Some(ctx_line) = ctx.get(&line) {
                self.post_context.push(ctx_line.to_string());
            }
        }

        self
    }

    /// Builds the [RgItem].
    pub(crate) fn build(self) -> RgItem {
        let context = self
            .pre_context
            .into_iter()
            .chain(iter::once(self.matched_line.clone()))
            .chain(self.post_context.into_iter())
            .collect::<Vec<_>>()
            .join("\n");

        RgItem {
            filename: self.filename,
            line_number: self.line_number,
            matched_line: self.matched_line,
            context,
        }
    }
}
