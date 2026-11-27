//! `chelabase_time::rfc3339`: `SystemTime` written and read as RFC 3339 through
//! `jiff::Timestamp`.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chelabase_time::rfc3339;
use proptest::prelude::*;

fn from_nanos(nanos: i128) -> SystemTime {
    let abs = Duration::new(
        (nanos.unsigned_abs() / 1_000_000_000) as u64,
        (nanos.unsigned_abs() % 1_000_000_000) as u32,
    );
    if nanos < 0 {
        UNIX_EPOCH - abs
    } else {
        UNIX_EPOCH + abs
    }
}

fn floor(nanos: i128, unit: i128) -> SystemTime {
    from_nanos(nanos.div_euclid(unit) * unit)
}

fn min_nanos() -> i128 {
    jiff::Timestamp::MIN.as_nanosecond()
}

fn max_nanos() -> i128 {
    jiff::Timestamp::MAX.as_nanosecond()
}

fn check(nanos: i128) {
    let t = from_nanos(nanos);
    let full = rfc3339::format(t).unwrap();
    assert!(full.ends_with('Z'), "{full}");
    assert_eq!(rfc3339::parse(&full).unwrap(), t, "{full}");

    let millis = rfc3339::format_millis(t).unwrap();
    let fraction = millis.rsplit_once('.').unwrap().1;
    assert!(
        fraction.len() == 4
            && fraction.ends_with('Z')
            && fraction[..3].bytes().all(|b| b.is_ascii_digit()),
        "{millis}"
    );
    assert_eq!(
        rfc3339::parse(&millis).unwrap(),
        floor(nanos, 1_000_000),
        "{millis}"
    );

    let seconds = rfc3339::format_seconds(t).unwrap();
    assert!(!seconds.contains('.'), "{seconds}");
    assert_eq!(
        rfc3339::parse(&seconds).unwrap(),
        floor(nanos, 1_000_000_000),
        "{seconds}"
    );
}

#[test]
fn round_trip() {
    for nanos in [
        0,
        1,
        -1,
        -500_000_000,
        -1_999_999_999,
        1_518_568_087_123_456_789,
        -1_518_568_087_987_654_321,
        min_nanos(),
        max_nanos(),
    ] {
        check(nanos);
    }
    assert_eq!(rfc3339::format(UNIX_EPOCH).unwrap(), "1970-01-01T00:00:00Z");
    assert_eq!(
        rfc3339::format(from_nanos(1_518_568_087_500_000_000)).unwrap(),
        "2018-02-14T00:28:07.5Z"
    );
    assert_eq!(
        rfc3339::format_millis(from_nanos(-1)).unwrap(),
        "1969-12-31T23:59:59.999Z"
    );
    assert_eq!(
        rfc3339::format_seconds(from_nanos(-1)).unwrap(),
        "1969-12-31T23:59:59Z"
    );
    assert_eq!(
        rfc3339::format_millis(UNIX_EPOCH).unwrap(),
        "1970-01-01T00:00:00.000Z"
    );

    // Past jiff's range, formatting is an error, not a panic.
    assert!(rfc3339::format(from_nanos(max_nanos() + 1)).is_err());
    assert!(rfc3339::format_millis(from_nanos(min_nanos() - 1)).is_err());
    assert!(rfc3339::format_seconds(UNIX_EPOCH + Duration::from_secs(1 << 50)).is_err());
}

proptest! {
    #[test]
    fn round_trip_across_the_range(nanos in min_nanos()..=max_nanos()) {
        check(nanos);
    }
}

#[test]
fn reads_humantime_written() {
    for (text, nanos) in [
        ("2018-02-14T00:28:07Z", 1_518_568_087_000_000_000),
        ("2018-02-14T00:28:07.123Z", 1_518_568_087_123_000_000),
        ("2018-02-14T00:28:07.123456Z", 1_518_568_087_123_456_000),
        ("2018-02-14T00:28:07.123456789Z", 1_518_568_087_123_456_789),
        ("2018-02-14T00:28:07.000000000Z", 1_518_568_087_000_000_000),
        ("1970-01-01T00:00:00Z", 0),
        ("9999-12-30T22:00:00Z", 253_402_207_200_000_000_000),
    ] {
        assert_eq!(rfc3339::parse(text).unwrap(), from_nanos(nanos), "{text}");
    }
}

#[test]
fn rejects_garbage_and_out_of_range() {
    for text in [
        "",
        "garbage",
        "2018-02-30T00:00:00Z",
        "2018-02-14T25:00:00Z",
        "2018-02-14T00:28:07",
        "10000-01-01T00:00:00Z",
        "+10000-01-01T00:00:00Z",
        "-010000-01-01T00:00:00Z",
        "9999-12-31T23:59:59Z",
    ] {
        assert!(rfc3339::parse(text).is_err(), "{text:?}");
    }
}
