//! `chelabase_time::human` writes and reads what humantime 2.2.0 and humantime-serde 1.1.1 did:
//! the literal tables in `tests/humantime_tables.rs`, taken from those crates (the oracle's
//! `frozen_tables_are_humantimes` holds humantime to the same rows), plus the cases humantime
//! itself can't show: where it panics, a clap argument, and the stored tenant-config strings.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chelabase_time::human::{self, DurationError, TimestampError};
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

#[path = "humantime_tables.rs"]
mod tables;

/// One past the last second the helpers format and parse: `10000-01-01T00:00:00Z`.
const YEAR_10000: u64 = 253_402_300_800;

/// A `Duration`, an `Option<Duration>` and a defaulted `Option<Duration>`, all through the one
/// `with` path.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Conf {
    #[serde(with = "chelabase_time::human::serde")]
    d: Duration,
    #[serde(with = "chelabase_time::human::serde")]
    o: Option<Duration>,
    #[serde(default, with = "chelabase_time::human::serde")]
    od: Option<Duration>,
}

/// The shape of the proxy's `ConcurrencyLockOptions`.
#[derive(Deserialize)]
struct Epoch {
    #[serde(deserialize_with = "chelabase_time::human::serde::deserialize")]
    epoch: Duration,
}

/// The shape of the fork's `LsnLease` (`SystemTimeAsRfc3339Millis`).
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Lease {
    #[serde(with = "chelabase_time::human::rfc3339_millis")]
    valid_until: SystemTime,
}

type Fields = (Duration, Option<Duration>, Option<Duration>);

fn std_duration((secs, nanos): tables::D) -> Duration {
    Duration::new(secs, nanos)
}

fn fields<E: ToString>(r: Result<Conf, E>) -> Result<Fields, String> {
    r.map(|c| (c.d, c.o, c.od)).map_err(|e| e.to_string())
}

fn expected_fields(
    row: &Result<(tables::D, Option<tables::D>, Option<tables::D>), &str>,
) -> Result<Fields, String> {
    match row {
        Ok((d, o, od)) => Ok((std_duration(*d), o.map(std_duration), od.map(std_duration))),
        Err(e) => Err((*e).to_owned()),
    }
}

fn text<T: ToString, E: ToString>(r: Result<T, E>) -> Result<String, String> {
    r.map(|v| v.to_string()).map_err(|e| e.to_string())
}

fn expected_text(row: &Result<&str, &str>) -> Result<String, String> {
    row.map(str::to_owned).map_err(str::to_owned)
}

fn since_epoch(r: Result<SystemTime, TimestampError>) -> Result<(u64, u32), (String, String)> {
    match r {
        Ok(t) => {
            let d = t.duration_since(UNIX_EPOCH).expect("not before the epoch");
            Ok((d.as_secs(), d.subsec_nanos()))
        }
        Err(e) => Err((format!("{e:?}"), e.to_string())),
    }
}

#[test]
fn frozen_oracle_tables() {
    // Durations.
    for &(secs, nanos, text) in tables::FORMAT_DURATION {
        let d = Duration::new(secs, nanos);
        assert_eq!(human::format_duration(d).to_string(), text);
        assert_eq!(human::Duration::from(d).to_string(), text);
        assert_eq!(human::parse_duration(text), Ok(d), "{text}");
    }
    for &(text, secs, nanos) in tables::PARSE_DURATION_OK {
        let d = Duration::new(secs, nanos);
        assert_eq!(human::parse_duration(text), Ok(d), "{text:?}");
        assert_eq!(
            text.parse::<human::Duration>().map(Duration::from),
            Ok(d),
            "{text:?}"
        );
    }
    for &(text, debug) in tables::PARSE_DURATION_ERR {
        let e = human::parse_duration(text).unwrap_err();
        assert_eq!(format!("{e:?}"), debug, "{text:?}");
    }
    for &(text, secs, nanos, _) in tables::PARSE_DURATION_WIDER {
        assert_eq!(
            human::parse_duration(text),
            Ok(Duration::new(secs, nanos)),
            "{text:?}"
        );
    }
    for &(text, debug, _) in tables::PARSE_DURATION_OTHER_ERR {
        let e = human::parse_duration(text).unwrap_err();
        assert_eq!(format!("{e:?}"), debug, "{text:?}");
    }
    for &(text, secs, nanos, _) in tables::PARSE_DURATION_PENDING_SECOND_OK {
        assert_eq!(
            human::parse_duration(text),
            Ok(Duration::new(secs, nanos)),
            "{text:?}"
        );
    }
    for &(text, debug, _) in tables::PARSE_DURATION_PENDING_SECOND_ERR {
        let e = human::parse_duration(text).unwrap_err();
        assert_eq!(format!("{e:?}"), debug, "{text:?}");
    }
    for &text in tables::PARSE_DURATION_HUMANTIME_PANICS {
        assert_eq!(
            human::parse_duration(text),
            Err(DurationError::NumberOverflow),
            "{text:?}"
        );
    }
    for &(text, display, _) in tables::DURATION_ERROR_DISPLAY {
        assert_eq!(
            human::parse_duration(text).unwrap_err().to_string(),
            display,
            "{text:?}"
        );
    }

    // The serde module.
    for (d, o, od, json, toml) in tables::SERDE_SERIALIZE {
        let conf = Conf {
            d: std_duration(*d),
            o: o.map(std_duration),
            od: od.map(std_duration),
        };
        let json = expected_text(json);
        assert_eq!(text(serde_json::to_string(&conf)), json);
        assert_eq!(serde_json::from_str::<Conf>(&json.unwrap()).unwrap(), conf);
        let toml = expected_text(toml);
        assert_eq!(text(toml::to_string(&conf)), toml);
        // TOML has no null: a `None` writes no key, so `o`, which has no `default`, then
        // fails to read back (humantime-serde did the same).
        let back = toml::from_str::<Conf>(&toml.unwrap());
        match conf.o {
            Some(_) => assert_eq!(back.unwrap(), conf),
            None => assert!(back.unwrap_err().to_string().contains("missing field `o`")),
        }
    }
    for (doc, row) in tables::SERDE_DESERIALIZE_JSON {
        assert_eq!(
            fields(serde_json::from_str(doc)),
            expected_fields(row),
            "{doc}"
        );
    }
    for (doc, row) in tables::SERDE_DESERIALIZE_TOML {
        assert_eq!(fields(toml::from_str(doc)), expected_fields(row), "{doc}");
    }
    for (doc, row) in tables::SERDE_DESERIALIZE_WITH_JSON {
        assert_eq!(
            serde_json::from_str::<Epoch>(doc)
                .map(|e| e.epoch)
                .map_err(|e| e.to_string()),
            row.map(std_duration).map_err(str::to_owned),
            "{doc}"
        );
    }

    // SystemTime RFC 3339.
    for &(secs, nanos, smart, seconds, millis) in tables::RFC3339_FORMAT {
        let t = UNIX_EPOCH + Duration::new(secs, nanos);
        assert_eq!(human::format_rfc3339(t).unwrap().to_string(), smart);
        assert_eq!(
            human::format_rfc3339_seconds(t).unwrap().to_string(),
            seconds
        );
        assert_eq!(human::format_rfc3339_millis(t).unwrap().to_string(), millis);
        assert_eq!(human::parse_rfc3339(smart), Ok(t));
        assert_eq!(
            serde_json::to_string(&Lease { valid_until: t }).unwrap(),
            format!("{{\"valid_until\":\"{millis}\"}}")
        );
    }
    for (doc, row) in tables::RFC3339_MILLIS_DESERIALIZE_JSON {
        assert_eq!(
            serde_json::from_str::<Lease>(doc)
                .map(|l| l.valid_until)
                .map_err(|e| e.to_string()),
            row.map(|d| UNIX_EPOCH + std_duration(d))
                .map_err(str::to_owned),
            "{doc}"
        );
    }
    for &(text, secs, nanos) in tables::RFC3339_PARSE_OK {
        assert_eq!(
            since_epoch(human::parse_rfc3339(text)),
            Ok((secs, nanos)),
            "{text:?}"
        );
    }
    for &(text, debug, display) in tables::RFC3339_PARSE_ERR {
        assert_eq!(
            since_epoch(human::parse_rfc3339(text)),
            Err((debug.to_owned(), display.to_owned())),
            "{text:?}"
        );
    }
}

/// The range is humantime's, not `Timestamp`'s: 1970 to the end of 9999, with second `60`.
#[test]
fn rfc3339_range_matches_humantime() {
    let at = |secs: u64, nanos: u32| Ok(UNIX_EPOCH + Duration::new(secs, nanos));
    assert_eq!(human::parse_rfc3339("1970-01-01T00:00:00Z"), at(0, 0));
    // Past `Timestamp::MAX` (`9999-12-30T22:00:00.999999999Z`), which refuses these.
    assert_eq!(
        human::parse_rfc3339("9999-12-31T23:00:00Z"),
        at(253_402_297_200, 0)
    );
    assert_eq!(
        human::parse_rfc3339("9999-12-31T23:59:59Z"),
        at(YEAR_10000 - 1, 0)
    );
    assert_eq!(
        human::parse_rfc3339("9999-12-31T23:59:59.999999999Z"),
        at(YEAR_10000 - 1, 999_999_999)
    );
    {
        use chelabase_time::{Timestamp, TimestampExt};
        assert!(Timestamp::parse_rfc3339("9999-12-31T23:00:00Z").is_err());
    }
    // Second 60 is read as second 59 of the same minute, at any time of day.
    assert_eq!(
        human::parse_rfc3339("9999-12-31T23:59:60Z"),
        at(YEAR_10000 - 1, 0)
    );
    assert_eq!(
        human::parse_rfc3339("2016-12-31T23:59:60Z"),
        at(1_483_228_799, 0)
    );
    assert_eq!(
        human::parse_rfc3339("2018-02-14T12:00:60.5Z"),
        at(1_518_609_659, 500_000_000)
    );

    // Before 1970, past second 60, and five-digit years.
    for (text, error) in [
        ("1969-12-31T23:59:59Z", TimestampError::OutOfRange),
        ("1969-12-31T23:59:60Z", TimestampError::OutOfRange),
        ("0001-01-01T00:00:00Z", TimestampError::OutOfRange),
        ("9999-12-31T23:59:61Z", TimestampError::OutOfRange),
        ("9999-12-31T24:00:00Z", TimestampError::OutOfRange),
        ("10000-01-01T00:00:00Z", TimestampError::InvalidFormat),
        // Only `Z`: no offsets, no lower case, no space for the `T`.
        ("2018-02-14T00:28:07+01:00", TimestampError::InvalidFormat),
        ("2018-02-14T00:28:07+00:00", TimestampError::InvalidFormat),
        ("2018-02-14T00:28:07z", TimestampError::InvalidFormat),
        ("2018-02-14 00:28:07Z", TimestampError::InvalidFormat),
        ("2018-02-14T00:28:07", TimestampError::InvalidFormat),
    ] {
        assert_eq!(human::parse_rfc3339(text), Err(error), "{text}");
    }

    // The last time parses, formats and parses back.
    let last = UNIX_EPOCH + Duration::new(YEAR_10000 - 1, 999_999_999);
    assert_eq!(
        human::format_rfc3339(last).unwrap().to_string(),
        "9999-12-31T23:59:59.999999999Z"
    );
    assert_eq!(
        human::format_rfc3339_millis(last).unwrap().to_string(),
        "9999-12-31T23:59:59.999Z"
    );
    assert_eq!(
        human::format_rfc3339_seconds(last).unwrap().to_string(),
        "9999-12-31T23:59:59Z"
    );
}

/// humantime panics before the epoch and its `Display` fails from year 10000 on (a panic in
/// `to_string`). Here all three formatters return an error, and so does serialization.
#[test]
fn format_out_of_domain_is_an_error_not_a_panic() {
    let outside = [
        UNIX_EPOCH - Duration::new(0, 1),
        UNIX_EPOCH - Duration::from_secs(1),
        UNIX_EPOCH - Duration::from_secs(YEAR_10000),
        UNIX_EPOCH + Duration::from_secs(YEAR_10000),
        UNIX_EPOCH + Duration::new(YEAR_10000, 999_999_999),
        UNIX_EPOCH + Duration::from_secs(1 << 40),
    ];
    for t in outside {
        for formatted in [
            human::format_rfc3339(t),
            human::format_rfc3339_seconds(t),
            human::format_rfc3339_millis(t),
        ] {
            assert_eq!(
                formatted.map(|f| f.to_string()),
                Err(TimestampError::OutOfRange),
                "{t:?}"
            );
        }
        let error = serde_json::to_string(&Lease { valid_until: t }).unwrap_err();
        assert_eq!(
            error.to_string(),
            "numeric component is out of range",
            "{t:?}"
        );
        let error = toml::to_string(&Lease { valid_until: t }).unwrap_err();
        assert_eq!(
            error.to_string(),
            "numeric component is out of range",
            "{t:?}"
        );
    }

    // Inside the domain it serializes and reads back (bad strings: `frozen_oracle_tables`).
    let t = UNIX_EPOCH + Duration::new(1_790_769_600, 500_999_999);
    let json = serde_json::to_string(&Lease { valid_until: t }).unwrap();
    assert_eq!(json, r#"{"valid_until":"2026-09-30T12:00:00.500Z"}"#);
    assert_eq!(
        serde_json::from_str::<Lease>(&json).unwrap().valid_until,
        UNIX_EPOCH + Duration::new(1_790_769_600, 500_000_000)
    );
}

#[test]
fn clap_duration_value_parser() {
    let command = || {
        clap::Command::new("bin").arg(
            clap::Arg::new("x")
                .long("x")
                .value_parser(clap::value_parser!(human::Duration)),
        )
    };
    for (arg, secs) in [
        ("1h30m", 5400),
        ("1h 30m", 5400),
        ("7days", 604_800),
        ("0s", 0),
    ] {
        let matches = command().try_get_matches_from(["bin", "--x", arg]).unwrap();
        let x: human::Duration = *matches.get_one("x").unwrap();
        assert_eq!(Duration::from(x), Duration::from_secs(secs), "{arg}");
        // `Deref` gives the std methods, as `humantime::Duration` did.
        assert_eq!(x.as_secs(), secs, "{arg}");
    }

    for arg in ["5parsecs", "1h 30", ""] {
        let error = command()
            .try_get_matches_from(["bin", "--x", arg])
            .unwrap_err();
        assert_eq!(
            error.kind(),
            clap::error::ErrorKind::ValueValidation,
            "{arg:?}"
        );
    }
    let error = command()
        .try_get_matches_from(["bin", "--x", "5parsecs"])
        .unwrap_err();
    let source = std::error::Error::source(&error).expect("the parse error");
    assert_eq!(
        source.downcast_ref::<DurationError>(),
        Some(&DurationError::UnknownUnit {
            start: 1,
            end: 8,
            unit: "parsecs".to_owned(),
            value: 5,
        })
    );
}

/// The duration strings the storage controller keeps in `tenant_shards.config` (and chelabase
/// reads back): each parses, and its duration is written as the same string.
#[test]
fn stored_config_literals() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TenantConfig {
        #[serde(with = "chelabase_time::human::serde")]
        pitr_interval: Duration,
        #[serde(default, with = "chelabase_time::human::serde")]
        gc_period: Option<Duration>,
    }

    for (text, secs) in [
        ("7days", 604_800),
        ("1h 30m", 5_400),
        ("10m", 600),
        ("0s", 0),
        // A month is 30.44 days.
        ("1month", 2_630_016),
    ] {
        let d = Duration::from_secs(secs);
        assert_eq!(human::parse_duration(text), Ok(d), "{text}");
        assert_eq!(human::format_duration(d).to_string(), text);
        assert_eq!(text.parse::<human::Duration>().unwrap().to_string(), text);

        let json = format!("{{\"pitr_interval\":\"{text}\",\"gc_period\":\"{text}\"}}");
        let config: TenantConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(
            config,
            TenantConfig {
                pitr_interval: d,
                gc_period: Some(d),
            }
        );
        assert_eq!(serde_json::to_string(&config).unwrap(), json);

        let toml = format!("pitr_interval = \"{text}\"\ngc_period = \"{text}\"\n");
        let config: TenantConfig = toml::from_str(&toml).unwrap();
        assert_eq!(config.pitr_interval, d);
        assert_eq!(toml::to_string(&config).unwrap(), toml);
    }

    // 30.44 days is a month; 30 and 31 days aren't.
    assert_eq!(
        human::format_duration(Duration::from_secs(30 * 86_400)).to_string(),
        "30days"
    );
    assert_eq!(
        human::format_duration(Duration::from_secs(31 * 86_400)).to_string(),
        "1month 13h 26m 24s"
    );
}

proptest! {
    /// Any duration is written as a string that reads back as it, through the functions, the
    /// newtype and the serde module.
    #[test]
    fn any_duration_round_trips(secs in any::<u64>(), nanos in 0..1_000_000_000u32) {
        let d = Duration::new(secs, nanos);
        let text = human::format_duration(d).to_string();
        prop_assert_eq!(human::parse_duration(&text), Ok(d));
        prop_assert_eq!(text.parse::<human::Duration>().map(Duration::from), Ok(d));
        let conf = Conf { d, o: Some(d), od: None };
        let json = serde_json::to_string(&conf).unwrap();
        prop_assert_eq!(serde_json::from_str::<Conf>(&json).unwrap(), conf);
    }

    /// No string makes either parser panic.
    #[test]
    fn parsers_never_panic(s in "\\PC{0,40}", t in "[0-9a-zµTZ.:+ -]{0,40}") {
        for s in [&s, &t] {
            let _ = human::parse_duration(s);
            let _ = human::parse_rfc3339(s);
        }
    }

    /// Any time from the epoch to the end of 9999 is written as a string that reads back as it.
    #[test]
    fn any_time_in_range_round_trips(secs in 0..YEAR_10000, nanos in 0..1_000_000_000u32) {
        let t = UNIX_EPOCH + Duration::new(secs, nanos);
        let text = human::format_rfc3339(t).unwrap().to_string();
        prop_assert_eq!(human::parse_rfc3339(&text), Ok(t));
    }
}
