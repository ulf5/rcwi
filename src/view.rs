use std::{io::Stdout, sync::{MutexGuard, mpsc::Sender}};

use crossterm::event::KeyCode;
use tui::{Frame, backend::CrosstermBackend};

use crate::{App, cwl::AwsReq};


pub(crate) trait View {
    fn draw(&self, app: MutexGuard<App>, frame: &mut Frame<CrosstermBackend<Stdout>>);
    fn handle_input(&self, app: MutexGuard<App>, key_code: KeyCode, cwl: &Sender<AwsReq>);
}
