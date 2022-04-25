// sw: terminal stopwatch
// Copyright (C) 2022  Ula Shipman
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![feature(duration_checked_float)]

use std::io::{self, BufRead, Read, Write};
use std::process;
use std::time::Duration;

use sw::stopwatch::Stopwatch;
use sw::{FatalError, UserError};

fn main() {
    if let Err(error) = try_main() {
        eprintln!("fatal: {}", error);
        process::exit(1);
    }
}

fn try_main() -> Result<(), FatalError> {
    let mut sw = Stopwatch::new();

    print_splash()?;
    control_stopwatch(&mut sw)?;

    Ok(())
}

enum Command {
    Help,
    Display,
    Toggle,
    Reset,
    Change,
    Offset,
    Name,
    License,
    Quit,
}

impl Command {
    pub fn read(msg: &str, running: bool) -> Result<Result<Self, UserError>, FatalError> {
        let prompt = format!("{} {} ", msg, if running { '>' } else { '<' });
        match read_stdin(&prompt)?.to_lowercase().as_ref() {
            "h" => Ok(Ok(Self::Help)),
            "" => Ok(Ok(Self::Display)),
            "s" => Ok(Ok(Self::Toggle)),
            "r" => Ok(Ok(Self::Reset)),
            "c" => Ok(Ok(Self::Change)),
            "o" => Ok(Ok(Self::Offset)),
            "n" => Ok(Ok(Self::Name)),
            "l" => Ok(Ok(Self::License)),
            "q" => Ok(Ok(Self::Quit)),
            other => Ok(Err(UserError::UnrecognizedCommand(other.into()))),
        }
    }
}

enum Unit {
    Seconds,
    Minutes,
    Hours,
}

impl Unit {
    pub fn read() -> Result<Result<Self, UserError>, FatalError> {
        let mut stdout = io::stdout();
        writeln!(stdout, "(s)econds | (m)inutes | (h)ours")?;

        Ok(match read_stdin("which unit? ")?.to_lowercase().as_ref() {
            "s" => Ok(Self::Seconds),
            "m" => Ok(Self::Minutes),
            "h" => Ok(Self::Hours),
            other => Err(UserError::UnrecognizedUnit(other.into())),
        })
    }
}

fn read_stdin(msg: &str) -> Result<String, FatalError> {
    const READ_LIMIT: u64 = 128;

    let mut stdout = io::stdout();
    let stdin = io::stdin().lock();
    let mut input = String::new();

    write!(stdout, "{}", msg)?;
    stdout.flush()?;

    stdin.take(READ_LIMIT).read_line(&mut input)?;

    Ok(input.trim().escape_default().to_string())
}

fn read_duration(msg: &str) -> Result<Result<(Duration, bool), UserError>, FatalError> {
    match Unit::read()? {
        Ok(unit) => match read_stdin(msg)?.parse::<f64>() {
            Ok(scalar) => {
                let secs = match unit {
                    Unit::Seconds => scalar,
                    Unit::Minutes => scalar * 60.0,
                    Unit::Hours => scalar * 60.0 * 60.0,
                };
                match Duration::try_from_secs_f64(secs.abs()) {
                    Ok(duration) => Ok(Ok((duration, secs.is_sign_negative()))),
                    Err(error) => Ok(Err(UserError::InvalidDuration(error))),
                }
            }
            Err(error) => Ok(Err(UserError::InvalidFloat(error))),
        },
        Err(error) => Ok(Err(error)),
    }
}

fn print_splash() -> Result<(), FatalError> {
    let mut stderr = io::stderr();

    // splash text
    writeln!(
        stderr,
        "{} {}: {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_DESCRIPTION"),
    )?;
    writeln!(stderr, "type \"h\" for help, \"l\" for license")?;
    writeln!(stderr)?;

    Ok(())
}

fn control_stopwatch(stopwatch: &mut Stopwatch) -> Result<(), FatalError> {
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    // stopwatch name is empty to start
    let mut name = String::new();

    loop {
        // respond to command
        match Command::read(&name, stopwatch.is_running())? {
            Ok(command) => match command {
                Command::Help => {
                    writeln!(stdout, "| command | description          |")?;
                    writeln!(stdout, "| ---     | ---                  |")?;
                    writeln!(stdout, "| h       | print this message   |")?;
                    writeln!(stdout, "| <enter> | display elapsed time |")?;
                    writeln!(stdout, "| s       | toggle stopwatch     |")?;
                    writeln!(stdout, "| r       | reset stopwatch      |")?;
                    writeln!(stdout, "| c       | change elapsed time  |")?;
                    writeln!(stdout, "| o       | offset elapsed time  |")?;
                    writeln!(stdout, "| n       | name stopwatch       |")?;
                    writeln!(stdout, "| l       | print license info   |")?;
                    writeln!(stdout, "| q       | Abandon all Data     |")?;
                }

                Command::Display => writeln!(stdout, "{}", stopwatch)?,

                Command::Toggle => {
                    stopwatch.toggle();
                    if stopwatch.is_running() {
                        writeln!(stderr, "started stopwatch")?;
                    } else {
                        writeln!(stderr, "stopped stopwatch")?;
                    }
                }

                Command::Reset => {
                    stopwatch.reset();
                    writeln!(stderr, "reset stopwatch")?;
                }

                Command::Change => match read_duration("new value? ")? {
                    Ok((dur, is_neg)) => {
                        if is_neg {
                            writeln!(stderr, "{}", UserError::NegativeDuration)?;
                        } else {
                            stopwatch.set(dur);
                            writeln!(stderr, "updated elapsed time")?;
                        }
                    }
                    Err(error) => writeln!(stderr, "{}", error)?,
                },

                Command::Offset => match read_duration("offset by? ")? {
                    Ok((dur, is_neg)) => {
                        if is_neg {
                            stopwatch.sub(dur);
                            writeln!(stderr, "subtracted from elapsed time")?;
                        } else {
                            stopwatch.add(dur);
                            writeln!(stderr, "added to elapsed time")?;
                        }
                    }
                    Err(error) => writeln!(stderr, "{}", error)?,
                },

                Command::Name => {
                    name = read_stdin("new name? ")?;
                    if name.is_empty() {
                        writeln!(stderr, "cleared stopwatch name")?;
                    } else {
                        writeln!(stderr, "updated stopwatch name")?;
                    }
                }

                Command::License => {
                    writeln!(stderr, "copyright (C) 2022  Ula Shipman")?;
                    writeln!(stderr, "licensed under GPL-3.0-or-later")?;
                }

                Command::Quit => return Ok(()),
            },

            Err(error) => writeln!(stderr, "{}", error)?,
        }

        writeln!(stdout)?;
    }
}
