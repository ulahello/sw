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

use std::default::Default;
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
    /// Creates a [`Stopwatch`] with the given elapsed time.
    pub fn new(elapsed: Duration, running: bool) -> Self {
        Self {
            elapsed,
            start: if running { Some(Instant::now()) } else { None },
        }
    }

    /// Start measuring the time elapsed.
    ///
    /// # Errors
    ///
    /// Returns [`Error::AlreadyStarted`] if the stopwatch is running.
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
    /// Returns [`Error::AlreadyStopped`] if the stopwatch is not running.
    pub fn stop(&mut self) -> Result<(), Error> {
        if let Some(start) = self.start {
            self.add(Instant::now().saturating_duration_since(start));
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

    /// Stop and set the total elapsed time to `new`.
    pub fn set(&mut self, new: Duration) {
        self.elapsed = new;
        self.start = None;
    }

    /// Add `add` to the total elapsed time.
    pub fn add(&mut self, add: Duration) {
        self.elapsed = self.elapsed.saturating_add(add);
    }

    /// Subtract `sub` from the total elapsed time.
    pub fn sub(&mut self, sub: Duration) {
        self.sync_elapsed();
        self.elapsed = self.elapsed.saturating_sub(sub);
    }

    /// Return the total time elapsed.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        if let Some(start) = self.start {
            self.elapsed
                .saturating_add(Instant::now().saturating_duration_since(start))
        } else {
            self.elapsed
        }
    }

    /// Return [`true`] if the stopwatch is running.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        self.start.is_some()
    }

    /// Sync changes in the elapsed time, effectively toggling the stopwatch
    /// twice.
    fn sync_elapsed(&mut self) {
        if let Some(start) = self.start {
            let now = Instant::now();
            self.add(now.saturating_duration_since(start));
            self.start = Some(now);
        }
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self {
            elapsed: Duration::ZERO,
            start: None,
        }
    }
}

/// Errors associated with [`Stopwatch`]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// Called [`Stopwatch::start`] while running
    AlreadyStarted,
    /// Called [`Stopwatch::stop`] while stopped
    AlreadyStopped,
}

#[cfg(test)]
mod test {
    use crate::stopwatch::{Error, Stopwatch};
    use std::thread;
    use std::time::Duration;

    const SANE_TOLERANCE: Duration = Duration::from_millis(20);
    const SANE_DELAY: Duration = Duration::from_millis(200);

    #[test]
    fn default() {
        assert_eq!(Stopwatch::default().elapsed(), Duration::ZERO);
    }

    #[test]
    fn is_running() {
        let mut sw = Stopwatch::default();
        assert!(!sw.is_running());

        sw.start().unwrap();
        assert!(sw.is_running());

        sw.stop().unwrap();
        assert!(!sw.is_running());
    }

    #[test]
    fn toggle() {
        let mut sw = Stopwatch::default();
        assert!(!sw.is_running());

        sw.toggle();
        assert!(sw.is_running());

        sw.toggle();
        assert!(!sw.is_running());
    }

    #[test]
    fn reset() {
        let mut sw = Stopwatch::default();

        sw.start().unwrap();
        thread::sleep(SANE_DELAY);

        sw.reset();

        assert!(!sw.is_running());
        assert_eq!(sw.elapsed(), Duration::ZERO)
    }

    #[test]
    fn set() {
        let mut sw = Stopwatch::default();

        sw.start().unwrap();
        sw.set(SANE_DELAY);

        assert!(!sw.is_running());
        assert_eq!(sw.elapsed(), SANE_DELAY);
    }

    #[test]
    fn add() {
        let mut sw = Stopwatch::default();

        sw.add(SANE_DELAY);

        sw.start().unwrap();
        sw.add(SANE_DELAY);
        assert!(sw.is_running());

        sw.stop().unwrap();
        sw.add(SANE_DELAY);
        assert!(!sw.is_running());

        assert!(sw.elapsed() >= SANE_DELAY * 3);
        assert!(sw.elapsed() - (SANE_DELAY * 3) < SANE_TOLERANCE);
    }

    #[test]
    fn sub() {
        let mut sw = Stopwatch::default();

        sw.start().unwrap();
        thread::sleep(SANE_DELAY);

        sw.sub(SANE_DELAY);
        assert!(sw.elapsed() < SANE_TOLERANCE);
        assert!(sw.is_running());

        sw.set(SANE_DELAY * 4);
        sw.sub(SANE_DELAY * 3);
        assert!(sw.elapsed() >= SANE_DELAY);
        assert!(sw.elapsed - SANE_DELAY < SANE_TOLERANCE);
    }

    #[test]
    fn double_starts_stops_errs() {
        let mut sw = Stopwatch::default();

        assert_eq!(sw.start(), Ok(()));
        assert_eq!(sw.start(), Err(Error::AlreadyStarted));

        assert_eq!(sw.stop(), Ok(()));
        assert_eq!(sw.stop(), Err(Error::AlreadyStopped));
    }

    #[test]
    fn sane_elapsed_halted() {
        let mut sw = Stopwatch::default();

        sw.start().unwrap();
        thread::sleep(SANE_DELAY);
        sw.stop().unwrap();

        assert!(sw.elapsed() >= SANE_DELAY);
        assert!(sw.elapsed() - SANE_DELAY < SANE_TOLERANCE);
    }

    #[test]
    fn sane_elapsed_active() {
        let mut sw = Stopwatch::default();

        sw.start().unwrap();
        thread::sleep(SANE_DELAY);

        assert!(sw.elapsed() >= SANE_DELAY);
        assert!(sw.elapsed() - SANE_DELAY < SANE_TOLERANCE);
    }
}
