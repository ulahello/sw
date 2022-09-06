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

// TODO: make these macros

//! Functions for printing colored messages.

use termcolor::{Buffer, Color, ColorSpec, WriteColor};

use core::fmt::Display;
use std::io::{self, stderr, Write};

/// Writes a red message to [`stderr`].
///
/// # Errors
///
/// Writing to `stderr` may fail.
pub fn red(msg: impl Display) -> io::Result<()> {
    writeln_color(stderr(), Color::Red, msg)
}

/// Writes a yellow message to [`stderr`].
///
/// # Errors
///
/// Writing to `stderr` may fail.
pub fn yellow(msg: impl Display) -> io::Result<()> {
    writeln_color(stderr(), Color::Yellow, msg)
}

/// Writes a magenta message to [`stderr`].
///
/// # Errors
///
/// Writing to `stderr` may fail.
pub fn magenta(msg: impl Display) -> io::Result<()> {
    writeln_color(stderr(), Color::Magenta, msg)
}

/// Writes a green message to [`stderr`].
///
/// # Errors
///
/// Writing to `stderr` may fail.
pub fn green(msg: impl Display) -> io::Result<()> {
    writeln_color(stderr(), Color::Green, msg)
}

/// Writes a grey message to [`stderr`].
///
/// # Errors
///
/// Writing to `stderr` may fail.
pub fn cyan(msg: impl Display) -> io::Result<()> {
    writeln_color(stderr(), Color::Cyan, msg)
}

/// Writes a colored message to `writer`, with a newline at the end.
///
/// # Errors
///
/// Writing to the `writer` may fail.
fn writeln_color(mut writer: impl Write, color: Color, msg: impl Display) -> io::Result<()> {
    let mut buffer = Buffer::ansi();
    let mut spec = ColorSpec::new();

    spec.set_fg(Some(color));
    buffer.set_color(&spec)?;
    writeln!(buffer, "{}", msg)?;
    buffer.reset()?;

    writer.write_all(buffer.as_slice())?;
    Ok(())
}
