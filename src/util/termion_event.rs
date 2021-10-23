use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Read;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termion::event::Key;
use termion;

pub enum Event<I> {
    Input(I),
    Tick,
}

enum Stop {
    Now,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    stopper: mpsc::Sender<Stop>,
    stopper2: mpsc::Sender<Stop>,
    input_handle: thread::JoinHandle<()>,
    tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let (stx, srx) = mpsc::channel();
        let (stx2, srx2) = mpsc::channel();
        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {

                let mut file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open("tlog.txt")
                    .unwrap();

                let mut stdin = termion::async_stdin().bytes();
                loop {
                    let b = stdin.next();
                    match b {
                        Some(x) => match x {
                            Ok(k) => {
                                let e = termion::event::parse_event(k, &mut stdin);
                                if let Ok(event) = e {
                                    if let termion::event::Event::Key(key) = event {
                                        if let Err(_) = tx.send(Event::Input(key)) {
                                            eprintln!("closing2\n");
                                            writeln!(file, "error sending key").unwrap();
                                            return;
                                        } else {
                                            writeln!(file, "sending key").unwrap();
                                        }
                                    }
                                }
                            },
                            _ => {}
                        },
                        None => {
                            thread::sleep(config.tick_rate/2);
                        }
                    }
                    if let Ok(Stop::Now) = srx.try_recv() {
                        writeln!(file, "closing").unwrap();
                        eprintln!("closing1\n");
                        return;
                    }
                }
            })
        };
        let tick_handle = {
            thread::spawn(move || loop {
                if let Err(_err) = tx.send(Event::Tick) {
                    //eprintln!("{}", err);
                    break;
                }
                if let Ok(Stop::Now) = srx2.try_recv() {
                    return;
                }
                thread::sleep(config.tick_rate);
            })
        };
        Events {
            rx,
            stopper: stx,
            stopper2: stx2,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn close(self) {
        eprintln!("closing\n");
        self.stopper.send(Stop::Now).unwrap();
        self.stopper2.send(Stop::Now).unwrap();
        self.input_handle.join().unwrap();
        self.tick_handle.join().unwrap();
    }

}
