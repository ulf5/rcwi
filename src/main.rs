use crossterm::{
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
#[allow(dead_code)]
use editor_input::input_from_editor;
use flexi_logger::{FileSpec, Logger};
use indicium::simple::SearchIndex;
use overview::QueryLogRow;
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};

use std::{error::Error, io::stdout, time::Duration};
use tui::{backend::CrosstermBackend, Terminal};

use crate::{cwl::AwsReq, status_bar::StatusMessage, time_select::TimeSelector};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Widget {
    LogGroups,
    LogGroupsResults,
    Query,
    LogRows,
    TimeSelector,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SelectedView {
    Overview,
    LogGroups,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Insert,
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
    results: Vec<QueryLogRow>,
    log_filter: String,
    mode: Mode,
    break_inner: bool,
    quit: bool,
    status_message: StatusMessage,
    time_selector: TimeSelector,
}

impl Default for App {
    fn default() -> App {
        App {
            selected: SelectedView::Overview,
            focused: Widget::LogGroups,
            log_groups: vec![],
            filtered_log_groups: vec![],
            selected_log_groups: vec![],
            log_group_search_index: SearchIndex::default(),
            log_group_row: 0usize,
            query: "fields @timestamp, @message\n\
        | sort @timestamp desc\n"
                .to_string(),
            log_filter: "".to_string(),
            mode: Mode::Normal,
            break_inner: false,
            quit: false,
            results: vec![],
            status_message: StatusMessage::default(),
            time_selector: TimeSelector::default(),
        }
    }
}
mod cwl;
mod log_groups;
mod overview;
mod status_bar;
mod time_select;

fn main() -> Result<(), Box<dyn Error>> {
    let app = Arc::new(Mutex::new(App::default()));
    Logger::try_with_str("info")?.log_to_file(FileSpec::default().suppress_timestamp()).start()?;
    let (tx, rx): (Sender<AwsReq>, Receiver<AwsReq>) = std::sync::mpsc::channel();

    let app_r = app.clone();
    let main_handle = std::thread::spawn(move || loop {
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
                        SelectedView::Overview => overview::draw(app, f),
                    };
                })
                .unwrap();

            if poll(Duration::from_millis(50)).unwrap() {
                let event = read().unwrap();
                let mut app = app_r.lock().unwrap();
                if let CEvent::Key(key_code) = event {
                    match key_code.code {
                        KeyCode::Char('q') if app.mode == Mode::Normal => {
                            app.quit = true;
                            break;
                        }
                        k => {
                            match app.selected {
                                SelectedView::LogGroups => log_groups::handle_input(app, k, &tx),
                                SelectedView::Overview => overview::handle_input(app, k, &tx),
                            };
                        }
                    }
                }
                let app = app_r.lock().unwrap();
                if app.break_inner {
                    break;
                }
            }
        }
        disable_raw_mode().unwrap();
        execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
        let mut app = app_r.lock().unwrap();
        if app.quit {
            break;
        }
        app.query = input_from_editor(&app.query).unwrap();
        app.break_inner = false;
    });

    let app_r = app.clone();
    let _cwl_thread = std::thread::spawn(move || {
        cwl::run(app_r, rx);
    });

    main_handle.join().unwrap();

    Ok(())
}
