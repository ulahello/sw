// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use crate::command::Command;
use crate::parse::ReadDur;
use crate::shell;

use libsw::Stopwatch;
use termcolor::Color;

use core::fmt;
use core::time::Duration;
use std::io;
use std::time::Instant;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    sw: Stopwatch,
    since_stop: Stopwatch,
    name: String,
    prec: u8,
}

impl State {
    const DEFAULT_PRECISION: u8 = 2;
    const MAX_PRECISION: u8 = 9;

    pub fn new() -> Self {
        Self {
            sw: Stopwatch::new(),
            since_stop: Stopwatch::new_started(),
            name: String::new(),
            prec: Self::DEFAULT_PRECISION,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn sw(&self) -> &Stopwatch {
        &self.sw
    }

    pub fn set_prec(&mut self, mut prec: u8) -> Result<(), u8> {
        if prec > Self::MAX_PRECISION {
            prec = Self::MAX_PRECISION;
            self.prec = prec;
            Err(prec)
        } else {
            self.prec = prec;
            Ok(())
        }
    }

    pub fn update(&mut self, command: &Command) -> io::Result<()> {
        match command {
            Command::Help => {
                shell::write(concat!(
                    "| command | description           |\n",
                    "| ------- | --------------------- |\n",
                    "| h       | show help             |\n",
                    "| <enter> | display elapsed time  |\n",
                    "| s       | toggle stopwatch      |\n",
                    "| r       | reset stopwatch       |\n",
                    "| c       | change elapsed time   |\n",
                    "| o       | offset elapsed time   |\n",
                    "| n       | name stopwatch        |\n",
                    "| p       | set display precision |\n",
                    "| l       | print license info    |\n",
                    "| q       | Abandon all Data      |",
                ))?;
            }

            Command::Display => {
                shell::write(DurationFmt::new(self.sw.elapsed(), self.prec))?;
                if self.sw.is_running() {
                    shell::log(Color::Green, "running")?;
                } else {
                    shell::log(Color::Yellow, "stopped")?;
                }
            }

            Command::Toggle => {
                let now = Instant::now();
                self.sw.toggle_at(now);
                self.since_stop.toggle_at(now);

                if self.sw.is_running() {
                    shell::log(Color::Magenta, "started stopwatch")?;
                    shell::log(
                        Color::Cyan,
                        format!(
                            "{} since stopped",
                            DurationFmt::new(self.since_stop.elapsed(), self.prec)
                        ),
                    )?;
                    self.since_stop.reset();
                } else {
                    shell::log(Color::Magenta, "stopped stopwatch")?;
                }
            }

            Command::Reset => {
                if self.sw.is_running() {
                    self.since_stop.start().unwrap();
                }
                self.sw.reset();
                shell::log(Color::Magenta, "reset stopwatch")?;
            }

            Command::Change => {
                if let Some(ReadDur { dur, is_neg }) = shell::read_dur("new elapsed? ")? {
                    if is_neg {
                        shell::log(Color::Red, "new elapsed time can't be negative")?;
                    } else {
                        if self.sw.is_running() {
                            self.since_stop.start().unwrap();
                        }
                        self.sw.set(dur);
                        shell::log(Color::Magenta, "updated elapsed time")?;
                    }
                }
            }

            Command::Offset => {
                if let Some(ReadDur { dur, is_neg }) = shell::read_dur("offset by? ")? {
                    if is_neg {
                        shell::log(Color::Magenta, "subtracted from elapsed time")?;
                        if let Some(new_sw) = self.sw.checked_sub(dur) {
                            self.sw = new_sw;
                        } else {
                            self.sw.set_in_place(Duration::ZERO);
                            shell::log(Color::Yellow, "elapsed time clamped to zero")?;
                        }
                    } else {
                        self.sw += dur;
                        shell::log(Color::Magenta, "added to elapsed time")?;
                    }
                }
            }

            Command::Name => {
                let new = shell::read("new name? ")?;
                if new.is_empty() && !self.name.is_empty() {
                    shell::log(Color::Magenta, "cleared name")?;
                }
                self.name = new;
            }

            Command::Precision => {
                let try_prec = shell::read("new precision? ")?;
                if try_prec.is_empty() {
                    if self.prec != Self::DEFAULT_PRECISION {
                        self.prec = Self::DEFAULT_PRECISION;
                        shell::log(
                            Color::Magenta,
                            format!("reset precision to {}", Self::DEFAULT_PRECISION),
                        )?;
                    }
                } else {
                    match try_prec.parse() {
                        Ok(prec) => {
                            if let Err(clamped) = self.set_prec(prec) {
                                shell::log(
                                    Color::Yellow,
                                    format!("precision clamped to {clamped}"),
                                )?;
                            } else {
                                shell::log(Color::Magenta, "updated precision")?;
                            }
                        }
                        Err(err) => shell::log(Color::Red, err)?,
                    }
                }
            }

            Command::License => {
                shell::write(format!(
                    "copyright (C) 2022-2023 {}",
                    env!("CARGO_PKG_AUTHORS")
                ))?;
                shell::write(format!("licensed under {}", env!("CARGO_PKG_LICENSE")))?;
            }

            Command::Quit => (),
        }

        // visually separate command outputs
        shell::write("")?;

        Ok(())
    }
}

#[derive(Debug)]
struct DurationFmt {
    dur: Duration,
    prec: u8,
}

impl DurationFmt {
    pub const fn new(dur: Duration, prec: u8) -> Self {
        Self { dur, prec }
    }
}

impl fmt::Display for DurationFmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let total_secs = self.dur.as_secs();
        let total_mins = total_secs / 60;
        let secs = total_secs % 60;
        let mins = total_mins % 60;
        let hours = total_mins / 60;
        write!(f, "{hours:02}:{mins:02}:{secs:02}")?;
        if self.prec != 0 {
            let nanos = self.dur.subsec_nanos();
            write!(
                f,
                ".{:0>width$}",
                nanos / 10_u32.pow(9 - u32::from(self.prec)),
                width = self.prec.into()
            )?;
        }
        Ok(())
    }
}
