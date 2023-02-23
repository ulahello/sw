// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use termcolor::Color;

use std::io;

use crate::shell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Display,
    Toggle,
    Reset,
    Change,
    Offset,
    Name,
    Precision,
    License,
    Quit,
}

impl Command {
    pub fn read(name: &str, is_running: bool) -> io::Result<Option<Self>> {
        let prompt = format!("{name} {} ", if is_running { "*" } else { ";" });
        let command = match shell::read(prompt)?.to_lowercase().as_ref() {
            "h" => Self::Help,
            "" => Self::Display,
            "s" => Self::Toggle,
            "r" => Self::Reset,
            "c" => Self::Change,
            "o" => Self::Offset,
            "n" => Self::Name,
            "p" => Self::Precision,
            "l" => Self::License,
            "q" => Self::Quit,
            _ => {
                shell::log(Color::Red, r#"unknown command (try "h" for help)"#)?;
                shell::write("")?;
                return Ok(None);
            }
        };
        Ok(Some(command))
    }
}
