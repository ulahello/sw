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
}

impl Command {
    pub fn from_stdin() -> Result<Result<Self, UserError>, FatalError> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(match input.to_lowercase().trim() {
            "q" => Ok(Self::Quit),
            "h" => Ok(Self::Help),
            "" => Ok(Self::Display),
            "s" => Ok(Self::Toggle),
            "r" => Ok(Self::Reset),
            other => Err(UserError::UnrecognizedCommand(other.into())),
        })
        }
    }
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
        // prompt for command
        write!(stdout, "> ")?;
        stdout.flush()?;

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
            },

            Err(error) => writeln!(stderr, "{}", error)?,
        }
    }
}
