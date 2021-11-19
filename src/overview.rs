use std::{io::Stdout, sync::mpsc::Sender};

use crate::{
    cwl::AwsReq,
    status_bar::{self, StatusMessage},
    time_select::{self, TimeSelector, TimeSelectorInput},
    Mode, SelectedView, Widget,
};
use crossterm::event::KeyCode;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub(crate) fn draw(
    app: std::sync::MutexGuard<crate::App>,
    frame: &mut Frame<CrosstermBackend<Stdout>>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(9),
                Constraint::Min(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(frame.size());

    let first_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(50)].as_ref())
        .split(chunks[0]);
    let selected_log_groups_string = app.selected_log_groups.join(", ");
    let log_groups = Paragraph::new(selected_log_groups_string.as_str())
        .style(match app.focused {
            Widget::LogGroups => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        })
        .block(Block::default().borders(Borders::ALL).title("log groups"));
    frame.render_widget(log_groups, first_chunk[0]);

    time_select::draw(&app, frame, first_chunk[1]);

    let log_groups = Paragraph::new(app.query.as_str())
        .style(match app.focused {
            Widget::Query => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        })
        .block(Block::default().borders(Borders::ALL).title("query"));
    frame.render_widget(log_groups, chunks[1]);

    let messages: Vec<ListItem> = app
        .results
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m.message)))];
            ListItem::new(content).style(
                if app.focused != Widget::LogGroupsResults || app.mode != Mode::Insert {
                    Style::default()
                } else if i == app.log_group_row {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default()
                },
            )
        })
        .collect();
    let messages = List::new(messages).block(
        Block::default()
            .style(match app.focused {
                Widget::LogRows => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .borders(Borders::ALL)
            .title("results"),
    );
    frame.render_widget(messages, chunks[2]);

    if app.time_selector.popup {
        let centered_rect = centered_rect(20, 20, frame.size());
        frame.render_widget(Clear, centered_rect); //this clears out the background
        let block = Block::default()
            .style(Style::default().fg(Color::Yellow))
            .title("Select time")
            .borders(Borders::ALL);

        let popup_parts = Layout::default()
            .direction(Direction::Horizontal)
            .margin(2)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(block.inner(centered_rect));

        frame.render_widget(block, centered_rect);
        let input_start = Paragraph::new(app.time_selector.selected_start_string.as_ref())
            .style(match app.time_selector.input {
                TimeSelectorInput::Start => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .block(Block::default().borders(Borders::ALL).title("start"));
        frame.render_widget(input_start, popup_parts[0]);
        if app.time_selector.input == TimeSelectorInput::Start {
            frame.set_cursor(
                popup_parts[0].x + app.time_selector.selected_start_string.width() as u16 + 1,
                popup_parts[0].y + 1,
            )
        }
        let input_end = Paragraph::new(app.time_selector.selected_end_string.as_ref())
            .style(match app.time_selector.input {
                TimeSelectorInput::End => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .block(Block::default().borders(Borders::ALL).title("end"));
        frame.render_widget(input_end, popup_parts[1]);
        if app.time_selector.input == TimeSelectorInput::End {
            frame.set_cursor(
                popup_parts[1].x + app.time_selector.selected_end_string.width() as u16 + 1,
                popup_parts[1].y + 1,
            )
        }
    }
    status_bar::draw(app, frame, chunks[3]);
}

pub(crate) fn handle_input(
    mut app: std::sync::MutexGuard<crate::App>,
    key_code: KeyCode,
    cwl: &Sender<AwsReq>,
) {
    match app.time_selector.popup {
        true => match key_code {
            KeyCode::Backspace => match app.time_selector.input {
                TimeSelectorInput::Start => {
                    app.time_selector.selected_start_string.pop();
                }
                TimeSelectorInput::End => {
                    app.time_selector.selected_end_string.pop();
                }
            },
            KeyCode::Tab => match app.time_selector.input {
                TimeSelectorInput::Start => {
                    app.time_selector.input = TimeSelectorInput::End;
                }
                TimeSelectorInput::End => {
                    app.time_selector.input = TimeSelectorInput::Start;
                }
            },
            KeyCode::Enter => {
                let new_time_selector = TimeSelector::from_strings(
                    &app.time_selector.selected_start_string,
                    &app.time_selector.selected_end_string,
                );
                if let Ok(new_time_selector) = new_time_selector {
                    app.time_selector = new_time_selector;
                    app.time_selector.popup = false;
                    app.status_message = StatusMessage::info("New time range selected");
                } else {
                    app.status_message = StatusMessage::error(new_time_selector.err().unwrap());
                }
            }
            KeyCode::Esc => {
                app.time_selector.popup = false;
            }
            KeyCode::Char(c) => match app.time_selector.input {
                TimeSelectorInput::Start => app.time_selector.selected_start_string.push(c),
                TimeSelectorInput::End => app.time_selector.selected_end_string.push(c),
            },
            _ => {}
        },
        false => match key_code {
            KeyCode::Enter => match app.focused {
                Widget::LogGroups => {
                    app.selected = SelectedView::LogGroups;
                    app.focused = Widget::LogGroups;
                    if app.log_groups.is_empty() {
                        cwl.send(AwsReq::ListLogGroups).unwrap();
                    }
                }
                Widget::Query => app.break_inner = true,
                Widget::TimeSelector => app.time_selector.popup = true,
                _ => {}
            },
            KeyCode::Char('h') | KeyCode::Left => match app.focused {
                Widget::LogGroups => {
                    app.focused = Widget::TimeSelector;
                }
                Widget::TimeSelector => {
                    app.focused = Widget::LogGroups;
                }
                _ => {}
            },
            KeyCode::Char('j') | KeyCode::Down => match app.focused {
                Widget::LogGroups | Widget::TimeSelector => {
                    app.focused = Widget::Query;
                }
                Widget::Query => {
                    app.focused = Widget::LogRows;
                }
                _ => {
                    app.focused = Widget::LogGroups;
                }
            },
            KeyCode::Char('k') | KeyCode::Up => match app.focused {
                Widget::LogGroups => {
                    app.focused = Widget::LogRows;
                }
                Widget::Query => {
                    app.focused = Widget::LogGroups;
                }
                Widget::TimeSelector => {
                    app.focused = Widget::LogRows;
                }
                Widget::LogRows => {
                    app.focused = Widget::Query;
                }
                _ => {
                    app.focused = Widget::LogGroups;
                }
            },
            KeyCode::Char('l') | KeyCode::Right => match app.focused {
                Widget::LogGroups => {
                    app.focused = Widget::TimeSelector;
                }
                Widget::TimeSelector => {
                    app.focused = Widget::LogGroups;
                }
                _ => {}
            },
            KeyCode::Char('r') => {
                cwl.send(AwsReq::RunQuery).unwrap();
            }
            _ => {}
        },
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}


pub(crate) struct QueryLogRow {
    pub(crate) message: String,
    pub(crate) timestamp: String,
    pub(crate) ptr: String,
}
