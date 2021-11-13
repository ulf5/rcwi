use crossterm::{
    event::{poll, read, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
};
#[allow(dead_code)]
use editor_input::input_from_editor;
use indicium::simple::SearchIndex;
use std::sync::{Arc, Mutex, mpsc::{Receiver, Sender}};

use std::{error::Error, io::stdout, time::Duration};
use tui::{Terminal, backend::CrosstermBackend};

use crate::cwl::AwsReq;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Widget {
    LogGroups,
    LogGroupsResults,
    Query,
    LogRows,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SelectedView {
    LogGroups,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Insert
}

struct App {
    selected: SelectedView,
    focused: Widget,
    log_groups: Vec<String>,
    filtered_log_groups: Vec<usize>,
    selected_log_groups: Vec<String>,
    log_group_search_index: SearchIndex<usize>,
    log_group_row: usize,
    query: String,
    log_filter: String,
    mode: Mode,
}

impl Default for App {
    fn default() -> App {
        App {
            selected: SelectedView::LogGroups,
            focused: Widget::LogGroups,
            log_groups: vec![],
            filtered_log_groups: vec![],
            selected_log_groups: vec![],
            log_group_search_index: SearchIndex::default(),
            log_group_row: 0usize,
            query: "hej".to_string(),
            log_filter: "".to_string(),
            mode: Mode::Normal,
        }
    }
}
mod cwl;
mod log_groups;


fn main() -> Result<(), Box<dyn Error>> {
    let app = Arc::new(Mutex::new(App::default()));

    let (tx, rx): (Sender<AwsReq>, Receiver<AwsReq>)  = std::sync::mpsc::channel();

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
                    match app.selected {
                        SelectedView::LogGroups => log_groups::draw(app, f),
                    };
                })
                .unwrap();

            if poll(Duration::from_millis(50)).unwrap() {
                let event = read().unwrap();
                let app = app_r.lock().unwrap();
                if let CEvent::Key(key_code) = event {
                    match key_code.code {
                        KeyCode::Char('q') if app.mode == Mode::Normal => break,
                        k => {
                            match app.selected {
                                SelectedView::LogGroups => log_groups::handle_input(app, k, &tx),
                            };
                        }
                    }
                }
            }
        }
        disable_raw_mode().unwrap();
    });

    let app_r = app.clone();
    let _cwl_thread = std::thread::spawn(move || {
        cwl::run(app_r, rx);
    });

    main_handle.join().unwrap();
    let app_g = app.lock().unwrap();

    println!("{}", input_from_editor(&app_g.query)?);

    Ok(())
}
