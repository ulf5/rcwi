use std::{io::Stdout, sync::mpsc::Sender};

use crossterm::event::KeyCode;
use tui::{Frame, backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, style::{Color, Style}, text::{Span, Spans}, widgets::{Block, Borders, List, ListItem, Paragraph}};
use unicode_width::UnicodeWidthStr;
use crate::{Mode, Widget, cwl::AwsReq, view::View};


pub(crate) struct LogGroups;
impl View for LogGroups {
    fn draw(&self, app: std::sync::MutexGuard<crate::App>, frame: &mut Frame<CrosstermBackend<Stdout>>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                Constraint::Length(3),
                Constraint::Min(1),
                ]
                .as_ref(),
            )
            .split(frame.size());

        let input = Paragraph::new(app.log_filter.as_str())
            .style(match app.focused {
                Widget::LogGroups => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
        .block(Block::default().borders(Borders::ALL).title("log groups"));
        frame.render_widget(input, chunks[0]);
        if app.mode == Mode::Insert && app.focused == Widget::LogGroups {
            frame.set_cursor(
                chunks[0].x + app.log_filter.width() as u16 + 1,
                chunks[0].y + 1,
            )
        }

        let messages: Vec<ListItem> = app.log_groups.iter()
            .enumerate()
            .filter(|(i, _e)| app.filtered_log_groups.contains(i))
            .map(|(_i, x)| x)
            .enumerate()
            .map(|(i, m)| {
                let marker =  if i == 0 {'*'} else {' '};
                let content = vec![Spans::from(Span::raw(format!("[{}] {}: {}", marker, i , m)))];
                ListItem::new(content).style(
                    if app.focused != Widget::LogGroupsResults || app.mode != Mode::Insert {
                        Style::default()
                    } else if i == app.log_group_row {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default()
                    }
                )
            })
        .collect();
        let messages = List::new(messages).block(
            Block::default()
            .style(match app.focused {
                Widget::LogGroupsResults => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .borders(Borders::ALL)
            .title("results"),
        );
        frame.render_widget(messages, chunks[1]);
    }

    fn handle_input(&self, mut app: std::sync::MutexGuard<crate::App>, key_code: KeyCode, cwl: &Sender<AwsReq>) {
        match app.mode {
            Mode::Normal => {
                match key_code {
                    KeyCode::Enter => {
                        if app.log_groups.is_empty() {
                            cwl.send(AwsReq::ListLogGroups).unwrap();
                        }
                        app.mode = Mode::Insert;
                    }
                    KeyCode::Char('j') => {
                        match app.focused {
                            Widget::LogGroups => {app.focused = Widget::LogGroupsResults;}
                            _ => {app.focused = Widget::LogGroups;}
                        }
                    }
                    KeyCode::Char('k') => {
                        match app.focused {
                            Widget::LogGroups => {app.focused = Widget::LogGroupsResults;}
                            _ => {app.focused = Widget::LogGroups;}
                        }
                    }
                    _ => {}
                }
            }
            Mode::Insert => {
                match app.focused {
                    Widget::LogGroups => {
                        match key_code {
                            KeyCode::Esc => {
                                app.mode = Mode::Normal;
                            },
                            KeyCode::Enter => {
                                app.focused = Widget::LogGroupsResults;
                            }
                            KeyCode::Char(c) => {
                                app.log_filter.push(c);
                                filter_log_groups(&mut app);
                            },
                            KeyCode::Backspace => {
                                app.log_filter.pop();
                                filter_log_groups(&mut app);
                            },
                            _ => {},
                        }
                    },
                    Widget::LogGroupsResults => {
                        match key_code {
                            KeyCode::Esc => {
                                app.mode = Mode::Normal;
                            },
                            KeyCode::Char('j') => {
                                app.log_group_row = (app.log_group_row + 1) % app.filtered_log_groups.len();
                            }
                            KeyCode::Char('k') => {
                                let l = app.filtered_log_groups.len();
                                let r = app.log_group_row;
                                app.log_group_row = if r > 0 && l > 0 { (r - 1) % l } else if r == 0 && l > 0 { l - 1 } else { 0 };
                            }
                            _ => {}
                        }
                    },
                    _ => panic!("something very wrong")
                }
            },
        }
    }
}

pub(crate) fn filter_log_groups(app: &mut std::sync::MutexGuard<crate::App>) {
    let res = app.log_group_search_index.search(&app.log_filter);
    app.filtered_log_groups = app
        .log_groups
        .iter()
        .enumerate()
        .filter(|(i, _x)| res.is_empty() || res.contains(&i))
        .map(|(i, _)| i)
        .collect();
}
