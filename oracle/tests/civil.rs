//! `civil::NaiveDateTime` against chrono 0.4.45's `NaiveDateTime`: what it writes (serde,
//! `Debug`, `Display`, `timestamp_millis`) for every value in jiff's civil range, and what its
//! `FromStr` and serde visitor accept and reject over a grammar of near-valid strings.
//!
//! Two differences are by design and checked as such: a second of `60` loads as the next
//! minute's second 0 (chrono keeps a leap second), and a value chrono reads that jiff's civil
//! range can't hold is an "input is out of range" error with `is_beyond_range()`.

#[path = "../../tests/civil_goldens/naive.rs"]
mod goldens;

use chelabase_time::civil::NaiveDateTime;
use chelabase_time::{Timestamp, TimestampExt};
use chrono::{DateTime, NaiveDate, TimeDelta, Timelike, Utc};
use proptest::prelude::*;
use serde_json::Value;

type Chrono = chrono::NaiveDateTime;

fn civil_min() -> Chrono {
    NaiveDate::from_ymd_opt(-9999, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
}

fn civil_max() -> Chrono {
    NaiveDate::from_ymd_opt(9999, 12, 31)
        .unwrap()
        .and_hms_nano_opt(23, 59, 59, 999_999_999)
        .unwrap()
}

/// What `NaiveDateTime` must do with a string, derived from what chrono does with it.
#[derive(Debug, PartialEq)]
enum Expected {
    /// chrono's value (its leap second carried into the next second, the one exception).
    Value(Chrono),
    /// chrono reads it, jiff's civil range can't hold it.
    Beyond,
    /// chrono's error message.
    Rejected(String),
}

fn expected(s: &str) -> Expected {
    match s.parse::<Chrono>() {
        Err(e) => Expected::Rejected(e.to_string()),
        Ok(c) => {
            let carried = if c.nanosecond() >= 1_000_000_000 {
                c.with_nanosecond(c.nanosecond() - 1_000_000_000)
                    .and_then(|c| c.checked_add_signed(TimeDelta::seconds(1)))
            } else {
                Some(c)
            };
            match carried {
                Some(c) if civil_min() <= c && c <= civil_max() => Expected::Value(c),
                _ => Expected::Beyond,
            }
        }
    }
}

/// Every form of a value, compared with chrono's.
fn assert_same_value(ours: NaiveDateTime, chrono: Chrono, context: &str) {
    assert_eq!(
        serde_json::to_string(&ours).unwrap(),
        serde_json::to_string(&chrono).unwrap(),
        "serde: {context:?}"
    );
    assert_eq!(format!("{ours:?}"), format!("{chrono:?}"), "{context:?}");
    assert_eq!(ours.to_string(), chrono.to_string(), "{context:?}");
    assert_eq!(
        ours.timestamp_millis(),
        chrono.and_utc().timestamp_millis(),
        "{context:?}"
    );
}

/// `FromStr` and the serde visitor against chrono, for one string.
fn assert_parses_as_chrono(s: &str) {
    let from_str = s.parse::<NaiveDateTime>();
    let serde = serde_json::from_value::<NaiveDateTime>(Value::String(s.to_owned()));
    match expected(s) {
        Expected::Value(chrono) => {
            let ours = from_str.unwrap_or_else(|e| panic!("{s:?}: chrono reads it, we said {e}"));
            assert_same_value(ours, chrono, s);
            assert_eq!(serde.unwrap(), ours, "{s:?}");
        }
        Expected::Beyond => {
            let err = from_str.expect_err(s);
            assert_eq!(err.to_string(), "input is out of range", "{s:?}");
            assert!(err.is_beyond_range(), "{s:?}");
            assert_eq!(
                serde.expect_err(s).to_string(),
                "input is out of range",
                "{s:?}"
            );
        }
        Expected::Rejected(message) => {
            let err = match from_str {
                Ok(v) => panic!("{s:?}: chrono says {message:?}, we read {v:?}"),
                Err(e) => e,
            };
            assert_eq!(err.to_string(), message, "{s:?}");
            assert!(!err.is_beyond_range(), "{s:?}");
            let chrono = serde_json::from_value::<Chrono>(Value::String(s.to_owned()));
            assert_eq!(
                serde.expect_err(s).to_string(),
                chrono.expect_err(s).to_string(),
                "{s:?}"
            );
        }
    }
}

/// A civil date-time in jiff's range as fields: (year, month, day, hour, minute, second, nano).
fn civil_fields() -> impl Strategy<Value = (i32, u32, u32, u32, u32, u32, u32)> {
    let year = prop_oneof![
        4 => -9999i32..=9999,
        3 => 1900i32..=2100,
        1 => -2i32..=2,
        1 => prop_oneof![Just(-9999i32), Just(9999), Just(-1000), Just(999), Just(1000)],
    ];
    let nano = prop_oneof![
        Just(0u32),
        (0u32..1_000).prop_map(|ms| ms * 1_000_000),
        (0u32..1_000_000).prop_map(|us| us * 1_000),
        0u32..1_000_000_000,
        Just(999_999_999u32),
    ];
    (
        year,
        1u32..=12,
        1u32..=31,
        0u32..24,
        0u32..60,
        0u32..60,
        nano,
    )
}

fn chrono_from_fields(f: (i32, u32, u32, u32, u32, u32, u32)) -> Option<Chrono> {
    NaiveDate::from_ymd_opt(f.0, f.1, f.2)?.and_hms_nano_opt(f.3, f.4, f.5, f.6)
}

/// One field of the grammar: mostly the valid form, sometimes a near miss.
fn digits(valid: std::ops::RangeInclusive<u32>, width: usize) -> BoxedStrategy<String> {
    let top = *valid.end();
    prop_oneof![
        40 => valid.clone().prop_map(move |v| format!("{v:0width$}")),
        // Unpadded, over-padded, and just past the range.
        4 => valid.clone().prop_map(|v| v.to_string()),
        1 => valid.prop_map(move |v| format!("{v:0w$}", w = width + 1)),
        1 => (top..=top + 2).prop_map(move |v| format!("{v:0width$}")),
        1 => Just(String::new()),
        1 => "[0-9a-z:.+ -]{1,3}",
    ]
    .boxed()
}

fn year() -> BoxedStrategy<String> {
    prop_oneof![
        24 => (0u32..=9999).prop_map(|y| format!("{y:04}")),
        2 => (1990u32..=2030).prop_map(|y| y.to_string()),
        // Signed, any number of digits: inside jiff's range, past it, past chrono's, past i64.
        3 => ("[+-]", 0u32..=10_001).prop_map(|(s, y)| format!("{s}{y:04}")),
        2 => ("[+-]", 9_990u32..=10_010).prop_map(|(s, y)| format!("{s}{y}")),
        1 => ("[+-]", 262_140u32..=262_146).prop_map(|(s, y)| format!("{s}{y}")),
        1 => ("[+-]", "[0-9]{1,22}").prop_map(|(s, y)| format!("{s}{y}")),
        1 => "[0-9]{1,6}",
        1 => Just(String::new()),
        1 => "[+-]{0,2}[0-9 ]{0,5}",
    ]
    .boxed()
}

/// Whitespace chrono may or may not skip, ASCII and not.
fn space() -> BoxedStrategy<String> {
    prop_oneof![
        12 => Just(String::new()),
        2 => Just(" ".to_owned()),
        1 => Just("  ".to_owned()),
        1 => Just("\t".to_owned()),
        1 => Just("\n".to_owned()),
        1 => Just("\u{a0}".to_owned()),
        1 => Just("\u{3000}".to_owned()),
        1 => Just("\u{2003} ".to_owned()),
    ]
    .boxed()
}

fn literal(valid: &'static str, wrong: &'static [&'static str]) -> BoxedStrategy<String> {
    prop_oneof![
        60 => Just(valid.to_owned()),
        2 => proptest::sample::select(wrong).prop_map(str::to_owned),
    ]
    .boxed()
}

fn fraction() -> BoxedStrategy<String> {
    prop_oneof![
        6 => Just(String::new()),
        8 => "\\.[0-9]{1,9}",
        2 => "\\.[0-9]{3}",
        2 => "\\.[0-9]{6}000",
        2 => "\\.[0-9]{10,24}",
        1 => Just(".".to_owned()),
        1 => "[.,][0-9a-z .-]{0,4}",
    ]
    .boxed()
}

fn suffix() -> BoxedStrategy<String> {
    prop_oneof![
        40 => Just(String::new()),
        1 => Just("Z".to_owned()),
        1 => Just("z".to_owned()),
        1 => Just("+00:00".to_owned()),
        1 => Just("-05:30".to_owned()),
        1 => Just(" UTC".to_owned()),
        1 => Just("\u{e9}".to_owned()),
        1 => "[ -~]{1,3}",
    ]
    .boxed()
}

/// A string close to `YYYY-MM-DDTHH:MM:SS[.fraction]`: each part is usually right and
/// sometimes slightly wrong, with optional whitespace at every joint.
fn near_valid() -> BoxedStrategy<String> {
    let date = (
        space(),
        year(),
        space(),
        literal("-", &["", "/", "\u{2212}", "--", ":"]),
        space(),
        digits(1..=12, 2),
        space(),
        literal("-", &["", "/", "\u{2212}", "--", ":"]),
        space(),
        digits(1..=31, 2),
    );
    let time = (
        space(),
        literal("T", &["t", " ", "", "_", "TT", "\u{442}"]),
        space(),
        digits(0..=23, 2),
        space(),
        literal(":", &["", ".", "::", "-"]),
        space(),
        digits(0..=59, 2),
        space(),
        literal(":", &["", ".", "::", "-"]),
    );
    let rest = (
        space(),
        // 60 is a leap second, 61 is out of range.
        digits(0..=60, 2),
        space(),
        fraction(),
        space(),
        suffix(),
    );
    (date, time, rest)
        .prop_map(|(d, t, r)| {
            [
                d.0, d.1, d.2, d.3, d.4, d.5, d.6, d.7, d.8, d.9, t.0, t.1, t.2, t.3, t.4, t.5,
                t.6, t.7, t.8, t.9, r.0, r.1, r.2, r.3, r.4, r.5,
            ]
            .concat()
        })
        .boxed()
}

/// A valid string (as chrono writes it) with one character deleted, inserted or replaced.
fn mutated() -> BoxedStrategy<String> {
    let replacement = prop_oneof![
        proptest::char::range(' ', '~'),
        Just('\u{a0}'),
        Just('\u{e9}'),
        Just('\u{2212}'),
        Just('\u{662}'),
    ];
    (
        civil_fields(),
        any::<prop::sample::Index>(),
        0u8..3,
        replacement,
    )
        .prop_filter_map("a date that exists", |(f, at, op, c)| {
            let mut chars: Vec<char> = format!("{:?}", chrono_from_fields(f)?).chars().collect();
            let i = at.index(chars.len());
            match op {
                0 => {
                    chars.remove(i);
                }
                1 => chars.insert(i, c),
                _ => chars[i] = c,
            }
            Some(chars.into_iter().collect())
        })
        .boxed()
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 8192, failure_persistence: None, ..ProptestConfig::default() })]

    /// AC 1: every civil date-time in jiff's range is written as chrono writes it (0, 3, 6 or
    /// 9 fraction digits, no offset), and chrono's output reads back as the same value.
    #[test]
    fn serialize_matches_chrono(fields in civil_fields()) {
        let chrono = chrono_from_fields(fields);
        let ours = NaiveDateTime::from_ymd_hms_nano(
            fields.0, fields.1, fields.2, fields.3, fields.4, fields.5, fields.6,
        );
        prop_assert_eq!(ours.is_some(), chrono.is_some(), "{:?}", fields);
        let (Some(ours), Some(chrono)) = (ours, chrono) else {
            return Ok(());
        };
        let written = format!("{chrono:?}");
        assert_same_value(ours, chrono, &written);
        prop_assert_eq!(written.parse::<NaiveDateTime>().unwrap(), ours);
        let json = serde_json::to_string(&chrono).unwrap();
        prop_assert_eq!(serde_json::from_str::<NaiveDateTime>(&json).unwrap(), ours);
        // What we write, chrono reads as the same value too.
        let ours_json = serde_json::to_string(&ours).unwrap();
        prop_assert_eq!(serde_json::from_str::<Chrono>(&ours_json).unwrap(), chrono);
        // Ordering is the calendar's.
        prop_assert_eq!(ours.cmp(&NaiveDateTime::MAX), chrono.cmp(&civil_max()));
    }

    /// AC 2: `FromStr` and the serde visitor accept what chrono accepts, with the same value,
    /// and reject what it rejects, with the same message.
    #[test]
    fn parse_matches_chrono_accept_and_reject(s in prop_oneof![6 => near_valid(), 3 => mutated(), 1 => any::<String>()]) {
        assert_parses_as_chrono(&s);
    }

    /// The conversions to and from `Timestamp` are chrono's `naive_utc` and `and_utc`.
    #[test]
    fn timestamp_conversions_match_chrono(fields in civil_fields()) {
        let (Some(ours), Some(chrono)) = (
            NaiveDateTime::from_ymd_hms_nano(
                fields.0, fields.1, fields.2, fields.3, fields.4, fields.5, fields.6,
            ),
            chrono_from_fields(fields),
        ) else {
            return Ok(());
        };
        let utc: DateTime<Utc> = chrono.and_utc();
        let rfc3339 = utc.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true);
        match Timestamp::try_from(ours) {
            Ok(t) => {
                prop_assert_eq!(t.to_rfc3339_auto(), rfc3339);
                prop_assert_eq!(NaiveDateTime::from(t), ours);
            }
            // Only the part of the civil range `Timestamp` doesn't hold.
            Err(_) => prop_assert!(Timestamp::parse_rfc3339(&rfc3339).is_err(), "{}", rfc3339),
        }
    }
}

/// Both ends of the civil range, and of `Timestamp`'s, which a random draw rarely hits.
#[test]
fn range_ends_match_chrono() {
    assert_same_value(NaiveDateTime::MIN, civil_min(), "MIN");
    assert_same_value(NaiveDateTime::MAX, civil_max(), "MAX");
    for s in [
        "-9999-01-01T00:00:00",
        "9999-12-31T23:59:59.999999999",
        "9999-12-30T22:00:00.999999999",
        "9999-12-30T22:00:01",
        "-9999-01-02T01:59:59",
        "-9999-01-02T01:59:58.999999999",
        "-10000-12-31T23:59:59.999999999",
        "+10000-01-01T00:00:00",
        "9999-12-31T23:59:60",
        "-10000-12-31T23:59:60",
    ] {
        assert_parses_as_chrono(s);
    }
    assert_eq!(
        NaiveDateTime::from(Timestamp::MAX).to_string(),
        DateTime::<Utc>::from_timestamp(253_402_207_200, 999_999_999)
            .unwrap()
            .naive_utc()
            .to_string()
    );
}

/// The literals the library keeps after this crate is deleted are chrono's, row by row.
#[test]
fn frozen_goldens_are_chronos() {
    for row in goldens::ACCEPTED {
        let chrono: Chrono = row.input.parse().unwrap();
        assert!(chrono.nanosecond() < 1_000_000_000, "{}", row.input);
        assert_eq!(expected(row.input), Expected::Value(chrono));
        assert_eq!(serde_json::to_string(&chrono).unwrap(), row.json);
        assert_eq!(chrono.to_string(), row.display);
        assert_eq!(chrono.and_utc().timestamp_millis(), row.millis);
        assert_parses_as_chrono(row.input);
    }
    for row in goldens::LEAP {
        let chrono: Chrono = row.input.parse().unwrap();
        assert!(chrono.nanosecond() >= 1_000_000_000, "{}", row.input);
        assert_eq!(format!("{chrono:?}"), row.chrono);
        let Expected::Value(carried) = expected(row.input) else {
            panic!("{} is beyond the range", row.input);
        };
        assert_eq!(format!("{carried:?}"), row.loaded);
        assert_parses_as_chrono(row.input);
    }
    for row in goldens::BEYOND {
        let chrono: Chrono = row.input.parse().unwrap();
        assert_eq!(format!("{chrono:?}"), row.chrono);
        assert_eq!(expected(row.input), Expected::Beyond);
        assert_parses_as_chrono(row.input);
    }
    for row in goldens::REJECTED {
        let err = row.input.parse::<Chrono>().unwrap_err();
        assert_eq!(err.to_string(), row.error, "{:?}", row.input);
        assert_parses_as_chrono(row.input);
    }
    for (json, error) in goldens::JSON_ERRORS {
        let chrono = serde_json::from_str::<Chrono>(json).unwrap_err();
        assert_eq!(chrono.to_string(), *error, "{json:?}");
        let ours = serde_json::from_str::<NaiveDateTime>(json).unwrap_err();
        assert_eq!(ours.to_string(), *error, "{json:?}");
    }
}

/// The fork's literals (`index.rs` and `manifest.rs` tests), through chrono and through us.
#[test]
fn fork_literals_match_chrono() {
    for s in [
        "2025-03-07T11:07:11.373105434",
        "2025-03-05T11:10:22.257901390",
        "2023-07-31T09:00:00.123",
        "2023-04-29T09:00:00.123",
        "2024-05-07T18:52:36.322426563",
        "2024-07-19T09:00:00.123",
        "2024-11-13T09:23:42.123",
        "2024-11-13T09:42:23.123",
    ] {
        assert_parses_as_chrono(s);
        let chrono: Chrono = s.parse().unwrap();
        assert_eq!(format!("{chrono:?}"), s);
        // The helper in the fork's `index.rs` tests reads nine digits with `%f`.
        let nine = format!("{}000000", s);
        if s.len() == 23 {
            let helper = Chrono::parse_from_str(&nine, "%Y-%m-%dT%H:%M:%S.%f").unwrap();
            assert_eq!(helper, chrono);
            assert_eq!(
                format!("{:?}", nine.parse::<NaiveDateTime>().unwrap()),
                format!("{helper:?}")
            );
        }
    }
}
