use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::ListItem,
};
use std::{collections::HashMap, iter};

// TODO: Document.

pub(crate) const CTX_LINES: u16 = 4;

#[derive(Clone)]
pub struct RgItem {
    filename: String,
    line_number: u16,
    matched_line: String,
    context: String,
}

impl RgItem {
    pub fn builder(
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

    pub(crate) fn context(&self) -> &str {
        &self.context
    }

    pub fn into_list_item(self) -> ListItem<'static> {
        ListItem::new(vec![Line::from(vec![
            Span::styled(self.filename, Style::default().fg(Color::LightMagenta)),
            Span::styled(
                format!(" [{}]", self.line_number),
                Style::default().fg(Color::LightMagenta),
            ),
            Span::raw(self.matched_line),
        ])])
    }
}

pub struct RgItemBuilder {
    filename: String,
    line_number: u16,
    matched_line: String,
    pre_context: Vec<String>,
    post_context: Vec<String>,
}

impl RgItemBuilder {
    pub fn add_pre_context(mut self, ctx: &HashMap<u16, &str>) -> Self {
        for line in self.line_number.saturating_sub(CTX_LINES)..self.line_number {
            if let Some(ctx_line) = ctx.get(&line) {
                self.pre_context.push(ctx_line.to_string());
            }
        }

        self
    }

    pub fn add_post_context(mut self, ctx: &HashMap<u16, &str>) -> Self {
        for line in self.line_number + 1..=self.line_number + CTX_LINES {
            if let Some(ctx_line) = ctx.get(&line) {
                self.post_context.push(ctx_line.to_string());
            }
        }

        self
    }

    pub fn build(self) -> RgItem {
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
