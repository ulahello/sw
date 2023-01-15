// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

#![warn(clippy::pedantic)]

mod command;
mod parse;
mod shell;
mod state;

use crate::command::Command;
use crate::state::State;

use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(err) = try_main() {
        eprintln!("fatal: {err}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn try_main() -> io::Result<()> {
    shell::splash_text()?;

    let mut state = State::new();
    loop {
        if let Some(command) = Command::read(state.name(), state.sw().is_running())? {
            state.update(&command)?;
            if command == Command::Quit {
                break;
            }
        }
    }

    Ok(())
}
