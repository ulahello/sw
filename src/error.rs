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

use std::fmt;
use std::io;
use std::num::ParseFloatError;

/// Fatal runtime errors
#[derive(Debug)]
pub enum FatalError {
    /// I/O error
    Io(io::Error),
}

impl From<io::Error> for FatalError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl fmt::Display for FatalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Io(err) => write!(f, "io: {}", err),
        }
    }
}

/// Errors in the user input
#[derive(Debug)]
pub enum UserError {
    /// Unrecognized command; contains the string which was passed
    UnrecognizedCommand(String),
    /// Unrecognized unit; contains the string which was passed
    UnrecognizedUnit(String),
    /// Negative value passed for duration
    NegativeDuration,
    /// Invalid floating point number; contains the parse error
    InvalidFloat(ParseFloatError),
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::UnrecognizedCommand(command) => write!(f, "unrecognized command `{}`", command),
            Self::UnrecognizedUnit(unit) => write!(f, "unrecognized unit `{}`", unit),
            Self::NegativeDuration => write!(f, "duration can't be negative"),
            Self::InvalidFloat(error) => write!(f, "invalid f64 ({})", error),
        }
    }
}
