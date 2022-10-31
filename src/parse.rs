// sw: terminal stopwatch
// copyright (C) 2022 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use core::fmt;
use core::num::ParseFloatError;
use core::time::{Duration, TryFromFloatSecsError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Unit {
    Second,
    Minute,
    Hour,
}

impl Unit {
    pub const fn from_char(chr: char) -> Result<Self, char> {
        match chr {
            's' => Ok(Self::Second),
            'm' => Ok(Self::Minute),
            'h' => Ok(Self::Hour),
            unk => Err(unk),
        }
    }
}

pub enum ParseErr {
    UnitMissing,
    UnitUnknown(char),
    FloatMissing,
    Float(ParseFloatError),
    Dur(TryFromFloatSecsError),
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::UnitMissing => write!(f, "missing unit")?,
            Self::UnitUnknown(missing) => write!(f, "unrecognised unit '{missing}'")?,
            Self::FloatMissing => write!(f, "unit given, but missing value")?,
            Self::Float(err) => write!(f, "{err}")?,
            Self::Dur(err) => write!(f, "{err}")?,
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadDur {
    pub dur: Duration,
    pub is_neg: bool,
}

impl ReadDur {
    pub fn parse(s: &str) -> Result<Self, ParseErr> {
        let mut chars = s.chars().rev();
        let maybe_unit = chars.next();
        let try_val: String = chars.rev().collect();

        if let Some(try_unit) = maybe_unit {
            match Unit::from_char(try_unit) {
                Ok(unit) => match try_val.trim().parse::<f64>() {
                    Ok(mut val) => {
                        let is_neg = val.is_sign_negative();
                        match unit {
                            Unit::Second => (),
                            Unit::Minute => val *= 60.0,
                            Unit::Hour => val *= 60.0 * 60.0,
                        }
                        match Duration::try_from_secs_f64(val.abs()) {
                            Ok(dur) => Ok(ReadDur { dur, is_neg }),
                            Err(err) => Err(ParseErr::Dur(err)),
                        }
                    }

                    Err(err) => {
                        if try_val.is_empty() {
                            Err(ParseErr::FloatMissing)
                        } else {
                            Err(ParseErr::Float(err))
                        }
                    }
                },

                Err(err) => Err(ParseErr::UnitUnknown(err)),
            }
        } else {
            Err(ParseErr::UnitMissing)
        }
    }
}
