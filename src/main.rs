// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

#![warn(clippy::pedantic)]

mod command;
mod parse;
mod shell;
mod state;

#[cfg(test)]
mod tests;

use termcolor::ColorChoice;

use std::io;
use std::process::ExitCode;

use crate::shell::Shell;
use crate::state::{Passback, State};

fn main() -> ExitCode {
    if let Err(err) = try_main() {
        eprintln!("fatal: {err}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn try_main() -> io::Result<()> {
    let mut shell = Shell::new(ColorChoice::Auto, 64);
    shell.splash_text()?;

    let mut state = State::new(&mut shell);
    loop {
        if let Some(passback) = state.update()? {
            match passback {
                Passback::Quit => break,
            }
        }
    }

    shell.finish()?;

    Ok(())
}
