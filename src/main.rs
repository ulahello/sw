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

/// Terminal stopwatch that runs as a shell.
#[derive(FromArgs)]
struct Args {
    /// disable text-based graphics and visual cues
    #[argh(short = 'v', switch)]
    no_visual_cues: bool,

    /// disable the use of colors in output
    #[argh(short = 'c', switch)]
    no_colors: bool,
}

fn main() -> ExitCode {
    let args: Args = argh::from_env();
    if let Err(err) = try_main(&args) {
        _ = writeln!(stderr(), "fatal error: {err}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn try_main(args: &Args) -> io::Result<()> {
    let cc = if args.no_colors {
        ColorChoice::Never
    } else {
        ColorChoice::Auto
    };
    let mut shell = Shell::new(cc, 64, !args.no_visual_cues);
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
