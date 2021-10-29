use crossterm::{
    event::{poll, read, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
};
#[allow(dead_code)]
use editor_input::input_from_editor;
use std::sync::{Arc, Mutex};
use std::{error::Error, io::stdout, time::Duration};
use tui::{Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, style::{Color, Style}, text::{Span, Spans}, widgets::{Block, Borders, List, ListItem, Paragraph}};

#[derive(Clone, Copy)]
enum Widget {
    LogGroups,
    LogGroupsResults,
    Query,
    LogRows,
}

struct App {
    selected: Option<Widget>,
    focused: Widget,
    log_groups: Vec<String>,
    query: String,
}

impl Default for App {
    fn default() -> App {
        App {
            selected: Some(Widget::LogGroups),
            focused: Widget::LogGroups,
            log_groups: vec!["hej".to_string()],
            query: "hej".to_string(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = Arc::new(Mutex::new(App::default()));

    let app_r = app.clone();
    let main_handle = std::thread::spawn(move || {
        enable_raw_mode().unwrap();
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        loop {
            terminal
                .draw(|f| {
                    let app = app_r.lock().unwrap();
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

                    let input = Paragraph::new("")
                        .style(match app.focused {
                            Widget::LogGroups => Style::default().fg(Color::Yellow),
                            _ => Style::default(),
                        })
                        .block(Block::default().borders(Borders::ALL).title("log groups"));
                    f.render_widget(input, chunks[0]);
                    match app.selected {
                        Some(Widget::LogGroups | Widget::LogGroupsResults) => {
                            let messages: Vec<ListItem> = app
                                .log_groups
                                .iter()
                                .enumerate()
                                .map(|(i, m)| {
                                    let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
                                    ListItem::new(content)
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
                            f.render_widget(messages, chunks[2]);
                        }
                        _ => {},
                    }


                })
                .unwrap();

            if poll(Duration::from_millis(50)).unwrap() {
                let mut app = app_r.lock().unwrap();
                let event = read().unwrap();
                if let CEvent::Key(key_code) = event {
                    match key_code.code {
                        KeyCode::Enter => {
                            app.selected = Some(app.focused);
                        }
                        KeyCode::Esc => {
                            app.selected = None;
                        },
                        KeyCode::Char('q') => break,
                        _ => {}
                    }
                }
            }
        }
        disable_raw_mode().unwrap();
    });
    main_handle.join().unwrap();
    let app_g = app.lock().unwrap();

    println!("{}", input_from_editor(&app_g.query)?);

    Ok(())
}
