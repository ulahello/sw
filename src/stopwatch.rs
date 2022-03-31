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

use std::time::{Duration, Instant};

pub struct Stopwatch {
    elapsed: Duration,
    start: Option<Instant>,
}

impl Stopwatch {
    pub const fn new() -> Self {
        Self {
            start: None,
            elapsed: Duration::ZERO,
        }
    }

    pub fn start(&mut self) -> Result<(), Error> {
        if self.start.is_some() {
            Err(Error::AlreadyStarted)
        } else {
            self.start = Some(Instant::now());
            Ok(())
        }
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        if self.start.is_none() {
            Err(Error::AlreadyStopped)
        } else {
            self.start = None;
            Ok(())
        }
    }

    pub fn elapsed(&self) -> Duration {
        if let Some(start) = self.start {
            self.elapsed + start.elapsed()
        } else {
            self.elapsed
        }
    }
}

pub enum Error {
    AlreadyStarted,
    AlreadyStopped,
}
