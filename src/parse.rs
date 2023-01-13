// sw: terminal stopwatch
// copyright (C) 2022 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};
use unicode_segmentation::UnicodeSegmentation;

use core::fmt;
use core::num::ParseFloatError;
use core::time::{Duration, TryFromFloatSecsError};
use std::io::{self, Write};

const SEC_PER_MIN: u8 = 60;
const _MIN_PER_HOUR: u8 = 60;
const SEC_PER_HOUR: u16 = 3600;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Unit {
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

struct ByteSpan {
    start: usize,
    up_to: Option<usize>,
}

impl ByteSpan {
    #[must_use]
    #[inline]
    pub fn new(start: usize, up_to: impl Into<Option<usize>>) -> Self {
        Self {
            start,
            up_to: up_to.into(),
        }
    }
}

pub struct ParseErr<'a> {
    src: &'a str,
    span: ByteSpan,
    kind: ErrKind<'a>,
}

enum ErrKind<'a> {
    UnitMissing,
    UnitUnknown(&'a str),
    FloatMissing,
    Float(ParseFloatError),
    Dur(TryFromFloatSecsError),
}

impl<'a> ParseErr<'a> {
    #[inline]
    const fn new(src: &'a str, span: ByteSpan, kind: ErrKind<'a>) -> Self {
        Self { src, span, kind }
    }

    pub fn log(&self) -> io::Result<()> {
        let bufwtr = BufferWriter::stdout(ColorChoice::Auto);
        let mut buffer = bufwtr.buffer();
        let mut spec = ColorSpec::new();

        // write source text with span red and bold
        write!(&mut buffer, "{}", &self.src[..self.span.start])?;

        spec.set_fg(Some(Color::Red));
        spec.set_bold(true);
        buffer.set_color(&spec)?;
        if let Some(up_to) = self.span.up_to {
            write!(&mut buffer, "{}", &self.src[self.span.start..up_to])?;

            spec.clear();
            buffer.set_color(&spec)?;
            writeln!(&mut buffer, "{}", &self.src[up_to..])?;
        } else {
            writeln!(&mut buffer, "{}", &self.src[self.span.start..])?;
            spec.clear();
            buffer.set_color(&spec)?;
        }

        // write error message
        spec.set_fg(Some(Color::Red));
        buffer.set_color(&spec)?;
        writeln!(&mut buffer, "{self}")?;

        // write help message
        spec.clear();
        spec.set_dimmed(true);
        buffer.set_color(&spec)?;
        writeln!(&mut buffer, "{self:#}")?;

        // flush buffer
        spec.clear();
        buffer.set_color(&spec)?;

        bufwtr.print(&buffer)?;
        Ok(())
    }
}

impl fmt::Display for ParseErr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if f.alternate() {
            // help message
            match &self.kind {
                ErrKind::FloatMissing | ErrKind::UnitMissing | ErrKind::UnitUnknown(_) => write!(
                    f,
                    "note: use 's' for seconds, 'm' for minutes, and 'h' for hours"
                )?,
                ErrKind::Float(_) => (),
                ErrKind::Dur(_) => (),
            }
        } else {
            // error message
            match &self.kind {
                ErrKind::UnitMissing => write!(f, "missing unit")?,
                ErrKind::UnitUnknown(missing) => write!(f, "unrecognised unit '{missing}'")?,
                ErrKind::FloatMissing => write!(f, "unit given, but missing value")?,
                ErrKind::Float(err) => write!(f, "{err}")?,
                ErrKind::Dur(err) => write!(f, "{err}")?,
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReadDur {
    pub dur: Duration,
    pub is_neg: bool,
}

impl ReadDur {
    // TODO: support HH:MM:SS.ss format
    pub fn parse(s: &str) -> Result<Self, ParseErr> {
        // whitespace? + float + whitespace? + unit

        let mut graphs = UnicodeSegmentation::grapheme_indices(s, true).peekable();
        let maybe_unit = graphs.clone().last();
        if let Some(try_unit) = maybe_unit {
            if let Ok(unit) = Unit::from_grapheme(try_unit.1) {
                let mut up_to = 0;
                while let Some((idx, _)) = graphs.next() {
                    up_to = idx;
                    if graphs.peek().is_none() {
                        // skip last grapheme which is the unit
                        break;
                    }
                }
                let float_str = &s[..up_to].trim();

                if float_str.is_empty() {
                    return Err(ParseErr::new(
                        s,
                        ByteSpan::new(0, up_to),
                        ErrKind::FloatMissing,
                    ));
                }
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
                                s,
                                ByteSpan::new(0, up_to),
                                ErrKind::Dur(dur_err),
                            )),
                        }
                    }
                    Err(float_err) => Err(ParseErr::new(
                        s,
                        ByteSpan::new(0, up_to),
                        ErrKind::Float(float_err),
                    )),
                }
            } else {
                Err(ParseErr::new(
                    s,
                    ByteSpan::new(try_unit.0, None),
                    ErrKind::UnitUnknown(try_unit.1),
                ))
            }
        } else {
            Err(ParseErr::new(
                s,
                ByteSpan::new(0, None),
                ErrKind::UnitMissing,
            ))
        }
    }
}
