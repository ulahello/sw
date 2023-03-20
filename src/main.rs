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

use argh::FromArgs;
use termcolor::ColorChoice;

use std::io::{self, stderr, Write};
use std::process::ExitCode;

use crate::shell::Shell;
use crate::state::{Passback, State};

/// Terminal stopwatch
#[derive(FromArgs)]
struct Args {
    /// disable text-based graphics and visual cues
    #[argh(short = 'x', switch)]
    no_visual_cues: bool,
}

fn main() -> ExitCode {
    if let Err(err) = try_main(argh::from_env()) {
        _ = writeln!(stderr(), "fatal: {err}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn try_main(args: Args) -> io::Result<()> {
    let mut shell = Shell::new(ColorChoice::Auto, 64, !args.no_visual_cues);
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
