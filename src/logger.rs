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

use log::{self, Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use std::io::Write;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

/// Simple logging implementation for `sw` non-fatal events.
pub struct Logger;

impl Logger {
    /// One-time initialize the logger.
    ///
    /// # Errors
    ///
    /// Returns [`SetLoggerError`] if the logger has already been initialized.
    pub fn init() -> Result<(), SetLoggerError> {
        log::set_logger(&Self).map(|()| log::set_max_level(LevelFilter::Trace))
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let stderr = BufferWriter::stderr(ColorChoice::Auto);
            let mut buffer = stderr.buffer();

            // set log color based on level
            buffer
                .set_color(ColorSpec::new().set_fg(Some(match record.level() {
                    Level::Error => Color::Ansi256(9), // bright red
                    Level::Warn => Color::Yellow,
                    Level::Info => Color::Ansi256(13), // bright magenta
                    Level::Debug => Color::Green,
                    Level::Trace => Color::Ansi256(8), // gray
                })))
                .unwrap();

            // print log contents
            writeln!(buffer, "{}", record.args()).unwrap();

            // reset color
            buffer.reset().unwrap();

            // flush buffer
            stderr.print(&buffer).unwrap();
        }
    }

    fn flush(&self) {}
}
