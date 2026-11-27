//! `civil::NaiveDateTime`, `Timestamp::to_rfc3339_nanos` and `Timestamp::strftime` against
//! literals frozen from chrono 0.4.45 (`civil_goldens/`). The pageserver stores the
//! `NaiveDateTime` form in S3 (`index_part.json`, the tenant manifest, the import index), so
//! these bytes may never change.

#[path = "civil_goldens/naive.rs"]
mod naive;
#[path = "civil_goldens/timestamp.rs"]
mod timestamp;

use chelabase_time::civil::NaiveDateTime;
use chelabase_time::{Timestamp, TimestampExt};
use serde_json::{Value, json};

fn parse(s: &str) -> NaiveDateTime {
    s.parse().unwrap_or_else(|e| panic!("{s:?}: {e}"))
}

fn from_json_string(s: &str) -> Result<NaiveDateTime, serde_json::Error> {
    serde_json::from_value(Value::String(s.to_owned()))
}

/// Through `FromStr`, which reads the signed years `parse_rfc3339` refuses.
fn instant(s: &str) -> Timestamp {
    s.parse().unwrap_or_else(|e| panic!("{s}: {e}"))
}

/// AC 6 (and AC 1, 2): every lenient form chrono reads, and every string it refuses, with
/// chrono's result.
#[test]
fn chrono_leniency_literals() {
    for row in naive::ACCEPTED {
        let input = row.input;
        let value = parse(input);
        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            row.json,
            "{input:?}"
        );
        assert_eq!(value.to_string(), row.display, "{input:?}");
        assert_eq!(
            format!("\"{value:?}\""),
            row.json,
            "Debug is the serde form: {input:?}"
        );
        assert_eq!(value.timestamp_millis(), row.millis, "{input:?}");
        assert_eq!(from_json_string(input).unwrap(), value, "{input:?}");
        // What it writes reads back as itself.
        let written: NaiveDateTime = serde_json::from_str(row.json).unwrap();
        assert_eq!(written, value, "{input:?}");
    }
    for row in naive::REJECTED {
        let input = row.input;
        let err = input
            .parse::<NaiveDateTime>()
            .expect_err(&format!("{input:?} parsed"));
        assert_eq!(err.to_string(), row.error, "{input:?}");
        assert!(!err.is_beyond_range(), "{input:?}");
        let err = from_json_string(input).expect_err(&format!("{input:?} deserialized"));
        assert_eq!(err.to_string(), row.error, "{input:?}");
    }
    for (json, error) in naive::JSON_ERRORS {
        let err = serde_json::from_str::<NaiveDateTime>(json).expect_err(json);
        assert_eq!(err.to_string(), *error, "{json:?}");
    }

    // The forms the issue names, each pinned to chrono's verdict.
    let accepted = |s: &str| naive::ACCEPTED.iter().any(|r| r.input == s);
    let rejected = |s: &str| naive::REJECTED.iter().any(|r| r.input == s);
    assert!(accepted("2026-09-30T12:00:00"), "no fraction");
    assert!(
        accepted("2026-09-30T12:00:00.1234567891234567890123"),
        "more than 9 fraction digits"
    );
    assert!(accepted("-2026-09-30T12:00:00"), "a signed year");
    assert!(
        accepted(" 2026-09-30T12:00:00") && accepted("2026-09-30T 12:00:00"),
        "whitespace before a numeric field"
    );
    // chrono's `NaiveDateTime` separator is a case-sensitive `T` (unlike its RFC 3339 parsers).
    assert!(rejected("2026-09-30t12:00:00"), "a lowercase separator");
    assert!(rejected("2026-09-30 12:00:00"), "a space separator");
    assert!(rejected("2026-09-30T12:00:00x"), "trailing characters");
    assert!(rejected("2026-09-30T12:00:00Z"), "a Z suffix");
    assert!(rejected("2026-09-30T12:00:00+00:00"), "an offset suffix");
    // A 5-digit year needs a sign; chrono reads it, jiff's range can't hold it.
    assert!(rejected("12345-01-01T00:00:00"));
    assert!(
        naive::BEYOND
            .iter()
            .any(|r| r.input == "+12345-01-01T00:00:00")
    );
}

/// AC 7: the fork's literals (`index.rs` and `manifest.rs` tests) come back as the same bytes.
#[test]
fn fork_literals_round_trip() {
    for literal in [
        "2025-03-07T11:07:11.373105434",
        "2025-03-05T11:10:22.257901390",
        "2023-07-31T09:00:00.123",
        "2023-04-29T09:00:00.123",
        "2024-05-07T18:52:36.322426563",
        "2024-07-19T09:00:00.123",
        "2024-11-13T09:23:42.123",
        "2024-11-13T09:42:23.123",
    ] {
        let value = parse(literal);
        assert_eq!(format!("{value:?}"), literal);
        let json = format!("\"{literal}\"");
        let read: NaiveDateTime = serde_json::from_str(&json).unwrap();
        assert_eq!(read, value);
        assert_eq!(serde_json::to_string(&read).unwrap(), json);
        assert_eq!(serde_json::to_vec(&read).unwrap(), json.as_bytes());
    }
    // The fork's test helper reads `%Y-%m-%dT%H:%M:%S.%f` with nine digits; that is the same
    // value as the three the JSON holds.
    for (nine, three) in [
        ("2023-07-31T09:00:00.123000000", "2023-07-31T09:00:00.123"),
        ("2023-04-29T09:00:00.123000000", "2023-04-29T09:00:00.123"),
        ("2024-07-19T09:00:00.123000000", "2024-07-19T09:00:00.123"),
        ("2024-11-13T09:23:42.123000000", "2024-11-13T09:23:42.123"),
        ("2024-11-13T09:42:23.123000000", "2024-11-13T09:42:23.123"),
    ] {
        assert_eq!(parse(nine), parse(three));
        assert_eq!(
            serde_json::to_string(&parse(nine)).unwrap(),
            format!("\"{three}\"")
        );
    }

    // A document shaped like the tenant manifest and index_part: the bytes survive a load and
    // a store, inside `Option`, tuples and nested objects.
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct Stored {
        archived_at: NaiveDateTime,
        deleted_at: Option<NaiveDateTime>,
        marked_invisible_at: Option<NaiveDateTime>,
        original_ancestor: (String, String, NaiveDateTime),
    }
    let document = json!({
        "archived_at": "2025-03-07T11:07:11.373105434",
        "deleted_at": "2023-07-31T09:00:00.123",
        "marked_invisible_at": null,
        "original_ancestor": [
            "e2bfd8c633d713d279e6fcd2bcc15b6d",
            "0/15A7618",
            "2024-05-07T18:52:36.322426563"
        ],
    });
    let stored: Stored = serde_json::from_value(document.clone()).unwrap();
    assert_eq!(serde_json::to_value(&stored).unwrap(), document);
    assert_eq!(
        serde_json::to_string(&stored).unwrap(),
        r#"{"archived_at":"2025-03-07T11:07:11.373105434","deleted_at":"2023-07-31T09:00:00.123","marked_invisible_at":null,"original_ancestor":["e2bfd8c633d713d279e6fcd2bcc15b6d","0/15A7618","2024-05-07T18:52:36.322426563"]}"#
    );
}

/// AC 3: what chrono reads and jiff's civil range can't hold is an error, never a panic.
#[test]
fn beyond_range_is_an_error_not_a_panic() {
    for row in naive::BEYOND {
        let input = row.input;
        let err = input
            .parse::<NaiveDateTime>()
            .expect_err(&format!("{input:?} parsed"));
        assert_eq!(err.to_string(), "input is out of range", "{input:?}");
        assert!(err.is_beyond_range(), "{input:?}");
        let err = from_json_string(input).expect_err(&format!("{input:?} deserialized"));
        assert_eq!(err.to_string(), "input is out of range", "{input:?}");
        // So is the form chrono wrote for it.
        let err = row.chrono.parse::<NaiveDateTime>().unwrap_err();
        assert!(err.is_beyond_range(), "{:?}", row.chrono);
    }
    // chrono's own range and calendar errors are not "beyond range": chrono refuses them too.
    for input in [
        "+262143-01-01T00:00:00",
        "+20000-02-30T12:00:00",
        "2026-02-29T12:00:00",
        "+9223372036854775808-01-01T00:00:00",
    ] {
        let err = input.parse::<NaiveDateTime>().unwrap_err();
        assert_eq!(err.to_string(), "input is out of range", "{input:?}");
        assert!(!err.is_beyond_range(), "{input:?}");
    }
    // The constructor returns `None` past the range and for a date or time that doesn't exist.
    assert!(NaiveDateTime::from_ymd_hms_nano(10_000, 1, 1, 0, 0, 0, 0).is_none());
    assert!(NaiveDateTime::from_ymd_hms_nano(-10_000, 12, 31, 23, 59, 59, 0).is_none());
    assert!(NaiveDateTime::from_ymd_hms_nano(i32::MAX, 1, 1, 0, 0, 0, 0).is_none());
    assert!(NaiveDateTime::from_ymd_hms_nano(2026, 2, 29, 0, 0, 0, 0).is_none());
    assert!(NaiveDateTime::from_ymd_hms_nano(2026, 1, 1, 24, 0, 0, 0).is_none());
    assert!(NaiveDateTime::from_ymd_hms_nano(2026, 1, 1, 0, 0, 60, 0).is_none());
    assert!(NaiveDateTime::from_ymd_hms_nano(2026, 1, 1, 0, 0, 0, 1_000_000_000).is_none());
    assert!(NaiveDateTime::from_ymd_hms_nano(2026, u32::MAX, 1, 0, 0, 0, 0).is_none());
}

/// AC 5: a second of `60` loads as the next minute's second 0, the rule `Timestamp` uses.
#[test]
fn leap_second_loads_without_panic() {
    for row in naive::LEAP {
        let input = row.input;
        let value = parse(input);
        assert_eq!(format!("{value:?}"), row.loaded, "{input:?}");
        assert_eq!(from_json_string(input).unwrap(), value, "{input:?}");
        assert_eq!(value, parse(row.loaded), "{input:?}");
        // chrono wrote `:60` back; reading that is the same value again.
        assert_eq!(parse(row.chrono), value, "{input:?}");
    }
    // The issue's example, and the same instant as `Timestamp` reads it.
    let leap = parse("2016-12-31T23:59:60.5");
    assert_eq!(format!("{leap:?}"), "2017-01-01T00:00:00.500");
    assert_eq!(
        Timestamp::try_from(leap).unwrap(),
        instant("2016-12-31T23:59:60.5Z")
    );
    // Past the range's last second there is nothing to carry into.
    let err = "9999-12-31T23:59:60".parse::<NaiveDateTime>().unwrap_err();
    assert!(err.is_beyond_range());
}

/// AC 9: `timestamp_millis` is chrono's `and_utc().timestamp_millis()`, floor before 1970.
#[test]
fn timestamp_millis_floor_before_epoch() {
    for (input, millis) in [
        ("1970-01-01T00:00:00", 0),
        ("1970-01-01T00:00:00.000999999", 0),
        ("1970-01-01T00:00:00.001", 1),
        ("1969-12-31T23:59:59.999999999", -1),
        ("1969-12-31T23:59:59.999", -1),
        ("1969-12-31T23:59:59.9989", -2),
        ("1969-12-31T23:59:59.000000001", -1000),
        ("1969-12-31T23:59:59", -1000),
        ("1969-12-31T23:59:58.999999999", -1001),
        ("2025-03-07T11:07:11.373105434", 1_741_345_631_373),
        ("0000-01-01T00:00:00", -62_167_219_200_000),
        ("-9999-01-01T00:00:00", -377_705_116_800_000),
        ("-9999-01-01T00:00:00.000000001", -377_705_116_800_000),
        ("9999-12-31T23:59:59.999999999", 253_402_300_799_999),
    ] {
        assert_eq!(parse(input).timestamp_millis(), millis, "{input}");
    }
    // Every frozen row agrees (they hold chrono's value).
    for row in naive::ACCEPTED {
        assert_eq!(
            parse(row.input).timestamp_millis(),
            row.millis,
            "{}",
            row.input
        );
    }
}

/// AC 4: the range is jiff's whole civil range, and the conversion to `Timestamp` is fallible.
#[test]
fn civil_range_and_fallible_timestamp_conversion() {
    let (min, max) = (NaiveDateTime::MIN, NaiveDateTime::MAX);
    assert_eq!(format!("{min:?}"), "-9999-01-01T00:00:00");
    assert_eq!(format!("{max:?}"), "9999-12-31T23:59:59.999999999");
    assert_eq!(parse("-9999-01-01T00:00:00"), min);
    assert_eq!(parse("9999-12-31T23:59:59.999999999"), max);
    assert_eq!(min.to_string(), "-9999-01-01 00:00:00");
    assert_eq!(max.to_string(), "9999-12-31 23:59:59.999999999");
    assert_eq!(
        serde_json::to_string(&max).unwrap(),
        "\"9999-12-31T23:59:59.999999999\""
    );
    assert_eq!(min.timestamp_millis(), -377_705_116_800_000);
    assert_eq!(max.timestamp_millis(), 253_402_300_799_999);
    assert!(min < max);

    // A civil time `Timestamp` can't hold: an error, and `timestamp_millis` is still chrono's.
    let late = parse("9999-12-31T23:00:00");
    let err = Timestamp::try_from(late).unwrap_err();
    assert_eq!(
        err.to_string(),
        "the date and time is outside the range of Timestamp"
    );
    assert_eq!(late.timestamp_millis(), 253_402_297_200_000);
    assert!(Timestamp::try_from(max).is_err());
    assert!(Timestamp::try_from(min).is_err());

    // Exactly at `Timestamp`'s ends, fraction included, and one nanosecond or second past them.
    assert_eq!(
        Timestamp::try_from(parse("9999-12-30T22:00:00.999999999")).unwrap(),
        Timestamp::MAX
    );
    assert!(Timestamp::try_from(parse("9999-12-30T22:00:01")).is_err());
    assert_eq!(
        Timestamp::try_from(parse("-9999-01-02T01:59:59")).unwrap(),
        Timestamp::MIN
    );
    assert!(Timestamp::try_from(parse("-9999-01-02T01:59:58.999999999")).is_err());

    // Every `Timestamp` is a civil date-time in UTC, and comes back.
    for t in [
        Timestamp::MIN,
        Timestamp::MAX,
        Timestamp::UNIX_EPOCH,
        instant("2025-03-07T11:07:11.373105434Z"),
        instant("1969-12-31T23:59:59.999999999Z"),
    ] {
        let civil = NaiveDateTime::from(t);
        assert_eq!(format!("{civil:?}Z"), t.to_rfc3339_auto());
        assert_eq!(Timestamp::try_from(civil).unwrap(), t);
        assert_eq!(civil.timestamp_millis(), t.timestamp_millis());
    }

    // The constructor, and the jiff value underneath.
    let built = NaiveDateTime::from_ymd_hms_nano(2025, 3, 7, 11, 7, 11, 373_105_434).unwrap();
    assert_eq!(built, parse("2025-03-07T11:07:11.373105434"));
    assert_eq!(NaiveDateTime::from_jiff(built.to_jiff()), built);
    assert_eq!(
        NaiveDateTime::from_ymd_hms_nano(9999, 12, 31, 23, 59, 59, 999_999_999).unwrap(),
        max
    );
    assert_eq!(
        NaiveDateTime::from_ymd_hms_nano(-9999, 1, 1, 0, 0, 0, 0).unwrap(),
        min
    );

    // `now_utc` is the current UTC time as a civil date-time.
    let before = Timestamp::now();
    let now = Timestamp::try_from(NaiveDateTime::now_utc()).unwrap();
    assert!(before <= now && now <= Timestamp::now());
}

/// AC 8: chrono's `to_rfc3339_opts(SecondsFormat::Nanos, true)`.
#[test]
fn rfc3339_nanos_goldens() {
    for (input, nanos) in timestamp::RFC3339_NANOS {
        assert_eq!(instant(input).to_rfc3339_nanos(), *nanos, "{input}");
        // Nine digits read back as the same instant.
        assert_eq!(instant(nanos), instant(input), "{input}");
    }
}

/// AC 8: each of the fork's strftime patterns writes what chrono's `format` wrote.
#[test]
fn strftime_goldens() {
    for row in timestamp::STRFTIME {
        assert_eq!(
            instant(row.instant).strftime(row.pattern).as_deref(),
            Some(row.output),
            "{} {:?}",
            row.instant,
            row.pattern
        );
    }
    // Every fork pattern has rows.
    for pattern in timestamp::FORK_PATTERNS {
        assert!(
            timestamp::STRFTIME.iter().any(|r| r.pattern == *pattern),
            "{pattern:?}"
        );
    }
}

/// `strftime` writes only the directives the fork's patterns use; anything else is `None`,
/// not jiff's (different) rendering of it and not a panic.
#[test]
fn strftime_refuses_other_directives() {
    let t = instant("2026-09-30T12:00:00.123456789Z");
    for pattern in [
        "%f", "%.f", "%.6f", "%.9f", "%3f", "%y", "%e", "%j", "%z", "%:z", "%Z", "%s", "%+", "%c",
        "%D", "%F", "%R", "%I", "%p", "%b", "%a", "%n", "%t", "%%", "%-d", "%_d", "%0d", "%",
        "%Y-%", "%.", "%.3", "%.3x", "%\u{e9}", "%Q",
    ] {
        assert_eq!(t.strftime(pattern), None, "{pattern:?}");
    }
    // Text without a directive is written as it is.
    assert_eq!(t.strftime("").as_deref(), Some(""));
    assert_eq!(
        t.strftime("\u{e9}t\u{e9} %Y \u{2212}").as_deref(),
        Some("\u{e9}t\u{e9} 2026 \u{2212}")
    );
}
