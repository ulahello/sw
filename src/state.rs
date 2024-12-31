// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use libsw_core::Sw;
use termcolor::{Color, ColorSpec};
use unicode_width::UnicodeWidthStr;

use core::num::IntErrorKind;
use core::time::Duration;
use core::{cmp, fmt, mem};
use std::io;
use std::time::Instant;

use crate::command::Command;
use crate::parse::ReadDur;
use crate::shell::Shell;

struct Crate {
    name: &'static str,
    license: &'static str,
    owners: &'static [&'static str],
}

// NOTE: volatile, copypasted data
const DEPENDENCIES: [Crate; 6] = [
    Crate {
        name: "argh",
        license: "BSD-3-Clause",
        owners: &[
            "Taylor Cramer <cramertj@google.com>",
            "Benjamin Brittain <bwb@google.com>",
            "Erick Tryzelaar <etryzelaar@google.com>",
        ],
    },
    Crate {
        name: "libsw-core",
        license: "MIT OR Apache-2.0",
        owners: &["Ula Shipman <ula.hello@mailbox.org>"],
    },
    Crate {
        name: "strsim",
        license: "MIT",
        owners: &["Danny Guo <danny@dannyguo.com>"],
    },
    Crate {
        name: "termcolor",
        license: "Unlicense OR MIT",
        owners: &["Andrew Gallant <jamslam@gmail.com>"],
    },
    Crate {
        name: "unicode-segmentation",
        license: "MIT/Apache-2.0",
        owners: &[
            "kwantam <kwantam@gmail.com>",
            "Manish Goregaokar <manishsmail@gmail.com>",
        ],
    },
    Crate {
        name: "unicode-width",
        license: "MIT/Apache-2.0",
        owners: &[
            "kwantam <kwantam@gmail.com>",
            "Manish Goregaokar <manishsmail@gmail.com>",
        ],
    },
];

impl fmt::Display for Crate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            name,
            owners,
            license,
        } = self;
        write!(f, "'{name}' by ")?;
        for owner in *owners {
            write!(f, "{owner}, ")?;
        }
        write!(f, "licensed under '{license}'")?;
        Ok(())
    }
}

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
    const MAX_PRECISION: u8 = crate::MAX_NANOS_CHARS;
    const COMMAND_SUGGEST_SIMILAR_THRESHOLD: f64 = 0.4;

    pub fn new(shell: &'shell mut Shell, name: String) -> Self {
        let input = String::with_capacity(shell.read_limit().into()); // @alloc
        Self {
            sw: Sw::new(),
            since_stop: Sw::new_started(),
            name,
            input,
            prec: Self::DEFAULT_PRECISION,
            shell,
        }
    }

    pub fn clamp_prec(spec: u8) -> (u8, bool) {
        let new = cmp::min(Self::MAX_PRECISION, spec);
        let clamped = spec != new;
        (new, clamped)
    }

    pub fn update(&mut self) -> io::Result<Option<Passback>> {
        let mut passback = None;
        let mut cb = self.shell.create_cmd_buf();
        let result = cb.read_cmd(&mut self.input, &self.name, self.sw.is_running())?;
        match result {
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
                    let (state, color) = if self.sw.is_running() {
                        ("running", Color::Green)
                    } else {
                        ("stopped", Color::Yellow)
                    };
                    cb.writeln_color(
                        ColorSpec::new().set_fg(Some(color)),
                        format_args!("{state}"),
                    )?;
                    if self.sw.checked_elapsed_at(now).is_none() {
                        cb.error(format_args!("elapsed time overflowing"))?;
                    }
                }

                Command::Toggle => {
                    let now = Instant::now();
                    let sw_overflow = !self.sw.checked_toggle_at(now);
                    if sw_overflow {
                        self.sw.stop_at(now);
                    }
                    if self.sw.is_running() {
                        assert!(!sw_overflow);
                        cb.info_change(format_args!("started stopwatch"))?;
                        cb.info_idle(format_args!(
                            "{} since stopped",
                            DurationFmt::new(
                                self.since_stop.elapsed_at(now),
                                self.prec,
                                cb.visual_cues()
                            )
                        ))?;
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
                    let sw_was_running = self.sw.is_running();
                    self.sw.reset();
                    if sw_was_running {
                        cb.info_change(format_args!("stopped and reset stopwatch"))?;
                    } else {
                        cb.info_change(format_args!("reset stopwatch"))?;
                    };
                }

                Command::Change => {
                    cb.read(&mut self.input, format_args!("new elapsed? "))?;
                    if let Some(try_read_dur) = ReadDur::parse(Shell::input(&self.input), false) {
                        match try_read_dur {
                            Ok(ReadDur { dur, is_neg }) => {
                                assert!(!is_neg);
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
                                if is_neg {
                                    let now = Instant::now();
                                    let underflow = dur > self.sw.elapsed_at(now);
                                    self.sw = self.sw.saturating_sub_at(dur, now);
                                    cb.info_change(format_args!("subtracted from elapsed time"))?;
                                    if underflow {
                                        cb.warn(format_args!("elapsed time clamped to zero"))?;
                                    }
                                } else {
                                    /* TODO: not aware of anchor, so its
                                     * possible to add to an overflowing
                                     * stopwatch without the warning */
                                    let overflow = self.sw.checked_add(dur).is_none();
                                    self.sw = self.sw.saturating_add(dur);
                                    cb.info_change(format_args!("added to elapsed time"))?;
                                    if overflow {
                                        cb.warn(format_args!(
                                            "new elapsed time too large, clamped to maximum"
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
                    let new_name = Shell::input(&self.input);
                    if new_name == self.name {
                        cb.info_idle(format_args!("name unchanged"))?;
                    } else {
                        if new_name.is_empty() {
                            cb.info_change(format_args!("cleared name"))?;
                        } else {
                            cb.info_change(format_args!("set name"))?;
                        }
                        self.name.replace_range(.., new_name);
                    }
                }

                Command::Precision => {
                    cb.read(&mut self.input, format_args!("new precision? "))?;
                    let try_prec = Shell::input(&self.input);
                    let parsed = match try_prec.parse::<u8>() {
                        Ok(prec) => Ok(Some(prec)),
                        Err(err) => match err.kind() {
                            IntErrorKind::PosOverflow => Ok(Some(u8::MAX)), // clamp overflow for better error ux
                            IntErrorKind::Empty => Ok(None),
                            _ => Err(err),
                        },
                    };
                    match parsed {
                        Ok(spec) => {
                            let (new_prec, clamped) =
                                Self::clamp_prec(spec.unwrap_or(Self::DEFAULT_PRECISION));
                            let old_prec = mem::replace(&mut self.prec, new_prec);
                            if clamped {
                                cb.warn(format_args!("precision clamped to {new_prec}"))?;
                            } else if old_prec == new_prec {
                                cb.info_idle(format_args!("precision unchanged"))?;
                            } else if spec.is_none() {
                                cb.info_change(format_args!("reset precision to {new_prec}"))?;
                            } else {
                                cb.info_change(format_args!("updated precision"))?;
                            }
                        }
                        Err(err) => cb.error(format_args!("{err}"))?,
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
                    for dep in DEPENDENCIES {
                        cb.writeln(format_args!("{dep}"))?;
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
                cb.error(format_args!(r#"unknown command (try "h" for help)"#))?;

                // try to find similarly named command and present it to the user
                if UnicodeWidthStr::width(unk) > 1 {
                    let (similarity, similar_cmd) = Command::iter()
                        .iter()
                        .map(|cmd| {
                            (
                                strsim::normalized_damerau_levenshtein(unk, cmd.long_name()),
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

        // sw and since_stop have mutually exclusive state
        if self.sw.is_running() {
            self.since_stop.reset();
        } else if self.since_stop.is_stopped() {
            let now = self.shell.last_read_time.unwrap();
            self.since_stop.start_at(now);
        }
        assert_ne!(self.sw.is_running(), self.since_stop.is_running());

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
