// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use unicode_segmentation::UnicodeSegmentation;

use core::fmt;
use core::num::ParseFloatError;
use core::time::{Duration, TryFromFloatSecsError};

use super::{ByteSpan, ParseErr, ReadDur, Unit, SEC_PER_HOUR, SEC_PER_MIN};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum UnitErrKind<'s> {
    UnitMissing,
    UnitUnknown(&'s str),
    FloatMissing(Unit),
    Float { err: ParseFloatError, unit: Unit },
    Dur(TryFromFloatSecsError),
}

impl UnitErrKind<'_> {
    pub(crate) fn has_help_message(&self) -> bool {
        match self {
            Self::UnitMissing
            | Self::Float { .. }
            | Self::FloatMissing(_)
            | Self::UnitUnknown(_) => true,

            Self::Dur(_) => false,
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
                Self::Float { err: _, unit } | Self::FloatMissing(unit) => {
                    write!(f, "expected the number of {unit}s")
                }

                Self::Dur(_) => {
                    unreachable!()
                }
            }
        } else {
            match self {
                Self::UnitMissing => write!(f, "missing unit"),
                Self::UnitUnknown(unk) => write!(f, "unrecognised unit '{unk}'"),
                Self::FloatMissing(_) => write!(f, "unit given, but missing value"),
                Self::Float { err, unit: _ } => write!(f, "{err}"),
                Self::Dur(err) => write!(f, "{err}"),
            }
        }
    }
}

impl ReadDur {
    pub fn parse_as_unit(s: &str) -> Result<Self, ParseErr> {
        // whitespace? + float + whitespace? + unit

        let (try_unit_idx, try_unit) = UnicodeSegmentation::grapheme_indices(s, true)
            .peekable()
            .last()
            .ok_or(ParseErr::new(
                ByteSpan::new(0, s.len(), s),
                UnitErrKind::UnitMissing,
            ))?;

        let unit = Unit::from_grapheme(try_unit).ok().ok_or(ParseErr::new(
            ByteSpan::new(try_unit_idx, try_unit.len(), s),
            UnitErrKind::UnitUnknown(try_unit),
        ))?;

        let len = try_unit_idx;
        let float_str = &s[..len].trim();
        if float_str.is_empty() {
            return Err(ParseErr::new(
                ByteSpan::new(0, len, s),
                UnitErrKind::FloatMissing(unit),
            ));
        }

        // TODO: don't use floats
        match float_str.parse::<f64>() {
            Ok(mut float) => {
                match unit {
                    Unit::Second => (),
                    Unit::Minute => float *= f64::from(SEC_PER_MIN),
                    Unit::Hour => float *= f64::from(SEC_PER_HOUR),
                }
                let secs = float.abs();

                match Duration::try_from_secs_f64(secs) {
                    Ok(dur) => {
                        let is_neg = float.is_sign_negative();
                        Ok(ReadDur { dur, is_neg })
                    }
                    Err(dur_err) => Err(ParseErr::new(
                        ByteSpan::new(0, len, s),
                        UnitErrKind::Dur(dur_err),
                    )),
                }
            }

            Err(float_err) => Err(ParseErr::new(
                ByteSpan::new(0, len, s),
                UnitErrKind::Float {
                    err: float_err,
                    unit,
                },
            )),
        }
    }
}
