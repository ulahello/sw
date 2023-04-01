// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use libsw::Sw;
use termcolor::{Color, ColorSpec};

use core::fmt;
use core::num::IntErrorKind;
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
                        DurationFmt::new(self.sw.elapsed_at(now), self.prec, cb.visual_cues())
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
                        self.sw
                            .stop_at(now)
                            .expect("Sw::checked_toggle_at can only return None if Sw::is_running");
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
                            DurationFmt::new(
                                self.since_stop.elapsed_at(now),
                                self.prec,
                                cb.visual_cues()
                            )
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
                        self.since_stop.start().expect(
                            "since_stop and sw are never simultaneously running or stopped",
                        );
                    }
                    let sw_was_running = self.sw.is_running();
                    self.sw.reset();
                    let msg = if sw_was_running {
                        format_args!("stopped and reset stopwatch")
                    } else {
                        format_args!("reset stopwatch")
                    };
                    cb.info_change(msg)?;
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
                                        self.since_stop.start().expect(
                                            "since_stop and sw are never simultaneously running or stopped",
                                        );
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
                        /* if the number is too big, just clamp it. this
                         * improves the quality of error messages by hiding the
                         * implementation detail that we're parsing for a u8
                         * rather than another sized integer. */
                        let parsed = match try_prec.parse::<u8>() {
                            Ok(prec) => Ok(prec),
                            Err(err) => {
                                if *err.kind() == IntErrorKind::PosOverflow {
                                    Ok(u8::MAX)
                                } else {
                                    Err(err)
                                }
                            }
                        };

                        match parsed {
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
                    cb.writeln(format_args!(""))?;
                    cb.writeln(format_args!(
                        "{} uses the following libraries:",
                        env!("CARGO_PKG_NAME")
                    ))?;
                    // NOTE: volatile, copypasted data
                    for (name, license, owners) in [
                        (
                            "argh",
                            "BSD-3-Clause",
                            "Taylor Cramer <cramertj@google.com>, Benjamin Brittain <bwb@google.com>, Erick Tryzelaar <etryzelaar@google.com>",
                        ),
                        (
                            "libsw",
                            "MIT OR Apache-2.0",
                            "Ula Shipman <ula.hello@mailbox.org>",
                        ),
                        (
                            "termcolor",
                            "Unlicense OR MIT",
                            "Andrew Gallant <jamslam@gmail.com>",
                        ),
                        (
                            "unicode-segmentation",
                            "MIT/Apache-2.0",
                            "kwantam <kwantam@gmail.com>, Manish Goregaokar <manishsmail@gmail.com>",
                        ),
                        (
                            "unicode-width",
                            "MIT/Apache-2.0",
                            "kwantam <kwantam@gmail.com>, Manish Goregaokar <manishsmail@gmail.com>",
                        ),
                    ] {
                        cb.writeln(format_args!(
                            "'{name}' by {owners}, licensed under '{license}'"
                        ))?;
                    }
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
    visual_cues: bool,
}

impl DurationFmt {
    #[allow(clippy::assertions_on_constants)]
    #[must_use]
    pub const fn new(dur: Duration, prec: u8, visual_cues: bool) -> Self {
        assert!(prec <= 9);
        assert!(State::MAX_PRECISION == 9);
        Self {
            dur,
            prec,
            visual_cues,
        }
    }
}

impl fmt::Display for DurationFmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> where {
        fn plural(len: impl Into<u64>) -> &'static str {
            let len: u64 = len.into();
            if len == 1 {
                ""
            } else {
                "s"
            }
        }

        fn subsecs(f: &mut impl fmt::Write, fmt: &DurationFmt) -> fmt::Result {
            if fmt.prec != 0 {
                let nanos = fmt.dur.subsec_nanos();
                let width: usize = fmt.prec.into();
                write!(
                    f,
                    ".{:0>width$}",
                    nanos / 10_u32.pow(9 - u32::from(fmt.prec)),
                )?;
            }
            Ok(())
        }

        let total_secs = self.dur.as_secs();
        let total_mins = total_secs / 60;
        let secs = total_secs % 60;
        let mins = total_mins % 60;
        let hours = total_mins / 60;
        if self.visual_cues {
            let pad_zero = 2;
            write!(f, "{hours:0pad_zero$}:{mins:0pad_zero$}:{secs:0pad_zero$}")?;
            subsecs(f, self)?;
        } else {
            if hours != 0 {
                write!(f, "{hours} hour{}, ", plural(hours))?;
            }
            if mins != 0 {
                write!(f, "{mins} minute{}, ", plural(mins))?;
            }
            write!(f, "{secs}")?;
            subsecs(f, self)?;
            write!(
                f,
                " second{}",
                if self.prec == 0 { plural(secs) } else { "s" }
            )?;
        }
        Ok(())
    }
}
