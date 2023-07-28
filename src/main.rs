// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

#![warn(clippy::pedantic)]

mod command;
mod parse;
mod shell;
mod state;

const MAX_NANOS_CHARS: u8 = 9;
const SHELL_READ_LIMIT: u16 = 1024;

#[cfg(test)]
mod tests;

use argh::FromArgs;
use termcolor::ColorChoice;

use std::io::{self, stderr, stdin, stdout, BufWriter, IsTerminal, Write};
use std::process::ExitCode;

use crate::shell::Shell;
use crate::state::{Passback, State};

/// Terminal stopwatch that runs as a shell.
#[allow(clippy::struct_excessive_bools)]
#[derive(FromArgs)]
struct Args {
    /// disable text-based graphics and visual cues
    #[argh(short = 'v', switch)]
    no_visual_cues: bool,

    /// disable the use of colors in output
    #[argh(short = 'c', switch)]
    no_colors: bool,

    /// disable checking that standard output and input are both terminals
    #[argh(switch)]
    no_tty_check: bool,

    /// display version
    #[argh(short = 'V', switch)]
    version: bool,
}

fn main() -> ExitCode {
    fn print_error(err: &io::Error) -> io::Result<()> {
        let mut stderr = BufWriter::new(stderr());
        writeln!(stderr, "fatal error: {err}")?;
        stderr.flush()?;
        Ok(())
    }

    let args: Args = argh::from_env();
    if let Err(err) = try_main(&args) {
        _ = print_error(&err);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn try_main(args: &Args) -> io::Result<()> {
    if args.version {
        let mut stdout = BufWriter::new(stdout());
        writeln!(
            stdout,
            "{name} {version}",
            name = env!("CARGO_PKG_NAME"),
            version = env!("CARGO_PKG_VERSION")
        )?;
        stdout.flush()?;
        return Ok(());
    }

    if !args.no_tty_check {
        if !stdout().is_terminal() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "stdout is not a terminal (pass --no-tty-check to ignore)",
            ));
        } else if !stdin().is_terminal() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "stdin is not a terminal (pass --no-tty-check to ignore)",
            ));
        }
    }

    let cc = if args.no_colors {
        ColorChoice::Never
    } else {
        ColorChoice::Auto
    };
    let mut shell = Shell::new(cc, SHELL_READ_LIMIT, !args.no_visual_cues);
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
