// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use libsw::{Instant, Sw};
use termcolor::{Color, ColorSpec};
use unicode_width::UnicodeWidthStr;

use core::fmt;
use core::num::IntErrorKind;
use core::time::Duration;
use std::io;

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
    input: String,
    prec: u8,
    shell: &'shell mut Shell,
}

impl<'shell> State<'shell> {
    const DEFAULT_PRECISION: u8 = 2;
    const MAX_PRECISION: u8 = 9;
    const COMMAND_SUGGEST_SIMILAR_THRESHOLD: f64 = 0.4;

    pub fn new(shell: &'shell mut Shell) -> Self {
        let input = String::with_capacity(shell.read_limit().into()); // @alloc
        Self {
            sw: Sw::new(),
            since_stop: Sw::new_started(),
            name: input.clone(),
            input,
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
        match cb.read_cmd(&mut self.input, &self.name, self.sw.is_running())? {
            Ok(command) => match command {
                Command::Help => {
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
                        /* if this branch is reached, since_stop was (and is)
                         * running. so we know that sw was stopped but is now
                         * running, meaning since_stop will be reset in the next
                         * condition, meaning we dont have to do anything. */
                    }

                    if self.sw.is_running() {
                        cb.info_change(format_args!("started stopwatch"))?;
                        if let Some(since_stop_elapsed) = self.since_stop.checked_elapsed_at(now) {
                            cb.info_idle(format_args!(
                                "{} since stopped",
                                DurationFmt::new(since_stop_elapsed, self.prec, cb.visual_cues())
                            ))?;
                        }
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
                        "stopped and reset stopwatch"
                    } else {
                        "reset stopwatch"
                    };
                    cb.info_change(format_args!("{msg}"))?;
                }

                Command::Change => {
                    cb.read(&mut self.input, format_args!("new elapsed? "))?;
                    if let Some(try_read_dur) = ReadDur::parse(Shell::input(&self.input), false) {
                        match try_read_dur {
                            Ok(ReadDur { dur, is_neg }) => {
                                assert!(!is_neg);
                                if self.sw.is_running() {
                                    self.since_stop.start().expect(
                                        "since_stop and sw are never simultaneously running or stopped",
                                    );
                                }
                                self.sw.set(dur);
                                cb.info_change(format_args!("updated elapsed time"))?;
                            }

                            Err(err) => err.display(&mut cb)?,
                        }
                    } else {
                        cb.info_idle(format_args!("elapsed time unchanged"))?;
                    }
                }

                Command::Offset => {
                    cb.read(&mut self.input, format_args!("offset by? "))?;
                    if let Some(try_read_dur) = ReadDur::parse(Shell::input(&self.input), true) {
                        match try_read_dur {
                            Ok(ReadDur { dur, is_neg }) => {
                                let now = Instant::now();
                                #[allow(clippy::collapsible_else_if)]
                                if is_neg {
                                    cb.info_change(format_args!("subtracted from elapsed time"))?;
                                    if let Some(new_sw) = self.sw.checked_sub_at(dur, now) {
                                        self.sw = new_sw;
                                    } else if self.sw.checked_elapsed_at(now).is_some() {
                                        self.sw.reset_in_place();
                                        cb.warn(format_args!("elapsed time clamped to zero"))?;
                                    } else {
                                        self.sw = self.sw.saturating_sub_at(dur, now);
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
                    cb.read(&mut self.input, format_args!("new name? "))?;
                    let new = Shell::input(&self.input);
                    if new == self.name {
                        cb.info_idle(format_args!("name unchanged"))?;
                    } else {
                        if new.is_empty() {
                            cb.info_change(format_args!("cleared name"))?;
                        } else {
                            cb.info_change(format_args!("set name"))?;
                        }
                        self.name.clear();
                        self.name.push_str(new)
                    }
                }

                Command::Precision => {
                    cb.read(&mut self.input, format_args!("new precision? "))?;
                    let try_prec = Shell::input(&self.input);
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
                            "strsim",
                            "MIT",
                            "Danny Guo <danny@dannyguo.com>",
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
                    assert!(
                        passback.is_none(),
                        "State::update is not called after Passback::Quit"
                    );
                    passback = Some(Passback::Quit);
                }
            },

            Err(unk) => {
                cb.error(format_args!(
                    r#"unknown command '{unk}' (try "h" for help)"#
                ))?;

                // try to find similarly named command and present it to the user
                if UnicodeWidthStr::width(unk) > 1 {
                    let (similarity, similar_cmd) = Command::iter()
                        .iter()
                        .map(|cmd| {
                            (
                                strsim::normalized_damerau_levenshtein(&unk, cmd.long_name()),
                                cmd,
                            )
                        })
                        .reduce(|(mut most_similar, mut closest_cmd), (similarity, cmd)| {
                            if similarity > most_similar {
                                most_similar = similarity;
                                closest_cmd = cmd;
                            }
                            (most_similar, closest_cmd)
                        })
                        .expect("there is at least 1 command");

                    if similarity >= Self::COMMAND_SUGGEST_SIMILAR_THRESHOLD {
                        cb.info_idle(format_args!(
                            "note: the '{}' command has a similar name",
                            similar_cmd.long_name()
                        ))?;
                    }
                }
            }
        }

        Ok(passback)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DurationFmt {
    dur: Duration,
    prec: u8, // <= crate::MAX_NANOS_CHARS
    visual_cues: bool,
}

impl DurationFmt {
    #[allow(clippy::assertions_on_constants)]
    #[must_use]
    pub const fn new(dur: Duration, prec: u8, visual_cues: bool) -> Self {
        assert!(prec <= crate::MAX_NANOS_CHARS);
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
                    nanos / 10_u32.pow(u32::from(crate::MAX_NANOS_CHARS) - u32::from(fmt.prec)),
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
