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

//! `sw` state.

use crate::color::{cyan, green, magenta, red, yellow};
use crate::error::{FatalError, UserError};

use libsw::Stopwatch;

use core::fmt;
use core::time::Duration;
use std::io::{self, BufRead, Read, Write};
use std::time::Instant;

/// Stopwatch state.
pub struct State {
    /// Main [`Stopwatch`].
    pub sw: Stopwatch,
    /// Records the time elapsed while `sw` is stopped.
    since_stop: Stopwatch,
    /// Name of the stopwatch.
    pub name: String,
    /// Precision, in digits, for displaying the time
    prec: usize,
}

impl State {
    const PRECISION_INIT: usize = 2;
    const PRECISION_MAX: usize = 9; // nanosecond precision

    /// Returns a new [`State`].
    #[allow(clippy::new_without_default)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            sw: Stopwatch::new(),
            since_stop: Stopwatch::new_started(),
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
    ///
    /// # Errors
    ///
    /// - The command may be invalid
    /// - Reading and writing to the terminal may fail
    #[allow(clippy::missing_panics_doc)]
    pub fn update<W: Write>(
        &mut self,
        command: &Command,
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
                // display time elapsed
                writeln!(
                    stdout,
                    "{:?}",
                    DurationFmt::new(self.sw.elapsed(), self.prec)
                )?;
                stdout.flush()?;

                // indicate status
                if self.sw.is_running() {
                    green("running")?;
                } else {
                    yellow("stopped")?;
                }
            }

            Command::Toggle => {
                let now = Instant::now();
                self.sw.toggle_at(now);
                self.since_stop.toggle_at(now);

                if self.sw.is_running() {
                    magenta("started stopwatch")?;
                    cyan(format!(
                        "{} since stopped",
                        DurationFmt::new(self.since_stop.elapsed(), self.prec)
                    ))?;
                    self.since_stop.reset();
                } else {
                    magenta("stopped stopwatch")?;
                }
            }

            Command::Reset => {
                if self.sw.is_running() {
                    self.since_stop.start().unwrap();
                }
                self.sw.reset();
                magenta("reset stopwatch")?;
            }

            Command::Change => match read_duration("new value? ")? {
                Ok((dur, is_neg)) => {
                    if is_neg {
                        red(UserError::NegativeDuration)?;
                    } else {
                        if self.sw.is_running() {
                            self.since_stop.start().unwrap();
                        }
                        self.sw.set(dur);
                        magenta("updated elapsed time")?;
                    }
                }
                Err(error) => red(error)?,
            },

            Command::Offset => match read_duration("offset by? ")? {
                Ok((dur, is_neg)) => {
                    if is_neg {
                        self.sw -= dur;
                        magenta("subtracted from elapsed time")?;
                    } else {
                        self.sw += dur;
                        magenta("added to elapsed time")?;
                    }
                }
                Err(error) => red(error)?,
            },

            Command::Name => {
                self.name = readln("new name? ")?;
                if self.name.is_empty() {
                    magenta("cleared stopwatch name")?;
                } else {
                    magenta("updated stopwatch name")?;
                }
            }

            Command::Precision => match readln("new precision? ")?.parse::<usize>() {
                Ok(int) => {
                    if let Some(clamped) = self.set_precision(int) {
                        yellow(format!("precision clamped to {}", clamped))?;
                    } else {
                        magenta("updated precision")?;
                    }
                }
                Err(error) => red(UserError::InvalidInt(error))?,
            },

            Command::License => {
                writeln!(stdout, "copyright (C) 2022  {}", env!("CARGO_PKG_AUTHORS"))?;
                writeln!(stdout, "licensed under {}", env!("CARGO_PKG_LICENSE"))?;
            }

            Command::Quit => return Ok(true),
        }

        // sw and since_stop are inverse
        assert_eq!(self.sw.is_running(), self.since_stop.is_stopped());

        Ok(false)
    }
}

/// Enumeration over commands which can be executed by a [`State`].
#[allow(missing_docs)]
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
    /// Read a `Command` from the user.
    ///
    /// # Errors
    ///
    /// - Reading from `stdin` may fail
    /// - The user may enter an invalid command
    pub fn read(msg: &str, running: bool) -> Result<Result<Self, UserError>, FatalError> {
        let prompt = format!("{} {} ", msg, if running { '*' } else { ';' });
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

fn read_duration(msg: &str) -> Result<Result<(Duration, bool), UserError>, FatalError> {
    enum Unit {
        Seconds,
        Minutes,
        Hours,
    }

    impl Unit {
        fn new(chr: char) -> Result<Self, UserError> {
            match chr {
                's' => Ok(Self::Seconds),
                'm' => Ok(Self::Minutes),
                'h' => Ok(Self::Hours),
                s => Err(UserError::UnrecognizedUnit(s.into())),
            }
        }
    }

    let mut input = readln(msg)?;
    let try_unit = input.pop();
    let input: &str = input.trim();
    match input.parse::<f64>() {
        Ok(scalar) => {
            let unit = match Unit::new(try_unit.expect("if input is empty, it is an invalid float"))
            {
                Ok(unit) => unit,
                Err(error) => return Ok(Err(error)),
            };
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
        Err(_) if input.is_empty() => Ok(Err(UserError::UnrecognizedUnit(String::new()))),
        Err(error) => Ok(Err(UserError::InvalidFloat(error))),
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

struct DurationFmt {
    dur: Duration,
    prec: usize,
}

impl DurationFmt {
    pub fn new(dur: Duration, prec: usize) -> Self {
        Self { dur, prec }
    }

    fn plural(&self, x: f64) -> &'static str {
        if format!("{:.*}", self.prec, x) == "1" {
            ""
        } else {
            "s"
        }
    }
}

impl fmt::Debug for DurationFmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // TODO: don't use floats
        let m = self.dur.as_secs_f64() / 60.0;
        let h = m / 60.0;
        writeln!(f, "{}", self)?;
        writeln!(f, "{:.*} minute{}", self.prec, m, self.plural(m))?;
        write!(f, "{:.*} hour{}", self.prec, h, self.plural(h))?;
        Ok(())
    }
}

impl fmt::Display for DurationFmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // TODO: don't use floats
        let s = self.dur.as_secs_f64();
        write!(f, "{:.*} second{}", self.prec, s, self.plural(s))
    }
}
