use std::{fmt::Display, io, sync::MutexGuard};

use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub(crate) struct StatusMessage {
    text: String,
    level: StatusLevel,
}

pub(crate) enum StatusLevel {
    Info,
    Error,
}

impl Display for StatusLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusLevel::Info => f.write_str("INFO"),
            StatusLevel::Error => f.write_str("ERROR"),
        }
    }
}

impl Default for StatusMessage {
    fn default() -> Self {
        Self { text: "".to_string(), level: StatusLevel::Info }
    }
}

impl StatusMessage {
    pub(crate) fn info(text: &str) -> Self {
        Self { text: text.to_string(), level: StatusLevel::Info }
    }
    pub(crate) fn error(text: &str) -> Self {
        Self { text: text.to_string(), level: StatusLevel::Error }
    }
}

pub(crate) fn draw(
    app: MutexGuard<crate::App>,
    frame: &mut Frame<CrosstermBackend<io::Stdout>>,
    area: Rect,
) {
    let status_str = format!("[{}] {}", app.status_message.level, app.status_message.text);
    let status_bar = Paragraph::new(status_str.as_str())
        .style(match app.status_message.level {
            StatusLevel::Error => Style::default().fg(Color::Red),
            _ => Style::default(),
        })
        .block(Block::default().borders(Borders::ALL).title("status"));
    frame.render_widget(status_bar, area);
}
