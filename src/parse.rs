// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use termcolor::ColorSpec;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use core::fmt;
use core::num::{IntErrorKind, ParseIntError};
use core::time::Duration;
use std::io;

use crate::shell::{CmdBuf, ERROR};

pub(crate) mod long;
pub(crate) mod short;

use long::LongErrKind;
use short::ShortErrKind;

const SEC_PER_MIN: u8 = 60;
const MIN_PER_HOUR: u8 = 60;
const SEC_PER_HOUR: u16 = 3600;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadDur {
    pub dur: Duration,
    pub is_neg: bool,
}

impl ReadDur {
    pub fn parse(s: &str, allow_neg: bool) -> Option<Result<Self, ParseErr<'_>>> {
        if s.is_empty() {
            None
        } else {
            let parsed = match Self::parse_as_short(s, allow_neg) {
                Ok(short_ok) => Ok(short_ok),
                Err(short_err) => match Self::parse_as_long(s, allow_neg) {
                    Ok(long_ok) => Ok(long_ok),
                    Err(long_err) => {
                        if s.as_bytes().contains(&b':') {
                            Err(long_err)
                        } else {
                            Err(short_err)
                        }
                    }
                },
            };
            Some(parsed)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErrKind<'s> {
    Short(ShortErrKind<'s>),
    Long(LongErrKind),
    Negative,
}

impl From<LongErrKind> for ErrKind<'_> {
    fn from(long: LongErrKind) -> Self {
        Self::Long(long)
    }
}
impl<'s> From<ShortErrKind<'s>> for ErrKind<'s> {
    fn from(short: ShortErrKind<'s>) -> Self {
        Self::Short(short)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseErr<'s> {
    src: &'s str,
    span: ByteSpan<'s>,
    kind: ErrKind<'s>,
}

impl<'s> ParseErr<'s> {
    #[inline]
    pub(crate) fn new(span: ByteSpan<'s>, kind: impl Into<ErrKind<'s>>) -> Self {
        let mut kind = kind.into();

        match kind {
            ErrKind::Long(ref mut long_kind) => {
                if let LongErrKind::Int { group, err } = long_kind {
                    if *err.kind() == IntErrorKind::PosOverflow {
                        *long_kind = LongErrKind::DurationOverflow(*group);
                    }
                }
            }
            ErrKind::Short(ref mut short_kind) => {
                if let ShortErrKind::ParseInt { err, unit } = short_kind {
                    if *err.kind() == IntErrorKind::PosOverflow {
                        *short_kind = ShortErrKind::DurOverflow(*unit);
                    }
                }
            }
            // showing int overflow error to user breaks abstraction
            ErrKind::Negative => (),
        }

        Self {
            src: span.src,
            span,
            kind,
        }
    }

    // TODO: careful printing user input. may contain escape codes.
    pub fn display(&self, cmd: &mut CmdBuf<'_>) -> io::Result<()> {
        fn display_error_red_highlighted(err: &ParseErr, cmd: &mut CmdBuf<'_>) -> io::Result<()> {
            // text before span
            cmd.write(format_args!("{}", err.span.get_before()))?;

            // red span text
            cmd.write_color(
                ColorSpec::new().set_fg(Some(ERROR)),
                format_args!("{}", err.span.get()),
            )?;

            // text after span
            cmd.writeln(format_args!("{}", err.span.get_after()))?;

            Ok(())
        }

        fn display_error_caret_underlined(err: &ParseErr, cmd: &mut CmdBuf<'_>) -> io::Result<()> {
            display_error_red_highlighted(err, cmd)?;

            // write caret underline
            let spaces: usize = UnicodeWidthStr::width(err.span.get_before());
            let carets: usize = UnicodeWidthStr::width(err.span.get());
            cmd.writeln_color(
                ColorSpec::new().set_fg(Some(ERROR)),
                format_args!("{}{}", " ".repeat(spaces), "^".repeat(carets)),
            )?;

            Ok(())
        }

        fn display_error_no_visual(err: &ParseErr, cmd: &mut CmdBuf<'_>) -> io::Result<()> {
            // write what the error text is
            cmd.writeln_color(
                ColorSpec::new().set_fg(Some(ERROR)),
                format_args!("found error: {}", err.span.get()),
            )?;

            Ok(())
        }

        /* write source text errors highlighted */
        if cmd.visual_cues() {
            display_error_caret_underlined(self, cmd)?;
        } else {
            display_error_no_visual(self, cmd)?;
        }

        /* write error message */
        cmd.error(format_args!("{self}"))?;

        /* write help message */
        if self.has_help_message() {
            cmd.info_idle(format_args!("note: {self:#}"))?;
        }

        Ok(())
    }
}

impl ParseErr<'_> {
    fn has_help_message(&self) -> bool {
        match &self.kind {
            ErrKind::Short(short) => short.has_help_message(),
            ErrKind::Long(long) => long.has_help_message(),
            ErrKind::Negative => true,
        }
    }
}

impl fmt::Display for ParseErr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if f.alternate() {
            match &self.kind {
                ErrKind::Short(short) => write!(f, "{short:#}"),
                ErrKind::Long(long) => write!(f, "{long:#}"),
                ErrKind::Negative => write!(f, "only offsets to duration can be negative"),
            }
        } else {
            match &self.kind {
                ErrKind::Short(short) => write!(f, "{short}"),
                ErrKind::Long(long) => write!(f, "{long}"),
                ErrKind::Negative => write!(f, "expected positive duration"),
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ByteSpan<'s> {
    start: usize,
    len: usize,
    src: &'s str,
}

impl<'s> ByteSpan<'s> {
    #[must_use]
    #[inline]
    pub const fn new(start: usize, len: usize, s: &'s str) -> Self {
        Self { start, len, src: s }
    }

    #[must_use]
    #[inline]
    pub const fn new_all(s: &'s str) -> Self {
        Self::new(0, s.len(), s)
    }

    pub fn shift_start_left(&mut self, bytes: usize) {
        self.start -= bytes;
        self.len += bytes;
    }

    pub fn shift_start_right(&mut self, bytes: usize) {
        self.start += bytes;
        self.len -= bytes;
    }

    pub fn get(&self) -> &'s str {
        &self.src[self.start..self.start + self.len]
    }

    pub fn get_before(&self) -> &'s str {
        &self.src[..self.start]
    }

    pub fn get_after(&self) -> &'s str {
        &self.src[self.start + self.len..]
    }

    pub fn trim_whitespace(&mut self) {
        // forward
        {
            let graphs = UnicodeSegmentation::graphemes(self.get(), true);
            for chr in graphs {
                if chr.chars().all(char::is_whitespace) {
                    self.shift_start_right(chr.len());
                } else {
                    break;
                }
            }
        }
        // backward
        {
            let graphs = UnicodeSegmentation::graphemes(self.get(), true).rev();
            for chr in graphs {
                if chr.chars().all(char::is_whitespace) {
                    self.len -= chr.len();
                } else {
                    break;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Unit {
    Second,
    Minute,
    Hour,
}

impl Unit {
    #[inline]
    pub fn from_grapheme(grapheme: &str) -> Result<Self, &str> {
        match grapheme {
            "s" => Ok(Self::Second),
            "m" => Ok(Self::Minute),
            "h" => Ok(Self::Hour),
            unk => Err(unk),
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Second => "second",
            Self::Minute => "minute",
            Self::Hour => "hour",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ParseFracErr {
    ParseDigit {
        idx: usize,
        len: usize,
        err: ParseIntError,
    },
    NumeratorOverflow {
        idx: usize,
    },
}

pub(crate) fn parse_frac(s: &str, places: u8) -> Result<u32, ParseFracErr> {
    let mut num: u32 = 0;
    let mut place: u32 = places.into();
    let graphs = UnicodeSegmentation::grapheme_indices(s, true);
    for (idx, chr) in graphs {
        let digit = chr.parse::<u8>().map_err(|err| ParseFracErr::ParseDigit {
            idx,
            len: chr.len(),
            err,
        })?;
        if place == 0 {
            // excess digits truncated
        } else {
            assert!(digit < 10);
            num = num
                .checked_add(u32::from(digit) * 10_u32.pow(place - 1))
                .ok_or(ParseFracErr::NumeratorOverflow { idx })?;

            place -= 1;
        }
    }
    Ok(num)
}
