#[allow(dead_code)]

use editor_input::input_from_editor;
use std::{error::Error, io::{self, stdout}, sync::{Arc, Mutex}, time::Duration};
use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode, poll, read}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};


struct App {
}

impl Default for App {
    fn default() -> App {
        App {
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = Arc::new(Mutex::new(App::default()));

    let app_r = app.clone();
    let main_handle = std::thread::spawn(move || {
        let mut app = app_r.lock().unwrap();
        enable_raw_mode().unwrap();
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();


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
                    }).unwrap();

            if poll(Duration::from_millis(1_000)).unwrap() {
                // It's guaranteed that read() wont block if `poll` returns `Ok(true)`
                let event = read().unwrap();

                if event == CEvent::Key(KeyCode::Char('q').into()) {
                    break;
                }

            }
        }
        disable_raw_mode().unwrap();
    });
    let a = main_handle.join().unwrap();
    drop(a);

    println!("{}", input_from_editor()?);

    Ok(())
}
