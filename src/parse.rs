// sw: terminal stopwatch
// copyright (C) 2022 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use crate::shell::log;

use termcolor::Color;

use core::fmt;
use core::num::ParseFloatError;
use core::time::{Duration, TryFromFloatSecsError};
use std::io;

use ErrKind::*;

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

pub struct ParseErr {
    span: (usize, usize),
    kind: ErrKind,
}

enum ErrKind {
    UnitMissing,
    UnitUnknown(char),
    FloatMissing,
    Float(ParseFloatError),
    Dur(TryFromFloatSecsError),
}

impl ParseErr {
    #[inline]
    const fn new(span: (usize, usize), kind: ErrKind) -> Self {
        Self { span, kind }
    }

    pub fn log(&self, where_does_input_start: usize) -> io::Result<()> {
        log(
            Color::Red,
            format!(
                "{}{}{}",
                " ".repeat(where_does_input_start),
                " ".repeat(self.span.0.min(self.span.1)),
                "^".repeat(self.span.0.abs_diff(self.span.1) + 1)
            ),
        )?;
        log(Color::Red, self)?;

        Ok(())
    }
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match &self.kind {
            ErrKind::UnitMissing => write!(f, "missing unit")?,
            ErrKind::UnitUnknown(missing) => write!(f, "unrecognised unit '{missing}'")?,
            ErrKind::FloatMissing => write!(f, "unit given, but missing value")?,
            ErrKind::Float(err) => write!(f, "{err}")?,
            ErrKind::Dur(err) => write!(f, "{err}")?,
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
        let len = s.chars().count();
        let mut chars = s.chars().rev();
        let maybe_unit: Option<char> = chars.next();
        let try_val: String = chars.rev().collect();

        if let Some(try_unit) = maybe_unit {
            match Unit::from_char(try_unit) {
                Ok(unit) => {
                    let span = (0, len.saturating_sub(2));
                    match try_val.trim().parse::<f64>() {
                        Ok(mut val) => {
                            let is_neg = val.is_sign_negative();
                            match unit {
                                Unit::Second => (),
                                Unit::Minute => val *= 60.0,
                                Unit::Hour => val *= 60.0 * 60.0,
                            }
                            match Duration::try_from_secs_f64(val.abs()) {
                                Ok(dur) => Ok(ReadDur { dur, is_neg }),
                                Err(err) => Err(ParseErr::new(span, Dur(err))),
                            }
                        }

                        Err(err) => {
                            if try_val.is_empty() {
                                Err(ParseErr::new(span, FloatMissing))
                            } else {
                                Err(ParseErr::new(span, Float(err)))
                            }
                        }
                    }
                }

                Err(err) => Err(ParseErr::new(
                    (len.saturating_sub(1), len.saturating_sub(1)),
                    UnitUnknown(err),
                )),
            }
        } else {
            Err(ParseErr::new((0, 0), UnitMissing))
        }
    }
}
