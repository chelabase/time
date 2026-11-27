//! `Timestamp::to_rfc3339_nanos` and `Timestamp::strftime` against chrono 0.4.45's
//! `to_rfc3339_opts(SecondsFormat::Nanos, true)` and `format`, over the whole `Timestamp` range.

#[path = "../../tests/civil_goldens/timestamp.rs"]
mod goldens;

use chelabase_time::{Timestamp, TimestampExt};
use chrono::{DateTime, SecondsFormat, Utc};
use proptest::prelude::*;

/// `Timestamp::MIN` and `Timestamp::MAX` in whole seconds.
const MIN_SECS: i64 = -377_705_023_201;
const MAX_SECS: i64 = 253_402_207_200;

fn both(secs: i64, nanos: u32) -> (Timestamp, DateTime<Utc>) {
    let ours = Timestamp::from_unix_seconds(secs)
        .and_then(|t| t.checked_add(std::time::Duration::from_nanos(nanos.into())))
        .unwrap_or_else(|| panic!("{secs}.{nanos:09} is in range"));
    let chrono = DateTime::from_timestamp(secs, nanos).expect("chrono's range is wider");
    (ours, chrono)
}

/// Seconds across the range, with its ends and the epoch drawn more often.
fn secs() -> impl Strategy<Value = i64> {
    prop_oneof![
        6 => MIN_SECS..=MAX_SECS,
        2 => -100_000_000_000i64..=100_000_000_000,
        1 => MIN_SECS..=MIN_SECS + 86_400 * 400,
        1 => MAX_SECS - 86_400 * 400..=MAX_SECS,
        1 => -86_400i64..=86_400,
        // Years -1 to 1, where the year's sign and padding change.
        1 => -62_200_000_000i64..=-62_100_000_000,
    ]
}

/// Nanoseconds with the fraction widths chrono's auto format tells apart.
fn nanos() -> impl Strategy<Value = u32> {
    prop_oneof![
        Just(0u32),
        (0u32..1_000).prop_map(|ms| ms * 1_000_000),
        (0u32..1_000_000).prop_map(|us| us * 1_000),
        0u32..1_000_000_000,
        Just(999_999_999u32),
        Just(1u32),
    ]
}

/// A pattern made of the directives `strftime` writes and literal text between them.
fn supported_pattern() -> impl Strategy<Value = String> {
    let piece = prop_oneof![
        Just("%Y"),
        Just("%m"),
        Just("%d"),
        Just("%H"),
        Just("%M"),
        Just("%S"),
        Just("%T"),
        Just("%.3f"),
        Just("-"),
        Just(" "),
        Just(":"),
        Just("/"),
        Just("_"),
        Just("Z"),
        Just(" GMT"),
        Just("year="),
        Just("\u{e9}"),
    ];
    proptest::collection::vec(piece, 0..12).prop_map(|pieces| pieces.concat())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 4096, failure_persistence: None, ..ProptestConfig::default() })]

    /// AC 8: every pattern the fork formats with writes chrono's bytes.
    #[test]
    fn each_fork_pattern_matches_chrono(secs in secs(), nanos in nanos()) {
        let (ours, chrono) = both(secs, nanos);
        for pattern in goldens::FORK_PATTERNS {
            prop_assert_eq!(
                ours.strftime(pattern),
                Some(chrono.format(pattern).to_string()),
                "{:?} at {}", pattern, chrono
            );
        }
    }

    /// The directives behind those patterns, in any order and with any text between them.
    #[test]
    fn supported_directives_match_chrono(secs in secs(), nanos in nanos(), pattern in supported_pattern()) {
        let (ours, chrono) = both(secs, nanos);
        prop_assert_eq!(
            ours.strftime(&pattern),
            Some(chrono.format(&pattern).to_string()),
            "{:?} at {}", pattern, chrono
        );
    }

    /// AC 8: `to_rfc3339_nanos` is chrono's `SecondsFormat::Nanos` with `Z`.
    #[test]
    fn rfc3339_nanos_matches_chrono(secs in secs(), nanos in nanos()) {
        let (ours, chrono) = both(secs, nanos);
        prop_assert_eq!(ours.to_rfc3339_nanos(), chrono.to_rfc3339_opts(SecondsFormat::Nanos, true));
    }
}

/// The ends of the range, which a random draw rarely hits exactly.
#[test]
fn range_ends_match_chrono() {
    for (secs, nanos) in [
        (MIN_SECS, 0),
        (MIN_SECS, 1),
        (MAX_SECS, 999_999_999),
        (MAX_SECS, 0),
        (0, 0),
        (-1, 999_999_999),
    ] {
        let (ours, chrono) = both(secs, nanos);
        assert_eq!(
            ours.to_rfc3339_nanos(),
            chrono.to_rfc3339_opts(SecondsFormat::Nanos, true)
        );
        for pattern in goldens::FORK_PATTERNS {
            assert_eq!(
                ours.strftime(pattern),
                Some(chrono.format(pattern).to_string())
            );
        }
    }
    assert_eq!(both(MIN_SECS, 0).0, Timestamp::MIN);
    assert_eq!(both(MAX_SECS, 999_999_999).0, Timestamp::MAX);
}

/// The literals the library keeps after this crate is deleted are chrono's.
#[test]
fn frozen_goldens_are_chronos() {
    for (input, nanos) in goldens::RFC3339_NANOS {
        let chrono: DateTime<Utc> = input.parse().unwrap();
        assert_eq!(
            chrono.to_rfc3339_opts(SecondsFormat::Nanos, true),
            *nanos,
            "{input}"
        );
    }
    for row in goldens::STRFTIME {
        let chrono: DateTime<Utc> = row.instant.parse().unwrap();
        assert_eq!(
            chrono.format(row.pattern).to_string(),
            row.output,
            "{} {:?}",
            row.instant,
            row.pattern
        );
        assert!(goldens::FORK_PATTERNS.contains(&row.pattern));
    }
}
