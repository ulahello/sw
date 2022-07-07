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

use log::{debug, error, info, trace, warn};
use std::io::{self, BufRead, BufWriter, Read, Write};
use std::process::ExitCode;
use std::time::Duration;

use sw::stopwatch::Stopwatch;
use sw::{FatalError, Logger, UserError};

fn main() -> ExitCode {
    if let Err(error) = try_main() {
        eprintln!("fatal: {}", error);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn try_main() -> Result<(), FatalError> {
    Logger::init().unwrap();
    print_splash()?;

    let mut state = State::new();
    let mut stdout = BufWriter::new(io::stdout());

    loop {
        // respond to command
        match Command::read(&state.name, state.sw.is_running())? {
            Ok(command) => {
                if state.update(command, &mut stdout)? {
                    return Ok(());
                }
            }
            Err(error) => error!("{}", error),
        }

        writeln!(stdout)?;
        // bufwriter flushes sparingly so must do this manually
        stdout.flush()?;

        // update since_stop. the idea is for since_stop to be running while the
        // state.sw is stopped, and to reset as soon as state.sw starts
        if state.sw.is_running() && state.since_stop.is_running() {
            state.since_stop.reset();
        } else if !state.sw.is_running() && !state.since_stop.is_running() {
            state.since_stop.toggle();
        }
    }
}

fn readln(msg: &str) -> Result<String, FatalError> {
    const READ_LIMIT: u64 = 128;

    let mut stdout = io::stdout();
    let stdin = io::stdin().lock();
    let mut input = String::new();

    // print prompt (must flush)
    write!(stdout, "{}", msg)?;
    stdout.flush()?;

    // read a limited number of bytes from stdin
    stdin.take(READ_LIMIT).read_line(&mut input)?;

    // trim whitespace and escape weird characters
    Ok(input.trim().escape_default().to_string())
}

fn read_duration(msg: &str) -> Result<Result<(Duration, bool), UserError>, FatalError> {
    match Unit::read()? {
        Ok(unit) => match readln(msg)?.parse::<f64>() {
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
    let mut stderr = BufWriter::new(io::stderr());

    writeln!(
        stderr,
        "{} {}: {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_DESCRIPTION"),
    )?;
    writeln!(stderr, "type \"h\" for help, \"l\" for license")?;
    writeln!(stderr)?;

    stderr.flush()?;

    Ok(())
}

struct State {
    sw: Stopwatch,
    since_stop: Stopwatch,
    name: String,
    prec: usize,
}

impl State {
    const PRECISION_INIT: usize = 2;
    const PRECISION_MAX: usize = 9; // nanosecond precision

    pub fn new() -> Self {
        Self {
            sw: Stopwatch::new(Duration::ZERO, false),
            since_stop: Stopwatch::new(Duration::ZERO, true),
            name: String::new(),
            prec: Self::PRECISION_INIT,
        }
    }

    /// Sets the precision of displayed values. If the precision was clamped, it
    /// returns what it was clamped to.
    pub fn set_precision(&mut self, prec: usize) -> Option<usize> {
        if prec <= Self::PRECISION_MAX {
            self.prec = prec;
            None
        } else {
            self.prec = Self::PRECISION_MAX;
            Some(Self::PRECISION_MAX)
        }
    }

    /// Updates the state with a [`Command`], returning [true] if the command
    /// was [`Quit`](Command::Quit).
    pub fn update<W: Write>(
        &mut self,
        command: Command,
        stdout: &mut W,
    ) -> Result<bool, FatalError> {
        match command {
            Command::Help => {
                writeln!(stdout, "| command | description           |")?;
                writeln!(stdout, "| ------- | --------------------- |")?;
                writeln!(stdout, "| h       | print this message    |")?;
                writeln!(stdout, "| <enter> | display elapsed time  |")?;
                writeln!(stdout, "| s       | toggle stopwatch      |")?;
                writeln!(stdout, "| r       | reset stopwatch       |")?;
                writeln!(stdout, "| c       | change elapsed time   |")?;
                writeln!(stdout, "| o       | offset elapsed time   |")?;
                writeln!(stdout, "| n       | name stopwatch        |")?;
                writeln!(stdout, "| p       | set display precision |")?;
                writeln!(stdout, "| l       | print license info    |")?;
                writeln!(stdout, "| q       | Abandon all Data      |")?;
            }

            Command::Display => {
                let elapsed = self.sw.elapsed().as_secs_f32();

                // display time elapsed in different units
                writeln!(stdout, "{:.*} seconds", self.prec, elapsed)?;
                writeln!(stdout, "{:.*} minutes", self.prec, elapsed / 60.0)?;
                writeln!(stdout, "{:.*} hours", self.prec, elapsed / 60.0 / 60.0)?;

                stdout.flush()?;

                // indicate status
                if self.sw.is_running() {
                    debug!("running");
                } else {
                    warn!("stopped");
                }
            }

            Command::Toggle => {
                self.sw.toggle();
                if self.sw.is_running() {
                    info!("started stopwatch");
                    trace!(
                        "{:.*} seconds since stopped",
                        self.prec,
                        self.since_stop.elapsed().as_secs_f32()
                    );
                } else {
                    info!("stopped stopwatch");
                }
            }

            Command::Reset => {
                self.sw.reset();
                info!("reset stopwatch");
            }

            Command::Change => match read_duration("new value? ")? {
                Ok((dur, is_neg)) => {
                    if is_neg {
                        error!("{}", UserError::NegativeDuration);
                    } else {
                        self.sw.set(dur);
                        info!("updated elapsed time");
                    }
                }
                Err(error) => error!("{}", error),
            },

            Command::Offset => match read_duration("offset by? ")? {
                Ok((dur, is_neg)) => {
                    if is_neg {
                        self.sw.sub(dur);
                        info!("subtracted from elapsed time");
                    } else {
                        self.sw.add(dur);
                        info!("added to elapsed time");
                    }
                }
                Err(error) => error!("{}", error),
            },

            Command::Name => {
                self.name = readln("new name? ")?;
                if self.name.is_empty() {
                    info!("cleared stopwatch name");
                } else {
                    info!("updated stopwatch name");
                }
            }

            Command::Precision => match readln("new precision? ")?.parse::<usize>() {
                Ok(int) => {
                    if let Some(clamped) = self.set_precision(int) {
                        warn!("precision clamped to {}", clamped);
                    } else {
                        info!("updated precision");
                    }
                }
                Err(error) => error!("{}", UserError::InvalidInt(error)),
            },

            Command::License => {
                writeln!(stdout, "copyright (C) 2022  Ula Shipman")?;
                writeln!(stdout, "licensed under GPL-3.0-or-later")?;
            }

            Command::Quit => return Ok(true),
        };

        Ok(false)
    }
}

enum Command {
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
    pub fn read(msg: &str, running: bool) -> Result<Result<Self, UserError>, FatalError> {
        let prompt = format!("{} {} ", msg, if running { '>' } else { '<' });
        match readln(&prompt)?.to_lowercase().as_ref() {
            "h" => Ok(Ok(Self::Help)),
            "" => Ok(Ok(Self::Display)),
            "s" => Ok(Ok(Self::Toggle)),
            "r" => Ok(Ok(Self::Reset)),
            "c" => Ok(Ok(Self::Change)),
            "o" => Ok(Ok(Self::Offset)),
            "n" => Ok(Ok(Self::Name)),
            "p" => Ok(Ok(Self::Precision)),
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
        writeln!(io::stdout(), "(s)econds | (m)inutes | (h)ours")?;

        Ok(match readln("which unit? ")?.to_lowercase().as_ref() {
            "s" => Ok(Self::Seconds),
            "m" => Ok(Self::Minutes),
            "h" => Ok(Self::Hours),
            other => Err(UserError::UnrecognizedUnit(other.into())),
        })
    }
}
