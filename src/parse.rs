// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};
use unicode_segmentation::UnicodeSegmentation;

use core::num::{ParseFloatError, ParseIntError};
use core::time::{Duration, TryFromFloatSecsError};
use core::{fmt, ops};
use std::borrow::Cow;
use std::io::{self, Write};

const SEC_PER_MIN: u8 = 60;
const MIN_PER_HOUR: u8 = 60;
const SEC_PER_HOUR: u16 = 3600;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Second => "second",
            Self::Minute => "minute",
            Self::Hour => "hour",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ByteSpan {
    start: usize,
    up_to: Option<usize>,
}

impl ByteSpan {
    #[must_use]
    #[inline]
    pub fn new(start: usize, up_to: impl Into<Option<usize>>) -> Self {
        let up_to = up_to.into();
        if let Some(idx) = up_to {
            debug_assert!(start <= idx);
        }
        Self { start, up_to }
    }

    pub fn get<'a>(&self, s: &'a str) -> Option<&'a str> {
        if let Some(up_to) = self.up_to {
            if self.start == up_to {
                return None;
            }
            Some(&s[self.start..up_to])
        } else {
            Some(&s[self.start..])
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseErr<'a> {
    src: &'a str,
    span: ByteSpan,
    kind: ErrKind<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErrKind<'a> {
    /* unit format */
    UnitMissing,
    UnitUnknown(&'a str),
    FloatMissing,
    Float(ParseFloatError),
    Dur(TryFromFloatSecsError),

    /* sw format */
    ColonUnexpected,
    Int(ParseIntError),
    DurationOverflow(Group),
    SubsecondsTooLong,
    GroupExcess(Group),
}

impl<'a> ParseErr<'a> {
    #[inline]
    pub(crate) const fn new(src: &'a str, span: ByteSpan, kind: ErrKind<'a>) -> Self {
        Self { src, span, kind }
    }

    pub fn log(&self) -> io::Result<()> {
        let bufwtr = BufferWriter::stderr(ColorChoice::Auto);
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
        if let Some(msg) = self.help_message() {
            spec.clear();
            spec.set_dimmed(true);
            buffer.set_color(&spec)?;
            writeln!(&mut buffer, "note: {msg}")?;
        }

        // flush buffer
        spec.clear();
        buffer.set_color(&spec)?;

        bufwtr.print(&buffer)?;
        Ok(())
    }
}

impl ParseErr<'_> {
    fn help_message(&self) -> Option<Cow<'static, str>> {
        match &self.kind {
            ErrKind::UnitMissing | ErrKind::UnitUnknown(_) => {
                Some("use 's' for seconds, 'm' for minutes, and 'h' for hours".into())
            }

            ErrKind::ColonUnexpected => Some("there is no colon before hours".into()),
            ErrKind::DurationOverflow(_) => {
                Some("this duration is too large to be represented".into())
            }
            ErrKind::GroupExcess(group) => {
                Some(format!("{group} must be less than {}", group.max()).into())
            }

            ErrKind::Float(_)
            | ErrKind::FloatMissing
            | ErrKind::Dur(_)
            | ErrKind::Int(_)
            | ErrKind::SubsecondsTooLong => None,
        }
    }
}

impl fmt::Display for ParseErr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match &self.kind {
            ErrKind::UnitMissing => write!(f, "missing unit"),
            ErrKind::UnitUnknown(missing) => write!(f, "unrecognised unit '{missing}'"),
            ErrKind::FloatMissing => write!(f, "unit given, but missing value"),
            ErrKind::Float(err) => write!(f, "{err}"),
            ErrKind::Dur(err) => write!(f, "{err}"),

            ErrKind::ColonUnexpected => write!(f, "unexpected colon"),
            ErrKind::Int(err) => write!(f, "{err}"),
            ErrKind::DurationOverflow(group) => {
                write!(f, "duration oveflow while parsing {group}")
            }

            ErrKind::SubsecondsTooLong => write!(f, "too many characters in {}", Group::SecondsSub),
            ErrKind::GroupExcess(group) => write!(f, "value is out of range for {group}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadDur {
    pub dur: Duration,
    pub is_neg: bool,
}

impl ReadDur {
    pub fn parse(s: &str) -> Result<Self, ParseErr> {
        if s.as_bytes().contains(&b':') {
            Self::parse_as_sw(s)
        } else {
            Self::parse_as_unit(s)
        }
    }

    pub fn parse_as_unit(s: &str) -> Result<Self, ParseErr> {
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

    pub fn parse_as_sw(s: &str) -> Result<Self, ParseErr> {
        /* split up s into colon-separated groups */
        let (groups, is_neg) = {
            let mut groups = Groups::new();
            let mut cur = Group::SecondsSub;

            let mut prev_idx = None;

            // determine sign
            let mut first_idx = 0;
            let mut is_neg = false;
            let mut graphemes = UnicodeSegmentation::grapheme_indices(s, true);
            if let Some((_, chr)) = graphemes.next() {
                if chr == "-" {
                    is_neg = true;
                    first_idx = graphemes
                        .next()
                        // if there isn't anything after the "-", we dont want
                        // the group parsing to run, so we set the index to a
                        // value such that the loop never runs (assumes
                        // graphemes iterator uses take_while while index isn't
                        // below first_idx)
                        .map_or(usize::MAX, |(idx, _)| idx);
                }
            }

            let graphemes = UnicodeSegmentation::grapheme_indices(s, true)
                .rev()
                .take_while(|(idx, _)| first_idx <= *idx)
                .peekable();

            for (idx, chr) in graphemes {
                if groups[cur].is_none() {
                    groups[cur] = Some(ByteSpan::new(idx, prev_idx));
                }

                match cur {
                    Group::SecondsSub if chr == "." => {
                        let mut group = groups[cur].as_mut().unwrap();
                        if prev_idx.is_none() {
                            group.up_to = Some(idx);
                        }
                        group.start = prev_idx.unwrap_or(idx);
                        cur = Group::SecondsInt;
                        groups[cur] = Some(ByteSpan::new(idx, idx));
                    }

                    Group::SecondsSub if chr == ":" => {
                        groups[Group::SecondsInt] = groups[cur].take();
                        cur = Group::SecondsInt;
                        let mut group = groups[cur].as_mut().unwrap();
                        if prev_idx.is_none() {
                            group.up_to = Some(idx);
                        }
                        group.start = prev_idx.unwrap_or(idx);
                        cur = Group::Minutes;
                    }

                    Group::SecondsInt if chr == ":" => {
                        groups[cur].as_mut().unwrap().start = prev_idx.unwrap_or(idx);
                        cur = Group::Minutes;
                    }

                    Group::Minutes if chr == ":" => {
                        groups[cur].as_mut().unwrap().start = prev_idx.unwrap_or(idx);
                        cur = Group::Hours;
                    }

                    Group::Hours if chr == ":" => {
                        return Err(ParseErr::new(
                            s,
                            ByteSpan::new(idx, prev_idx),
                            ErrKind::ColonUnexpected,
                        ));
                    }

                    _ => {
                        groups[cur].as_mut().unwrap().start = idx;
                    }
                }

                prev_idx = Some(idx);
            }

            #[allow(unused_assignments)]
            if cur == Group::SecondsSub {
                groups[Group::SecondsInt] = groups[cur].take();
                cur = Group::SecondsInt;
            }

            (groups, is_neg)
        };

        /* parse groups as integers */
        let mut dur = Duration::ZERO;

        // secs (subs)
        if let Some(span) = groups[Group::SecondsSub] {
            if let Some(try_subs) = span.get(s) {
                let mut nanos: u32 = 0;
                let mut place: u32 = Group::SecondsSub.max().try_into().unwrap();
                for (idx, chr) in UnicodeSegmentation::grapheme_indices(try_subs, true) {
                    if place == 1 {
                        let mut err_span = span;
                        err_span.start = span.start + idx;
                        return Err(ParseErr::new(s, err_span, ErrKind::SubsecondsTooLong));
                    }
                    place /= 10;
                    match chr.parse::<u8>() {
                        Ok(digit) => {
                            debug_assert!(digit < 10);
                            nanos += u32::from(digit) * place;
                        }
                        Err(err) => return Err(ParseErr::new(s, span, ErrKind::Int(err))),
                    }
                }
                dur = dur
                    .checked_add(Duration::from_nanos(nanos.into()))
                    .ok_or_else(|| {
                        ParseErr::new(s, span, ErrKind::DurationOverflow(Group::SecondsSub))
                    })?;
            }
        }

        // secs (int)
        if let Some(span) = groups[Group::SecondsInt] {
            if let Some(try_secs) = span.get(s) {
                match try_secs.parse::<u64>() {
                    Ok(secs) => {
                        if secs >= Group::SecondsInt.max() {
                            return Err(ParseErr::new(
                                s,
                                span,
                                ErrKind::GroupExcess(Group::SecondsInt),
                            ));
                        }
                        dur = dur.checked_add(Duration::from_secs(secs)).ok_or_else(|| {
                            ParseErr::new(s, span, ErrKind::DurationOverflow(Group::SecondsInt))
                        })?;
                    }

                    Err(err) => return Err(ParseErr::new(s, span, ErrKind::Int(err))),
                }
            }
        }

        // mins
        if let Some(span) = groups[Group::Minutes] {
            if let Some(try_mins) = span.get(s) {
                match try_mins.parse::<u64>() {
                    Ok(mins) => {
                        if mins >= Group::Minutes.max() {
                            return Err(ParseErr::new(
                                s,
                                span,
                                ErrKind::GroupExcess(Group::Minutes),
                            ));
                        }
                        let secs = mins.checked_mul(SEC_PER_MIN.into()).ok_or_else(|| {
                            ParseErr::new(s, span, ErrKind::DurationOverflow(Group::Minutes))
                        })?;
                        dur = dur.checked_add(Duration::from_secs(secs)).ok_or_else(|| {
                            ParseErr::new(s, span, ErrKind::DurationOverflow(Group::Minutes))
                        })?;
                    }

                    Err(err) => return Err(ParseErr::new(s, span, ErrKind::Int(err))),
                }
            }
        }

        // hours
        if let Some(span) = groups[Group::Hours] {
            if let Some(try_hours) = span.get(s) {
                match try_hours.parse::<u64>() {
                    Ok(hours) => {
                        let secs = hours.checked_mul(SEC_PER_HOUR.into()).ok_or_else(|| {
                            ParseErr::new(s, span, ErrKind::DurationOverflow(Group::Hours))
                        })?;
                        dur = dur.checked_add(Duration::from_secs(secs)).ok_or_else(|| {
                            ParseErr::new(s, span, ErrKind::DurationOverflow(Group::Hours))
                        })?;
                    }

                    Err(err) => return Err(ParseErr::new(s, span, ErrKind::Int(err))),
                }
            }
        }

        Ok(Self { dur, is_neg })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Group {
    Hours,
    Minutes,
    SecondsInt,
    SecondsSub,
}

impl Group {
    pub(crate) fn max(self) -> u64 {
        match self {
            Self::Hours => u64::MAX / u64::from(SEC_PER_HOUR),
            Self::Minutes => MIN_PER_HOUR.into(),
            Self::SecondsInt => SEC_PER_MIN.into(),
            Self::SecondsSub => Duration::from_secs(1).as_nanos().try_into().unwrap(),
        }
    }
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Group::Hours => "hours",
            Group::Minutes => "minutes",
            Group::SecondsInt => "seconds",
            Group::SecondsSub => "subseconds",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Groups {
    pool: [Option<ByteSpan>; 4],
}

impl Groups {
    pub(crate) const fn new() -> Self {
        Self { pool: [None; 4] }
    }
}

impl ops::Index<Group> for Groups {
    type Output = Option<ByteSpan>;

    fn index(&self, idx: Group) -> &Self::Output {
        &self.pool[idx as usize]
    }
}

impl ops::IndexMut<Group> for Groups {
    fn index_mut(&mut self, idx: Group) -> &mut Self::Output {
        &mut self.pool[idx as usize]
    }
}
