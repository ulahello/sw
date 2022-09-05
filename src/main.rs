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

use sw::color::red;
use sw::error::FatalError;
use sw::state::{Command, State};

use std::io::{self, BufWriter, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(error) = try_main() {
        eprintln!("fatal: {}", error);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn try_main() -> Result<(), FatalError> {
    print_splash()?;

    let mut state = State::new();
    let mut stdout = BufWriter::new(io::stdout());

    loop {
        // respond to command
        match Command::read(&state.name, state.sw.is_running())? {
            Ok(command) => {
                if state.update(&command, &mut stdout)? {
                    return Ok(());
                }
            }
            Err(error) => red(error)?,
        }

        writeln!(stdout)?;
        // bufwriter flushes sparingly so must do this manually
        stdout.flush()?;
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
