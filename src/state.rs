// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use libsw::Sw;
use termcolor::{Color, ColorSpec};

use core::fmt;
use core::time::Duration;
use std::io;
use std::time::Instant;

use crate::command::Command;
use crate::parse::ReadDur;
use crate::shell::Shell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Passback {
    Quit,
}

pub struct State<'shell> {
    sw: Sw,
    since_stop: Sw,
    name: String,
    prec: u8,
    shell: &'shell mut Shell,
}

impl<'shell> State<'shell> {
    const DEFAULT_PRECISION: u8 = 2;
    const MAX_PRECISION: u8 = 9;

    pub fn new(shell: &'shell mut Shell) -> Self {
        Self {
            sw: Sw::new(),
            since_stop: Sw::new_started(),
            name: String::new(),
            prec: Self::DEFAULT_PRECISION,
            shell,
        }
    }

    pub fn set_prec(prec: &mut u8, mut new: u8) -> Result<(), u8> {
        if new > Self::MAX_PRECISION {
            new = Self::MAX_PRECISION;
            *prec = new;
            Err(new)
        } else {
            *prec = new;
            Ok(())
        }
    }

    pub fn update(&mut self) -> io::Result<Option<Passback>> {
        let mut passback = None;
        let mut cb = self.shell.create_cmd_buf();
        match cb.read_cmd(&self.name, self.sw.is_running())? {
            Ok(command) => match command {
                Command::Help => {
                    // TODO: be able to print help page for individual commands
                    for help_cmd in Command::iter() {
                        cb.writeln(format_args!(
                            "{} or {}. {}.",
                            help_cmd.long_name(),
                            help_cmd.short_name_display(),
                            help_cmd.description()
                        ))?;
                    }
                }

                Command::Display => {
                    let now = Instant::now();
                    cb.writeln(format_args!(
                        "{}",
                        DurationFmt::new(self.sw.elapsed_at(now), self.prec)
                    ))?;
                    if self.sw.is_running() {
                        cb.writeln_color(
                            ColorSpec::new().set_fg(Some(Color::Green)),
                            format_args!("running"),
                        )?;
                    } else {
                        cb.writeln_color(
                            ColorSpec::new().set_fg(Some(Color::Yellow)),
                            format_args!("stopped"),
                        )?;
                    }
                    if self.sw.checked_elapsed_at(now).is_none() {
                        cb.error(format_args!("elapsed time overflowing"))?;
                    }
                }

                Command::Toggle => {
                    let now = Instant::now();
                    let sw_overflow = self.sw.checked_toggle_at(now).is_none();
                    if sw_overflow {
                        self.sw.stop().unwrap();
                    }
                    if self.since_stop.checked_toggle_at(now).is_none() {
                        /* if this fails, since_stop was (and is) running. so we
                         * know that sw was stopped but is now running, meaning
                         * since_stop will be reset in the next condition, meaning
                         * it's fine for this to fail. */
                    }

                    if self.sw.is_running() {
                        cb.info_change(format_args!("started stopwatch"))?;
                        cb.info_idle(format_args!(
                            "{} since stopped",
                            DurationFmt::new(self.since_stop.elapsed_at(now), self.prec)
                        ))?;
                        self.since_stop.reset();
                        assert!(!sw_overflow);
                    } else {
                        cb.info_change(format_args!("stopped stopwatch"))?;
                        if sw_overflow {
                            cb.warn(format_args!(
                                "new elapsed time too large, clamped to maximum"
                            ))?;
                        }
                    }
                }

                Command::Reset => {
                    if self.sw.is_running() {
                        self.since_stop.start().unwrap();
                    }
                    self.sw.reset();
                    cb.info_change(format_args!("reset stopwatch"))?;
                }

                Command::Change => {
                    let input = cb.read(format_args!("new elapsed? "))?;
                    if let Some(try_read_dur) = ReadDur::parse(&input) {
                        match try_read_dur {
                            Ok(ReadDur { dur, is_neg }) => {
                                if is_neg {
                                    cb.error(format_args!("new elapsed time can't be negative"))?;
                                } else {
                                    if self.sw.is_running() {
                                        self.since_stop.start().unwrap();
                                    }
                                    self.sw.set(dur);
                                    cb.info_change(format_args!("updated elapsed time"))?;
                                }
                            }

                            Err(err) => err.display(&mut cb)?,
                        }
                    } else {
                        cb.info_idle(format_args!("elapsed time unchanged"))?;
                    }
                }

                Command::Offset => {
                    let input = cb.read(format_args!("offset by? "))?;
                    if let Some(try_read_dur) = ReadDur::parse(&input) {
                        match try_read_dur {
                            Ok(ReadDur { dur, is_neg }) =>
                            {
                                #[allow(clippy::collapsible_else_if)]
                                if is_neg {
                                    cb.info_change(format_args!("subtracted from elapsed time"))?;
                                    if let Some(new_sw) = self.sw.checked_sub(dur) {
                                        self.sw = new_sw;
                                    } else {
                                        self.sw.reset_in_place();
                                        cb.warn(format_args!("elapsed time clamped to zero"))?;
                                    }
                                } else {
                                    if let Some(new_sw) = self.sw.checked_add(dur) {
                                        self.sw = new_sw;
                                        cb.info_change(format_args!("added to elapsed time"))?;
                                    } else {
                                        cb.error(format_args!(
                                            "cannot add offset, new elapsed time would overflow"
                                        ))?;
                                    }
                                }
                            }

                            Err(err) => err.display(&mut cb)?,
                        }
                    } else {
                        cb.info_idle(format_args!("no offset applied"))?;
                    }
                }

                Command::Name => {
                    let new = cb.read(format_args!("new name? "))?;
                    if new == self.name {
                        cb.info_idle(format_args!("name unchanged"))?;
                    } else {
                        if new.is_empty() {
                            cb.info_change(format_args!("cleared name"))?;
                        } else {
                            cb.info_change(format_args!("set name"))?;
                        }
                        self.name = new;
                    }
                }

                Command::Precision => {
                    let try_prec = cb.read(format_args!("new precision? "))?;
                    if try_prec.is_empty() {
                        if self.prec == Self::DEFAULT_PRECISION {
                            cb.info_idle(format_args!("precision unchanged"))?;
                        } else {
                            self.prec = Self::DEFAULT_PRECISION;
                            cb.info_change(format_args!(
                                "reset precision to {}",
                                Self::DEFAULT_PRECISION
                            ))?;
                        }
                    } else {
                        match try_prec.parse() {
                            Ok(prec) => {
                                if let Err(clamped) = Self::set_prec(&mut self.prec, prec) {
                                    cb.warn(format_args!("precision clamped to {clamped}"))?;
                                } else {
                                    cb.info_change(format_args!("updated precision"))?;
                                }
                            }
                            Err(err) => cb.error(format_args!("{err}"))?,
                        }
                    }
                }

                Command::Visuals => {
                    cb.set_visual_cues(!cb.visual_cues());
                    cb.info_change(format_args!(
                        "visual cues {}",
                        if cb.visual_cues() {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    ))?;
                }

                Command::License => {
                    cb.writeln(format_args!(
                        "copyright (C) 2022-2023 {}",
                        env!("CARGO_PKG_AUTHORS")
                    ))?;
                    cb.writeln(format_args!("licensed under {}", env!("CARGO_PKG_LICENSE")))?;
                }

                Command::Quit => {
                    /* quit message comes from foot terminal
                     * (https://codeberg.org/dnkl/foot) */
                    cb.info_change(format_args!("goodbye"))?;
                    assert!(passback.is_none());
                    passback = Some(Passback::Quit);
                }
            },

            Err(unk) => cb.error(format_args!(
                r#"unknown command '{unk}' (try "h" for help)"#
            ))?,
        }

        Ok(passback)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DurationFmt {
    dur: Duration,
    prec: u8, // <= 9
}

impl DurationFmt {
    #[allow(clippy::assertions_on_constants)]
    #[must_use]
    pub const fn new(dur: Duration, prec: u8) -> Self {
        assert!(prec <= 9);
        assert!(State::MAX_PRECISION == 9);
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
