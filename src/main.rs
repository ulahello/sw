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

use std::io::{self, Write};
use std::time::Duration;

use sw::stopwatch::Stopwatch;
use sw::{FatalError, UserError};

fn main() {
    match control_stopwatch(Stopwatch::new()) {
        Ok(()) => (),
        Err(err) => {
            eprintln!("fatal: {}", err);
        }
    }
}

enum Command {
    Quit,
    Help,
    Display,
    Toggle,
    Reset,
    Set,
}

impl Command {
    pub fn from_stdin() -> Result<Result<Self, UserError>, FatalError> {
        Ok(match read_input("> ")?.to_lowercase().as_ref() {
            "q" => Ok(Self::Quit),
            "h" => Ok(Self::Help),
            "" => Ok(Self::Display),
            "s" => Ok(Self::Toggle),
            "r" => Ok(Self::Reset),
            "=" => Ok(Self::Set),
            other => Err(UserError::UnrecognizedCommand(other.into())),
        })
    }
}

enum Unit {
    Seconds,
    Minutes,
    Hours,
}

impl Unit {
    pub fn from_stdin() -> Result<Result<Self, UserError>, FatalError> {
        Ok(match read_input("which unit? ")?.to_lowercase().as_ref() {
            "s" => Ok(Self::Seconds),
            "m" => Ok(Self::Minutes),
            "h" => Ok(Self::Hours),
            other => Err(UserError::UnrecognizedUnit(other.into())),
        })
    }
}

fn read_input(msg: &str) -> Result<String, FatalError> {
    let mut stdout = io::stdout();
    write!(stdout, "{}", msg)?;
    stdout.flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().into())
}

fn control_stopwatch(mut stopwatch: Stopwatch) -> Result<(), FatalError> {
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    // splash text
    writeln!(
        stderr,
        "{} {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )?;

    writeln!(stderr, "type \"h\" for help")?;
    writeln!(stderr)?;

    loop {
        // respond to command
        match Command::from_stdin()? {
            Ok(command) => match command {
                Command::Quit => return Ok(()),

                Command::Help => {
                    writeln!(stdout, "| command | description          |")?;
                    writeln!(stdout, "| ---     | ---                  |")?;
                    writeln!(stdout, "| q       | quit                 |")?;
                    writeln!(stdout, "| h       | print this message   |")?;
                    writeln!(stdout, "| s       | toggle stopwatch     |")?;
                    writeln!(stdout, "| r       | reset stopwatch      |")?;
                    writeln!(stdout, "| =       | set elapsed time     |")?;
                    writeln!(stdout, "| <enter> | display elapsed time |")?;
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

                Command::Set => {
                    writeln!(stdout, "(s)econds | (m)inutes | (h)ours")?;

                    match Unit::from_stdin()? {
                        Ok(unit) => match read_input("new value? ")?.parse::<f64>() {
                            Ok(scalar) => {
                                if scalar.is_sign_negative() {
                                    writeln!(stderr, "{}", UserError::NegativeDuration)?;
                                } else {
                                    stopwatch.set(Duration::from_secs_f64(match unit {
                                        Unit::Seconds => scalar,
                                        Unit::Minutes => scalar * 60.0,
                                        Unit::Hours => scalar * 60.0 * 60.0,
                                    }));
                                }
                            }
                            Err(error) => {
                                writeln!(stderr, "{}", UserError::InvalidFloat(error))?;
                            }
                        },
                        Err(error) => writeln!(stderr, "{}", error)?,
                    }
                }
            },

            Err(error) => writeln!(stderr, "{}", error)?,
        }
    }
}
