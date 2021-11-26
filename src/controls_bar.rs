use std::{io};

use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub(crate) fn draw(
    frame: &mut Frame<CrosstermBackend<io::Stdout>>,
    area: Rect,
) {
    let controls = vec![
        "q (quit)",
        "hjkl/Arrows (control focus)",
        "Enter (select)",
        "Escape (go back)",
        "r (run the query)",
    ];
    let controls_bar = Paragraph::new(controls.join(" | "))
        .style(Style::default())
        .block(Block::default()
            .borders(Borders::NONE)
            .title("controls"));

    frame.render_widget(controls_bar, area);
}
