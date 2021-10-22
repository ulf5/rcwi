/// A simple example demonstrating how to handle user input. This is
/// a bit out of the scope of the library as it does not provide any
/// input handling out of the box. However, it may helps some to get
/// started.
///
/// This is a very simple example:
///   * A input box always focused. Every character you type is registered
///   here
///   * Pressing Backspace erases a character
///   * Pressing Enter pushes the current input in the history of previous
///   messages

#[allow(dead_code)]
mod util;

use crate::util::event::{Event, Events};
use editor_input::input_from_editor;
use std::{error::Error, io::{self, Write, stderr}, sync::{Arc, Mutex}};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use unicode_width::UnicodeWidthStr;

enum InputMode {
    Normal,
    Editing,
}

#[derive(PartialEq)]
enum Widget {
    LogGroups,
    Query,
    Results,
}

struct App {
    input: String,
    query_string: String,
    input_mode: InputMode,
    selected_widget: Widget,
    messages: Vec<String>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            query_string: String::new(),
            input_mode: InputMode::Normal,
            selected_widget: Widget::LogGroups,
            messages: Vec::new(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization

    let app = Arc::new(Mutex::new(App::default()));
    // Setup event handlers
    //

    let app_r = app.clone();
    let main_handle = std::thread::spawn(move || {
        let mut app = app_r.lock().unwrap();
        let stdout = io::stdout().into_raw_mode().unwrap();
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        let events = Events::new();

        loop {
            // Draw UI
            terminal
                .draw(|f| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(2)
                        .constraints(
                            [
                                Constraint::Length(3),
                                Constraint::Length(3),
                                Constraint::Min(1),
                            ]
                            .as_ref(),
                        )
                        .split(f.size());

                    let input = Paragraph::new(app.input.as_ref())
                        .style(match app.selected_widget {
                            Widget::LogGroups => Style::default().fg(Color::Yellow),
                            _ => Style::default(),
                        })
                        .block(Block::default().borders(Borders::ALL).title("Input"));
                    f.render_widget(input, chunks[0]);
                    match app.input_mode {
                        InputMode::Editing if app.selected_widget == Widget::LogGroups => {
                            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                            f.set_cursor(
                                // Put cursor past the end of the input text
                                chunks[0].x + app.input.width() as u16 + 1,
                                // Move one line down, from the border to the input line
                                chunks[0].y + 1,
                            )
                        }
                        _ =>
                            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
                            {}
                    }

                    let query = Paragraph::new(app.query_string.as_ref())
                        .style(match app.selected_widget {
                            Widget::Query => Style::default().fg(Color::Yellow),
                            _ => Style::default(),
                        })
                        .block(Block::default().borders(Borders::ALL).title("Query"));
                    f.render_widget(query, chunks[1]);

                    let messages: Vec<ListItem> = app
                        .messages
                        .iter()
                        .enumerate()
                        .map(|(i, m)| {
                            let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
                            ListItem::new(content)
                        })
                        .collect();
                    let messages = List::new(messages).block(
                        Block::default()
                            .style(match app.selected_widget {
                                Widget::Results => Style::default().fg(Color::Yellow),
                                _ => Style::default(),
                            })
                            .borders(Borders::ALL)
                            .title("Messages"),
                    );
                    f.render_widget(messages, chunks[2]);
                })
                .unwrap();

            // Handle input
            if let Event::Input(input) = events.next().unwrap() {
                match app.input_mode {
                    InputMode::Normal => match input {
                        Key::Char('i') | Key::Char('\n') => {
                            app.input_mode = InputMode::Editing;
                            match app.selected_widget {
                                Widget::Query => {
                                    app.query_string = input_from_editor().unwrap();
                                }
                                _ => {}
                            }
                        }
                        Key::Char('q') => {
                            break;
                        }
                        Key::Char('j') | Key::Down => match app.selected_widget {
                            Widget::LogGroups => {
                                app.selected_widget = Widget::Query;
                            }
                            Widget::Query => {
                                app.selected_widget = Widget::Results;
                            }
                            Widget::Results => {
                                app.selected_widget = Widget::LogGroups;
                            }
                        },
                        _ => {}
                    },
                    InputMode::Editing => match app.selected_widget {
                        Widget::LogGroups => match input {
                            Key::Char('\n') => {
                                let chars = app.input.drain(..).collect();
                                app.messages.push(chars);
                            }
                            Key::Char(c) => {
                                app.input.push(c);
                            }
                            Key::Backspace => {
                                app.input.pop();
                            }
                            Key::Esc => {
                                app.input_mode = InputMode::Normal;
                            }
                            _ => {
                                dbg!("HEJ");
                            }
                        },
                        _ => match input {
                            Key::Esc => {
                                app.input_mode = InputMode::Normal;
                            }
                            _ => {}
                        },
                    },
                }
            }
        }
        events.close();
    });
    main_handle.join().unwrap();

    println!("{}", input_from_editor()?);

    let app_r = app.clone();
    let main_handle = std::thread::spawn(move || {
        let mut app = app_r.lock().unwrap();
        let stdout = io::stdout().into_raw_mode().unwrap();
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        let events = Events::new();

        loop {
            // Draw UI
            terminal
                .draw(|f| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(2)
                        .constraints(
                            [
                                Constraint::Length(3),
                                Constraint::Length(3),
                                Constraint::Min(1),
                            ]
                            .as_ref(),
                        )
                        .split(f.size());

                    let input = Paragraph::new(app.input.as_ref())
                        .style(match app.selected_widget {
                            Widget::LogGroups => Style::default().fg(Color::Yellow),
                            _ => Style::default(),
                        })
                        .block(Block::default().borders(Borders::ALL).title("Input"));
                    f.render_widget(input, chunks[0]);
                    match app.input_mode {
                        InputMode::Editing if app.selected_widget == Widget::LogGroups => {
                            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                            f.set_cursor(
                                // Put cursor past the end of the input text
                                chunks[0].x + app.input.width() as u16 + 1,
                                // Move one line down, from the border to the input line
                                chunks[0].y + 1,
                            )
                        }
                        _ =>
                            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
                            {}
                    }

                    let query = Paragraph::new(app.query_string.as_ref())
                        .style(match app.selected_widget {
                            Widget::Query => Style::default().fg(Color::Yellow),
                            _ => Style::default(),
                        })
                        .block(Block::default().borders(Borders::ALL).title("Query"));
                    f.render_widget(query, chunks[1]);

                    let messages: Vec<ListItem> = app
                        .messages
                        .iter()
                        .enumerate()
                        .map(|(i, m)| {
                            let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
                            ListItem::new(content)
                        })
                        .collect();
                    let messages = List::new(messages).block(
                        Block::default()
                            .style(match app.selected_widget {
                                Widget::Results => Style::default().fg(Color::Yellow),
                                _ => Style::default(),
                            })
                            .borders(Borders::ALL)
                            .title("Messages"),
                    );
                    f.render_widget(messages, chunks[2]);
                })
                .unwrap();

            // Handle input
            if let Event::Input(input) = events.next().unwrap() {
                match app.input_mode {
                    InputMode::Normal => match input {
                        Key::Char('i') | Key::Char('\n') => {
                            app.input_mode = InputMode::Editing;
                            match app.selected_widget {
                                Widget::Query => {
                                    app.query_string = input_from_editor().unwrap();
                                }
                                _ => {}
                            }
                        }
                        Key::Char('q') => {
                            break;
                        }
                        Key::Char('j') | Key::Down => match app.selected_widget {
                            Widget::LogGroups => {
                                app.selected_widget = Widget::Query;
                            }
                            Widget::Query => {
                                app.selected_widget = Widget::Results;
                            }
                            Widget::Results => {
                                app.selected_widget = Widget::LogGroups;
                            }
                        },
                        _ => {}
                    },
                    InputMode::Editing => match app.selected_widget {
                        Widget::LogGroups => match input {
                            Key::Char('\n') => {
                                let chars = app.input.drain(..).collect();
                                app.messages.push(chars);
                            }
                            Key::Char(c) => {
                                app.input.push(c);
                            }
                            Key::Backspace => {
                                app.input.pop();
                            }
                            Key::Esc => {
                                app.input_mode = InputMode::Normal;
                            }
                            _ => {
                                dbg!("HEJ");
                            }
                        },
                        _ => match input {
                            Key::Esc => {
                                app.input_mode = InputMode::Normal;
                            }
                            _ => {}
                        },
                    },
                }
            }
        }
        events.close();
    });
    main_handle.join().unwrap();
    Ok(())
}
