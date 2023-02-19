// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};
use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

use core::iter::{Peekable, Rev};
use core::num::{ParseFloatError, ParseIntError};
use core::time::{Duration, TryFromFloatSecsError};
use core::{fmt, ops};
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
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseErr<'s> {
    src: &'s str,
    span: ByteSpan<'s>,
    kind: ErrKind<'s>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErrKind<'s> {
    /* unit format */
    UnitUnitMissing,
    UnitUnitUnknown(&'s str),
    UnitFloatMissing,
    UnitFloat(ParseFloatError),
    UnitDur(TryFromFloatSecsError),

    /* sw format */
    SwUnexpectedColon,
    SwUnexpectedDot(Group),
    SwUnexpectedSign { is_neg: bool },
    SwInt { group: Group, err: ParseIntError },
    SwDurationOverflow(Group),
    SwSubsecondsTooLong,
    SwGroupExcess(Group),
}

impl<'s> ParseErr<'s> {
    #[inline]
    pub(crate) const fn new(span: ByteSpan<'s>, kind: ErrKind<'s>) -> Self {
        Self {
            src: span.src,
            span,
            kind,
        }
    }

    pub fn log(&self) -> io::Result<()> {
        let bufwtr = BufferWriter::stderr(ColorChoice::Auto);
        let mut buffer = bufwtr.buffer();
        let mut spec = ColorSpec::new();
        buffer.set_color(&spec)?;

        /* write source text with span red and bold */
        // text before span
        write!(&mut buffer, "{}", self.span.get_before())?;

        // red span text
        spec.set_fg(Some(Color::Red));
        spec.set_bold(true);
        buffer.set_color(&spec)?;

        write!(&mut buffer, "{}", self.span.get())?;

        spec.clear();
        buffer.set_color(&spec)?;

        // text after span
        writeln!(&mut buffer, "{}", self.span.get_after())?;

        /* write error message */
        spec.set_fg(Some(Color::Red));
        buffer.set_color(&spec)?;
        writeln!(&mut buffer, "{self}")?;

        /* write help message */
        if self.has_help_message() {
            spec.clear();
            spec.set_dimmed(true);
            buffer.set_color(&spec)?;
            write!(&mut buffer, "note: ")?;
            self.help_message(&mut buffer)?;
            writeln!(&mut buffer)?;
        }

        /* flush buffer */
        spec.clear();
        buffer.set_color(&spec)?;
        bufwtr.print(&buffer)?;

        Ok(())
    }
}

impl ParseErr<'_> {
    fn has_help_message(&self) -> bool {
        match &self.kind {
            ErrKind::UnitUnitMissing
            | ErrKind::UnitUnitUnknown(_)
            | ErrKind::SwUnexpectedColon
            | ErrKind::SwUnexpectedDot(_)
            | ErrKind::SwDurationOverflow(_)
            | ErrKind::SwInt { .. }
            | ErrKind::SwGroupExcess(_) => true,

            ErrKind::UnitFloat(_)
            | ErrKind::UnitFloatMissing
            | ErrKind::UnitDur(_)
            | ErrKind::SwUnexpectedSign { .. }
            | ErrKind::SwSubsecondsTooLong => false,
        }
    }

    fn help_message(&self, f: &mut impl io::Write) -> io::Result<()> {
        match &self.kind {
            ErrKind::UnitUnitMissing | ErrKind::UnitUnitUnknown(_) => {
                write!(f, "use 's' for seconds, 'm' for minutes, and 'h' for hours")?;
            }

            ErrKind::SwUnexpectedColon => {
                write!(f, "there is no colon before {}", Group::Hours)?;
            }
            ErrKind::SwUnexpectedDot(group) => {
                assert_ne!(*group, Group::SecondsSub);
                if *group == Group::SecondsInt {
                    write!(
                        f,
                        "decimal point was already given for {}",
                        Group::SecondsSub
                    )?;
                } else {
                    write!(
                        f,
                        "found in {group}, but only {} can have fractional values",
                        Group::SecondsInt
                    )?;
                }
            }
            ErrKind::SwDurationOverflow(_) => {
                write!(f, "this duration is too large to be represented")?;
            }
            ErrKind::SwGroupExcess(group) => {
                write!(f, "{group} must be less than {}", group.max())?;
            }
            ErrKind::SwInt { group, err: _ } => write!(f, "{group} are parsed as an integer")?,

            ErrKind::UnitFloat(_)
            | ErrKind::UnitFloatMissing
            | ErrKind::UnitDur(_)
            | ErrKind::SwUnexpectedSign { .. }
            | ErrKind::SwSubsecondsTooLong => unreachable!(),
        }
        Ok(())
    }
}

impl fmt::Display for ParseErr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match &self.kind {
            ErrKind::UnitUnitMissing => write!(f, "missing unit"),
            ErrKind::UnitUnitUnknown(unk) => write!(f, "unrecognised unit '{unk}'"),
            ErrKind::UnitFloatMissing => write!(f, "unit given, but missing value"),
            ErrKind::UnitFloat(err) => write!(f, "{err}"),
            ErrKind::UnitDur(err) => write!(f, "{err}"),

            ErrKind::SwUnexpectedColon => write!(f, "unexpected colon"),
            ErrKind::SwUnexpectedDot(_) => write!(f, "unexpected decimal point"),
            ErrKind::SwUnexpectedSign { is_neg: _ } => {
                write!(f, "sign must be given at the beginning")
            }
            ErrKind::SwInt { group: _, err } => write!(f, "{err}"),
            ErrKind::SwDurationOverflow(group) => {
                write!(f, "duration oveflow while parsing {group}")
            }

            ErrKind::SwSubsecondsTooLong => {
                write!(f, "too many characters in {}", Group::SecondsSub)
            }
            ErrKind::SwGroupExcess(group) => write!(f, "value is out of range for {group}"),
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

        if let Some((try_unit_idx, try_unit)) = UnicodeSegmentation::grapheme_indices(s, true)
            .peekable()
            .last()
        {
            if let Ok(unit) = Unit::from_grapheme(try_unit) {
                let len = try_unit_idx;
                let float_str = &s[..len].trim();
                if float_str.is_empty() {
                    return Err(ParseErr::new(
                        ByteSpan::new(0, len, s),
                        ErrKind::UnitFloatMissing,
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
                                ByteSpan::new(0, len, s),
                                ErrKind::UnitDur(dur_err),
                            )),
                        }
                    }

                    Err(float_err) => Err(ParseErr::new(
                        ByteSpan::new(0, len, s),
                        ErrKind::UnitFloat(float_err),
                    )),
                }
            } else {
                Err(ParseErr::new(
                    ByteSpan::new(try_unit_idx, try_unit.len(), s),
                    ErrKind::UnitUnitUnknown(try_unit),
                ))
            }
        } else {
            Err(ParseErr::new(
                ByteSpan::new(0, s.len(), s),
                ErrKind::UnitUnitMissing,
            ))
        }
    }

    pub fn parse_as_sw(s: &str) -> Result<Self, ParseErr> {
        /* split string into groups of hours, minutes, etc */
        let (groups, is_neg): (Groups, bool) = {
            // NOTE: the lexer scans IN REVERSE
            let mut lexer = SwLexer::new(s).peekable();
            let mut cur = Group::SecondsSub;
            let mut groups = Groups::new(s);
            let mut is_neg = None;
            while let Some(token) = lexer.next() {
                match (cur, token.typ) {
                    (Group::SecondsSub, SwTokenKind::Colon) => {
                        // turns out the cur has been SecondsInt this whole
                        // time!
                        let tmp = groups[cur];
                        groups[cur] = ByteSpan::new(0, 0, s);
                        cur = Group::SecondsInt;
                        groups[cur] = tmp;

                        // now that we're SecondsInt and encountering a colon,
                        // transition.
                        cur = Group::Minutes;
                    }
                    (Group::SecondsSub, SwTokenKind::Dot) => cur = Group::SecondsInt,
                    (Group::SecondsSub, SwTokenKind::Data) => {
                        // NOTE: the question im asking is: "are there are no
                        // colons after this current token?" the way im asking
                        // it is "is this the last token or is the next token a
                        // sign, regardless of whether its the last because
                        // signs are only correctly parsed as the last"
                        let next_typ = lexer.peek().map(|token| token.typ);
                        if next_typ.is_none()
                            || next_typ == Some(SwTokenKind::Pos)
                            || next_typ == Some(SwTokenKind::Neg)
                        {
                            cur = Group::SecondsInt;
                        }
                        groups[cur] = token.span;
                    }

                    (Group::SecondsInt, SwTokenKind::Colon) => cur = Group::Minutes,

                    (Group::Minutes, SwTokenKind::Colon) => cur = Group::Hours,

                    (Group::Hours, SwTokenKind::Colon) => {
                        return Err(ParseErr::new(token.span, ErrKind::SwUnexpectedColon));
                    }

                    (_, SwTokenKind::Data) => groups[cur] = token.span,
                    (_, SwTokenKind::Dot) => {
                        return Err(ParseErr::new(token.span, ErrKind::SwUnexpectedDot(cur)));
                    }

                    (_, SwTokenKind::Pos) => {
                        if lexer.peek().is_none() {
                            is_neg = Some(false);
                        } else {
                            return Err(ParseErr::new(
                                token.span,
                                ErrKind::SwUnexpectedSign { is_neg: false },
                            ));
                        }
                    }
                    (_, SwTokenKind::Neg) => {
                        if lexer.peek().is_none() {
                            is_neg = Some(true);
                        } else {
                            return Err(ParseErr::new(
                                token.span,
                                ErrKind::SwUnexpectedSign { is_neg: true },
                            ));
                        }
                    }
                }
            }
            (groups, is_neg.unwrap_or(false))
        };

        /* parse group substrings into an actual duration */
        let mut dur = Duration::ZERO;

        // hours, minutes, seconds (whole)
        for (group, sec_per_unit) in [
            (Group::Hours, u64::from(SEC_PER_HOUR)),
            (Group::Minutes, u64::from(SEC_PER_MIN)),
            (Group::SecondsInt, 1),
        ] {
            let span = groups[group];
            let to_parse = span.get().trim();
            /* NOTE: we're trimming after we get the span, meaning the to_parse
             * doesn't reflect the span. */
            if !to_parse.is_empty() {
                match to_parse.parse::<u64>() {
                    Ok(units) => {
                        if units >= group.max() {
                            return Err(ParseErr::new(span, ErrKind::SwGroupExcess(group)));
                        }
                        let secs = units.checked_mul(sec_per_unit).ok_or_else(|| {
                            ParseErr::new(span, ErrKind::SwDurationOverflow(group))
                        })?;
                        dur = dur.checked_add(Duration::from_secs(secs)).ok_or_else(|| {
                            ParseErr::new(span, ErrKind::SwDurationOverflow(group))
                        })?;
                    }

                    Err(err) => return Err(ParseErr::new(span, ErrKind::SwInt { group, err })),
                }
            }
        }

        // subseconds
        {
            let group = Group::SecondsSub;
            let span = groups[group];
            let to_parse = span.get();
            if !to_parse.trim().is_empty() {
                let mut nanos: u32 = 0;
                let mut place: u32 = group.max().try_into().unwrap();
                let mut graphs = UnicodeSegmentation::graphemes(to_parse, true).peekable();

                // skip whitespace but maintain accurate span
                let mut err_span = span;
                while let Some(grapheme) = graphs.peek() {
                    if grapheme.chars().all(char::is_whitespace) {
                        err_span.shift_start_right(grapheme.len());
                        graphs.next();
                    } else {
                        break;
                    }
                }

                for chr in graphs {
                    if place == 1 {
                        return Err(ParseErr::new(err_span, ErrKind::SwSubsecondsTooLong));
                    }
                    assert!(place % 10 == 0, "{place} must be divisible by 10");
                    place /= 10;

                    err_span.shift_start_right(chr.len());

                    match chr.parse::<u8>() {
                        Ok(digit) => {
                            assert!(digit < 10);
                            nanos += u32::from(digit) * place;
                        }
                        Err(err) => return Err(ParseErr::new(span, ErrKind::SwInt { group, err })),
                    }
                }
                dur = dur
                    .checked_add(Duration::from_nanos(nanos.into()))
                    .ok_or_else(|| ParseErr::new(span, ErrKind::SwDurationOverflow(group)))?;
            }
        }

        Ok(Self { dur, is_neg })
    }
}

struct SwLexer<'s> {
    content: Peekable<Rev<GraphemeIndices<'s>>>,
    s: &'s str,
}

impl<'s> SwLexer<'s> {
    pub(crate) fn new(s: &'s str) -> Self {
        Self {
            content: UnicodeSegmentation::grapheme_indices(s, true)
                .rev()
                .peekable(),
            s,
        }
    }
}

impl<'s> Iterator for SwLexer<'s> {
    type Item = SwToken<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = loop {
            let next = self.content.next()?;
            if !next.1.chars().all(char::is_whitespace) {
                // only yield non-whitespace grapheme. Data still may contain
                // leading whitespace, but this prevents a Data token being
                // yielded that only contains whitespace
                break next;
            }
        };
        let mut span = ByteSpan::new(next.0, next.1.len(), self.s);
        match next.1 {
            ":" => Some(SwToken {
                typ: SwTokenKind::Colon,
                span,
            }),
            "." => Some(SwToken {
                typ: SwTokenKind::Dot,
                span,
            }),
            "+" => Some(SwToken {
                typ: SwTokenKind::Pos,
                span,
            }),
            "-" => Some(SwToken {
                typ: SwTokenKind::Neg,
                span,
            }),
            _ => {
                while let Some(d_next) = self.content.peek() {
                    match d_next.1 {
                        // TODO: handling single grapheme tokens in two places
                        ":" | "." | "+" | "-" => break,
                        _ => {
                            span.shift_start_left(d_next.1.len());
                            self.content.next();
                            continue;
                        }
                    }
                }
                Some(SwToken {
                    typ: SwTokenKind::Data,
                    span,
                })
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SwTokenKind {
    Colon,
    Dot,
    Pos,
    Neg,
    Data,
}

#[derive(Debug)]
struct SwToken<'s> {
    typ: SwTokenKind,
    span: ByteSpan<'s>,
}

#[derive(Debug)]
struct Groups<'s>([ByteSpan<'s>; 4]);

impl<'s> Groups<'s> {
    pub(crate) fn new(s: &'s str) -> Self {
        Self([ByteSpan::new(0, 0, s); 4])
    }
}

impl<'s> ops::Index<Group> for Groups<'s> {
    type Output = ByteSpan<'s>;

    fn index(&self, idx: Group) -> &Self::Output {
        &self.0[idx as usize]
    }
}

impl<'s> ops::IndexMut<Group> for Groups<'s> {
    fn index_mut(&mut self, idx: Group) -> &mut Self::Output {
        &mut self.0[idx as usize]
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
            Self::Hours => u64::MAX / u64::from(SEC_PER_HOUR) + 1,
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
