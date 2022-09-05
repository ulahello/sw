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

//! `sw` errors.

#![allow(clippy::module_name_repetitions)]

use std::fmt;
use std::io;
use std::num::{ParseFloatError, ParseIntError};
use std::time::FromFloatSecsError;

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
    /// Unrecognized command
    ///
    /// Contains the string which was passed.
    UnrecognizedCommand(String),

    /// Unrecognized unit
    ///
    /// Contains the string which was passed.
    UnrecognizedUnit(String),

    /// Negative value passed for duration
    NegativeDuration,

    /// Failed to create a `Duration` from floating point seconds
    ///
    /// Contains the conversion error.
    InvalidDuration(FromFloatSecsError),

    /// Invalid floating point number
    ///
    /// Contains the parse error.
    InvalidFloat(ParseFloatError),

    /// Invalid integer
    ///
    /// Contains the parse error.
    InvalidInt(ParseIntError),
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::UnrecognizedCommand(command) => write!(f, "unrecognized command `{}`", command),
            Self::UnrecognizedUnit(unit) => write!(
                f,
                "unrecognized unit `{}` (expected one of 's', 'm', 'h')",
                unit
            ),
            Self::NegativeDuration => write!(f, "duration can't be negative"),
            Self::InvalidDuration(error) => write!(f, "invalid duration ({})", error),
            Self::InvalidFloat(error) => write!(f, "invalid float ({})", error),
            Self::InvalidInt(error) => write!(f, "invalid int ({})", error),
        }
    }
}
