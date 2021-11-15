use std::{
    fmt::{Display, Write},
    io,
    sync::MutexGuard,
    time::{SystemTime, UNIX_EPOCH},
};

use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::Widget;

pub(crate) struct TimeSelector {
    selected_start: Time,
    selected_end: Time,
    pub(crate) popup: bool,
}
impl TimeSelector {
    pub(crate) fn from_strings(start: &str, end: &str) -> Result<Self, &'static str> {
        let start_time = to_time(start)?;
        let end_time = to_time(end)?;
        if start_time.is_relative() && end_time.is_relative() {
            return Err("Both start and end time can't be relative");
        }
        Ok(Self { selected_start: start_time, selected_end: end_time, popup: false })
    }

    pub(crate) fn to_timestamps(&self) -> (i64, i64) {
        if let Time::Relative(u, v) = self.selected_start {
            let end = match self.selected_end {
                Time::Specific(_) => todo!(),
                Time::Now => now(),
                Time::Relative(_, _) => panic!("can't happen"),
            };
            let offset = to_offset(u, v);
            let start = end - offset;
            return (start, end);
        }
        if let Time::Relative(u, v) = self.selected_end {
            let start = match self.selected_start {
                Time::Specific(_) => todo!(),
                Time::Now => now(),
                Time::Relative(_, _) => panic!("can't happen"),
            };
            let offset = to_offset(u, v);
            let end = start + offset;
            return (start, end);
        }
        let start = match self.selected_start {
            Time::Specific(_) => todo!(),
            Time::Now => now(),
            Time::Relative(_, _) => panic!("can't happen"),
        };
        let end = match self.selected_end {
            Time::Specific(_) => todo!(),
            Time::Now => now(),
            Time::Relative(_, _) => panic!("can't happen"),
        };
        (start, end)
    }
}

impl Display for TimeSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{} - {}", self.selected_start, self.selected_end))
    }
}

impl Default for TimeSelector {
    fn default() -> Self {
        Self {
            selected_start: Time::Relative(RelativeUnit::Hours, 1),
            selected_end: Time::Now,
            popup: false,
        }
    }
}

fn now() -> i64 {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

fn to_time(string: &str) -> Result<Time, &'static str> {
    let string = string.trim().to_lowercase();
    if string == "now" {
        return Ok(Time::Now);
    }
    let chars = string.chars();
    let mut nums = String::default();
    for (i, c) in chars.enumerate() {
        if c.is_digit(10) {
            nums.push(c);
        } else {
            match c {
                's' => {
                    if i == string.len() - 1 {
                        let parsed = nums.parse();
                        if let Ok(num) = parsed {
                            return Ok(Time::Relative(RelativeUnit::Seconds, num));
                        } else {
                            return Err("Invalid number of seconds");
                        }
                    } else {
                        return Err("Invalid relative suffix");
                    }
                }
                'm' => {
                    if i == string.len() - 1 {
                        let parsed = nums.parse();
                        if let Ok(num) = parsed {
                            return Ok(Time::Relative(RelativeUnit::Minutes, num));
                        } else {
                            return Err("Invalid number of minutes");
                        }
                    } else {
                        return Err("Invalid relative suffix");
                    }
                }
                'h' => {
                    if i == string.len() - 1 {
                        let parsed = nums.parse();
                        if let Ok(num) = parsed {
                            return Ok(Time::Relative(RelativeUnit::Hours, num));
                        } else {
                            return Err("Invalid number of hours");
                        }
                    } else {
                        return Err("Invalid relative suffix");
                    }
                }
                'd' => {
                    if i == string.len() - 1 {
                        let parsed = nums.parse();
                        if let Ok(num) = parsed {
                            return Ok(Time::Relative(RelativeUnit::Days, num));
                        } else {
                            return Err("Invalid number of days");
                        }
                    } else {
                        return Err("Invalid relative suffix");
                    }
                }
                '-' => todo!(),
                _ => todo!(),
            }
        }
    }

    Err("Something wrong")
}

fn to_offset(unit: RelativeUnit, value: u32) -> i64 {
    let multiplier = match unit {
        RelativeUnit::Seconds => 1,
        RelativeUnit::Minutes => 60,
        RelativeUnit::Hours => 3600,
        RelativeUnit::Days => 3600 * 24,
    };
    value as i64 * multiplier
}

#[derive(Clone, Copy)]
enum RelativeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
}

impl Display for RelativeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelativeUnit::Seconds => f.write_char('s'),
            RelativeUnit::Minutes => f.write_char('m'),
            RelativeUnit::Hours => f.write_char('h'),
            RelativeUnit::Days => f.write_char('d'),
        }
    }
}
enum Time {
    Relative(RelativeUnit, u32),
    Specific(i64),
    Now,
}

impl Time {
    fn is_relative(&self) -> bool {
        match self {
            Time::Relative(_, _) => true,
            Time::Specific(_) => false,
            Time::Now => false,
        }
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Time::Relative(u, v) => f.write_str(&format!("{}{}", v, u)),
            Time::Specific(_) => todo!(),
            Time::Now => f.write_str("now"),
        }
    }
}

pub(crate) fn draw(
    app: &MutexGuard<crate::App>,
    frame: &mut Frame<CrosstermBackend<io::Stdout>>,
    area: Rect,
) {
    let time_selector_string = app.time_selector.to_string();
    let status_bar = Paragraph::new(time_selector_string.as_str())
        .style(match app.focused {
            Widget::TimeSelector => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        })
        .block(Block::default().borders(Borders::ALL).title("selected time"));
    frame.render_widget(status_bar, area);
}
