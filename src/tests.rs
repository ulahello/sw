// sw: terminal stopwatch (tests)
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

mod parse {
    mod frac {
        use crate::parse::{parse_frac, ParseFracErr};

        #[test]
        fn basic() {
            assert_eq!(parse_frac("1", 1), Ok(1));
            assert_eq!(parse_frac("2", 1), Ok(2));
            assert_eq!(parse_frac("23", 1), Ok(2));
            assert_eq!(parse_frac("23", 2), Ok(23));
            assert_eq!(parse_frac("2", 2), Ok(20));
            assert_eq!(
                parse_frac("24🪴21", 5),
                Err(ParseFracErr::ParseDigit {
                    idx: 2,
                    len: 4,
                    err: "g".parse::<u8>().unwrap_err()
                })
            );
            {
                let s = (u64::from(u32::MAX) + 1).to_string();
                assert_eq!(
                    parse_frac(&s, s.len() as _),
                    Err(ParseFracErr::NumeratorOverflow { idx: s.len() - 1 })
                );
            }
        }
    }

    mod unit {
        // TODO: test unit format

        use crate::parse::*;
        use core::time::Duration;

        #[test]
        fn whitespace() {
            let expect = Ok(ReadDur {
                dur: Duration::from_secs(1),
                is_neg: false,
            });
            assert_eq!(ReadDur::parse_as_unit(" 1s", true), expect);
            assert_eq!(ReadDur::parse_as_unit("1s ", true), expect);
            assert_eq!(ReadDur::parse_as_unit("1 s", true), expect);
            assert_eq!(ReadDur::parse_as_unit("1. s", true), expect);
            assert_eq!(ReadDur::parse_as_unit("1 . s", true), expect);
            assert_eq!(ReadDur::parse_as_unit("1 .s", true), expect);
        }

        #[test]
        fn overflow_bug() {
            assert_eq!(
                ReadDur::parse_as_unit("0.2s", true),
                Ok(ReadDur {
                    dur: Duration::from_millis(200),
                    is_neg: false,
                })
            );
        }
    }

    mod sw {
        // TODO: test subsecond parsing

        use crate::parse::*;
        use core::time::Duration;
        use sw::*;

        fn test<'a>(
            runs: impl Iterator<Item = (&'a [&'static str], Result<ReadDur, ParseErr<'static>>)>,
        ) {
            for (inputs, expect) in runs {
                for input in inputs {
                    assert_eq!(ReadDur::parse_as_sw(input, true), expect);
                }
            }
        }

        #[test]
        fn basic() {
            let runs: [(&[&'static str], Result<ReadDur, ParseErr<'static>>); 4] = [
                // TODO: negative variants are algorithmic from the positive runs
                (
                    &["3", ":3", "0:3", "::3", "0::3", ":0:3", "0:0:3"],
                    Ok(ReadDur {
                        dur: Duration::from_secs(3),
                        is_neg: false,
                    }),
                ),
                (
                    &["-3", "-:3", "-0:3", "-::3", "-0::3", "-:0:3", "-0:0:3"],
                    Ok(ReadDur {
                        dur: Duration::from_secs(3),
                        is_neg: true,
                    }),
                ),
                (
                    &["3:", ":3:", ":3:0", "0:3:", "0:3:0"],
                    Ok(ReadDur {
                        dur: Duration::from_secs(180),
                        is_neg: false,
                    }),
                ),
                (
                    &["-3:", "-:3:", "-:3:0", "-0:3:", "-0:3:0"],
                    Ok(ReadDur {
                        dur: Duration::from_secs(180),
                        is_neg: true,
                    }),
                ),
            ];
            test(runs.into_iter());
        }

        #[test]
        fn zero_dur_corner_cases() {
            let runs: [(&[&'static str], Result<ReadDur, ParseErr<'static>>); 2] = [
                (
                    &["", ":", ":.", "::", "::."],
                    Ok(ReadDur {
                        dur: Duration::ZERO,
                        is_neg: false,
                    }),
                ),
                (
                    &["-", "-:", "-:.", "-::", "-::."],
                    Ok(ReadDur {
                        dur: Duration::ZERO,
                        is_neg: true,
                    }),
                ),
            ];
            test(runs.into_iter());
        }

        #[test]
        fn whitespace_trimmed() {
            const S: &str = " 1:2    45  6 : 4 ";
            let mut lexer: Vec<_> = SwLexer::new(S).into_iter().collect();
            assert_eq!(
                lexer.pop(),
                Some(SwToken {
                    typ: SwTokenKind::Data,
                    span: ByteSpan::new(1, 1, S),
                })
            );
            assert_eq!(
                lexer.pop(),
                Some(SwToken {
                    typ: SwTokenKind::Colon,
                    span: ByteSpan::new(2, 1, S),
                })
            );
            assert_eq!(
                lexer.pop(),
                Some(SwToken {
                    typ: SwTokenKind::Data,
                    span: ByteSpan::new(3, 10, S),
                })
            );
            assert_eq!(
                lexer.pop(),
                Some(SwToken {
                    typ: SwTokenKind::Colon,
                    span: ByteSpan::new(14, 1, S),
                })
            );
            assert_eq!(
                lexer.pop(),
                Some(SwToken {
                    typ: SwTokenKind::Data,
                    span: ByteSpan::new(16, 1, S),
                })
            );
        }
    }
}
