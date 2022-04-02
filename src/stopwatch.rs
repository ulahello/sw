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

//! Defines an abstraction for stopwatches

use std::fmt;
use std::time::{Duration, Instant};

/// A stopwatch abstraction. Measures and accumulates time between starts and
/// stops.
#[derive(Clone, Copy)]
#[must_use]
pub struct Stopwatch {
    elapsed: Duration,
    start: Option<Instant>,
}

impl Stopwatch {
    /// Creates a stopped [`Stopwatch`] with zero time elapsed.
    pub const fn new() -> Self {
        Self {
            start: None,
            elapsed: Duration::ZERO,
        }
    }

    /// Start measuring the time elapsed.
    ///
    /// # Errors
    ///
    /// Returns [`Error::AlreadyStarted`] if the stopwatch has already been
    /// started.
    pub fn start(&mut self) -> Result<(), Error> {
        if self.is_running() {
            Err(Error::AlreadyStarted)
        } else {
            self.start = Some(Instant::now());
            Ok(())
        }
    }

    /// Stop measuring the time elapsed since the last start.
    ///
    /// On success, this will add to the total elapsed time.
    ///
    /// # Errors
    ///
    /// Returns [`Error::AlreadyStopped`] if the stopwatch has already been
    /// stopped.
    pub fn stop(&mut self) -> Result<(), Error> {
        if let Some(start) = self.start {
            self.elapsed += start.elapsed();
            self.start = None;
            Ok(())
        } else {
            Err(Error::AlreadyStopped)
        }
    }

    /// Start or stop the stopwatch.
    ///
    /// If stopped, then start, and if running, then stop.
    pub fn toggle(&mut self) {
        if self.is_running() {
            let _ = self.stop();
        } else {
            let _ = self.start();
        }
    }

    /// Stop and reset the elapsed time to zero.
    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.start = None;
    }

    /// Return the total time elapsed.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        if let Some(start) = self.start {
            self.elapsed + start.elapsed()
        } else {
            self.elapsed
        }
    }

    /// Return [`true`] if the stopwatch is running.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        self.start.is_some()
    }
}

impl fmt::Display for Stopwatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let elapsed = self.elapsed().as_secs_f64();

        // display time elapsed in different units
        writeln!(f, "{:.4} seconds", elapsed)?;
        writeln!(f, "{:.4} minutes", elapsed / 60.0)?;
        writeln!(f, "{:.4} hours", elapsed / 60.0 / 60.0)?;

        // indicate status
        write!(
            f,
            "status: {}",
            if self.is_running() {
                "running"
            } else {
                "stopped"
            }
        )
    }
}

/// Errors associated with [`Stopwatch`]
#[derive(Clone, Copy)]
pub enum Error {
    /// Called [`Stopwatch::start`] while already running
    AlreadyStarted,
    /// Called [`Stopwatch::stop`] while already stopped
    AlreadyStopped,
}
