// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use unicode_segmentation::UnicodeSegmentation;

use core::fmt;
use core::num::ParseIntError;
use core::time::Duration;

use super::{ByteSpan, ErrKind, ParseErr, ParseFracErr, ReadDur, Unit, SEC_PER_HOUR, SEC_PER_MIN};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum UnitErrKind<'s> {
    UnitMissing,
    UnitUnknown(&'s str),
    DurMissing(Unit),
    ParseInt { err: ParseIntError, unit: Unit },
    DurOverflow(Unit),
}

impl UnitErrKind<'_> {
    pub(crate) fn has_help_message(&self) -> bool {
        match self {
            Self::UnitMissing
            | Self::DurMissing(_)
            | Self::ParseInt { .. }
            | Self::UnitUnknown(_)
            | Self::DurOverflow(_) => true,
        }
    }
}

impl fmt::Display for UnitErrKind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if f.alternate() {
            match self {
                Self::UnitMissing | Self::UnitUnknown(_) => {
                    write!(f, "use 's' for seconds, 'm' for minutes, and 'h' for hours")
                }
                Self::DurMissing(unit) | Self::ParseInt { err: _, unit } => {
                    write!(f, "expected the number of {unit}s")
                }
                Self::DurOverflow(_) => write!(f, "this duration is too large to be represented"),
            }
        } else {
            match self {
                Self::UnitMissing => write!(f, "missing unit"),
                Self::UnitUnknown(unk) => write!(f, "unrecognised unit '{unk}'"),
                Self::DurMissing(_) => write!(f, "unit given, but missing value"),
                Self::ParseInt { err, unit: _ } => write!(f, "{err}"),
                Self::DurOverflow(unit) => write!(f, "duration overflow while parsing {unit}s"),
            }
        }
    }
}

impl ReadDur {
    pub fn parse_as_unit(s: &str, allow_neg: bool) -> Result<Self, ParseErr> {
        // whitespace? + number + whitespace? + unit + whitespace?
        let s = s.trim_end();

        let (try_unit_idx, try_unit) = UnicodeSegmentation::grapheme_indices(s, true)
            .peekable()
            .last()
            .ok_or(ParseErr::new(
                ByteSpan::new_all(s),
                UnitErrKind::UnitMissing,
            ))?;

        let unit = Unit::from_grapheme(try_unit).map_err(|_| {
            ParseErr::new(
                ByteSpan::new(try_unit_idx, try_unit.len(), s),
                UnitErrKind::UnitUnknown(try_unit),
            )
        })?;

        let dur_len = try_unit_idx;
        let mut dur_span = ByteSpan::new(0, dur_len, s);
        dur_span.trim_whitespace();
        if dur_span.get().is_empty() {
            Err(ParseErr::new(dur_span, UnitErrKind::DurMissing(unit)))
        } else {
            let mut num_span = dur_span;
            let mut graphs = UnicodeSegmentation::grapheme_indices(s, true).peekable();

            // parse sign
            let mut is_neg = false;
            let mut neg_span = None;
            if let Some((_, sign)) = graphs.peek() {
                let mut valid = false;
                if *sign == "+" {
                    valid = true;
                    is_neg = false;
                } else if *sign == "-" {
                    valid = true;
                    is_neg = true;
                    neg_span = Some(ByteSpan::new(dur_span.start, sign.len(), dur_span.src));
                }
                if valid {
                    num_span.shift_start_right(sign.len());
                }
            }

            if !allow_neg && is_neg {
                return Err(ParseErr::new(neg_span.unwrap(), ErrKind::Negative));
            }

            // find "." to distinguish whole from fractional part
            let mut int_span = num_span;
            let mut sub_span = None;
            if let Some((dot_idx, dot)) = graphs.find(|(_, chr)| *chr == ".") {
                let dot_span = ByteSpan::new(dot_idx, dot.len(), s);

                // adjust int_span
                int_span.len = dot_span.start - int_span.start;

                // adjust sub_span
                let tmp_sub_start = dot_span.start + dot_span.len;
                sub_span = Some(ByteSpan::new(
                    tmp_sub_start,
                    dur_span.len - tmp_sub_start,
                    s,
                ));
            }

            // parse int
            int_span.trim_whitespace();
            let mut ints = 0;
            if !int_span.get().is_empty() {
                ints = int_span
                    .get()
                    .parse::<u64>()
                    .map_err(|err| ParseErr::new(int_span, UnitErrKind::ParseInt { err, unit }))?;
            }

            // parse subs
            let mut subs: u32 = 0;
            if let Some(mut sub_span) = sub_span {
                sub_span.trim_whitespace();

                // TODO: can't specify full precision hours or minutes
                let places = 9; // u32::MAX digits
                subs = super::parse_frac(sub_span.get(), places).map_err(|err| match err {
                    ParseFracErr::ParseDigit { idx, len, err } => {
                        let mut span = sub_span;
                        span.shift_start_right(idx);
                        span.len = len;
                        ParseErr::new(span, UnitErrKind::ParseInt { err, unit })
                    }
                    ParseFracErr::NumeratorOverflow { idx: _ } => {
                        ParseErr::new(sub_span, UnitErrKind::DurOverflow(unit))
                    }
                })?;
            }

            // scale value based on unit
            let mut dur = Duration::new(ints, subs);
            dur = dur
                .checked_mul(match unit {
                    Unit::Second => 1,
                    Unit::Minute => u32::from(SEC_PER_MIN),
                    Unit::Hour => u32::from(SEC_PER_HOUR),
                })
                .ok_or(ParseErr::new(num_span, UnitErrKind::DurOverflow(unit)))?;

            Ok(ReadDur { dur, is_neg })
        }
    }
}
