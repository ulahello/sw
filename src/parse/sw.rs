// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

use core::iter::{Peekable, Rev};
use core::num::{IntErrorKind, ParseIntError};
use core::time::Duration;
use core::{fmt, ops};

use super::{
    ByteSpan, ErrKind, ParseErr, ParseFracErr, ReadDur, MIN_PER_HOUR, SEC_PER_HOUR, SEC_PER_MIN,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SwErrKind {
    UnexpectedColon,
    UnexpectedDot(Group),
    UnexpectedSign { is_neg: bool },
    Int { group: Group, err: ParseIntError },
    DurationOverflow(Group),
}

impl SwErrKind {
    pub(crate) fn has_help_message(&self) -> bool {
        match self {
            Self::UnexpectedColon
            | Self::UnexpectedDot(_)
            | Self::DurationOverflow(_)
            | Self::Int { .. } => true,

            Self::UnexpectedSign { .. } => false,
        }
    }
}

impl fmt::Display for SwErrKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if f.alternate() {
            match self {
                Self::UnexpectedColon => {
                    write!(f, "there is no colon before {}", Group::Hours)
                }
                Self::UnexpectedDot(group) => {
                    debug_assert_ne!(*group, Group::SecondsSub);
                    if *group == Group::SecondsInt {
                        write!(
                            f,
                            "decimal point was already given for {}",
                            Group::SecondsSub
                        )
                    } else {
                        write!(
                            f,
                            "found in {group}, but only {} can have fractional values",
                            Group::SecondsInt
                        )
                    }
                }
                Self::DurationOverflow(_) => {
                    write!(f, "this duration is too large to be represented")
                }
                Self::Int { group, err: _ } => write!(f, "{group} are parsed as an integer"),

                Self::UnexpectedSign { .. } => {
                    unreachable!()
                }
            }
        } else {
            match self {
                Self::UnexpectedColon => write!(f, "unexpected colon"),
                Self::UnexpectedDot(_) => write!(f, "unexpected decimal point"),
                Self::UnexpectedSign { is_neg: _ } => {
                    write!(f, "sign must be given at the beginning")
                }
                Self::Int { group: _, err } => write!(f, "{err}"),
                Self::DurationOverflow(group) => {
                    write!(f, "duration overflow while parsing {group}")
                }
            }
        }
    }
}

impl ReadDur {
    pub fn parse_as_sw(s: &str, allow_neg: bool) -> Result<Self, ParseErr> {
        /* split string into groups of hours, minutes, etc */
        let mut neg_span = None;
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
                        return Err(ParseErr::new(token.span, SwErrKind::UnexpectedColon));
                    }

                    (_, SwTokenKind::Data) => groups[cur] = token.span,
                    (_, SwTokenKind::Dot) => {
                        return Err(ParseErr::new(token.span, SwErrKind::UnexpectedDot(cur)));
                    }

                    (_, SwTokenKind::Pos) => {
                        if lexer.peek().is_none() {
                            is_neg = Some(false);
                        } else {
                            return Err(ParseErr::new(
                                token.span,
                                SwErrKind::UnexpectedSign { is_neg: false },
                            ));
                        }
                    }
                    (_, SwTokenKind::Neg) => {
                        if lexer.peek().is_none() {
                            is_neg = Some(true);
                            neg_span = Some(token.span);
                        } else {
                            return Err(ParseErr::new(
                                token.span,
                                SwErrKind::UnexpectedSign { is_neg: true },
                            ));
                        }
                    }
                }
            }
            (groups, is_neg.unwrap_or(false))
        };

        if !allow_neg && is_neg {
            return Err(ParseErr::new(neg_span.unwrap(), ErrKind::Negative));
        }

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
                        let secs = units.checked_mul(sec_per_unit).ok_or_else(|| {
                            ParseErr::new(span, SwErrKind::DurationOverflow(group))
                        })?;
                        dur = dur.checked_add(Duration::from_secs(secs)).ok_or_else(|| {
                            ParseErr::new(span, SwErrKind::DurationOverflow(group))
                        })?;
                    }

                    Err(err) => return Err(ParseErr::new(span, SwErrKind::Int { group, err })),
                }
            }
        }

        // subseconds
        {
            let group = Group::SecondsSub;
            let span = groups[group];
            let to_parse = span.get();
            if !to_parse.trim().is_empty() {
                let nanos =
                    super::parse_frac(to_parse, crate::MAX_NANOS_CHARS).map_err(|frac_err| {
                        match frac_err {
                            ParseFracErr::ParseDigit { idx, len, err } => {
                                let mut span = span;
                                span.shift_start_right(idx);
                                span.len = len;
                                debug_assert_ne!(*err.kind(), IntErrorKind::PosOverflow);
                                ParseErr::new(span, SwErrKind::Int { group, err })
                            }
                            ParseFracErr::NumeratorOverflow { .. } => {
                                unreachable!("max nanosecond has 9 characters, max u32 has 10")
                            }
                        }
                    })?;
                if u64::from(nanos) >= group.max() {
                    unreachable!("max nanosecond has 9 characters. add 1 to max and it has 10 characters. that case is checked previously.");
                }
                dur = dur
                    .checked_add(Duration::from_nanos(nanos.into()))
                    .ok_or_else(|| ParseErr::new(span, SwErrKind::DurationOverflow(group)))?;
            }
        }

        Ok(Self { dur, is_neg })
    }
}

pub(crate) struct SwLexer<'s> {
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

    fn advance(content: &mut Peekable<Rev<GraphemeIndices<'s>>>) -> Option<(usize, &'s str)> {
        loop {
            let next = content.next()?;
            if !next.1.chars().all(char::is_whitespace) {
                // only yield non-whitespace grapheme. this prevents prevents
                // Data from having trailing whitespace.
                break Some(next);
            }
        }
    }

    fn peek<'a>(
        content: &'a mut Peekable<Rev<GraphemeIndices<'s>>>,
    ) -> Option<&'a (usize, &'s str)> {
        content.peek()
    }

    fn single_token(next: &str) -> Option<SwTokenKind> {
        match next {
            ":" => Some(SwTokenKind::Colon),
            "." => Some(SwTokenKind::Dot),
            "+" => Some(SwTokenKind::Pos),
            "-" => Some(SwTokenKind::Neg),
            _ => None,
        }
    }
}

impl<'s> Iterator for SwLexer<'s> {
    type Item = SwToken<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = Self::advance(&mut self.content)?;
        let mut span = ByteSpan::new(next.0, next.1.len(), self.s);
        if let Some(typ) = Self::single_token(next.1) {
            Some(SwToken { typ, span })
        } else {
            let mut bytes_ignored = 0;
            while let Some(d_next) = Self::peek(&mut self.content) {
                if Self::single_token(d_next.1).is_some() {
                    break;
                }
                // ignore leading whitespace
                if d_next.1.chars().all(char::is_whitespace) {
                    bytes_ignored += d_next.1.len();
                } else {
                    // oops, not leading whitespace. add all the bytes we ignored to the span.
                    span.shift_start_left(d_next.1.len() + bytes_ignored);
                    bytes_ignored = 0;
                }
                self.content.next();
            }
            Some(SwToken {
                typ: SwTokenKind::Data,
                span,
            })
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SwTokenKind {
    Colon,
    Dot,
    Pos,
    Neg,
    Data,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SwToken<'s> {
    pub(crate) typ: SwTokenKind,
    pub(crate) span: ByteSpan<'s>,
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
    pub(crate) const fn max(self) -> u64 {
        match self {
            Self::Hours => u64::MAX / SEC_PER_HOUR as u64 + 1,
            Self::Minutes => MIN_PER_HOUR as _,
            Self::SecondsInt => SEC_PER_MIN as _,
            #[allow(clippy::cast_possible_truncation)]
            Self::SecondsSub => Duration::from_secs(1).as_nanos() as _,
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
