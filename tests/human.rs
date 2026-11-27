//! `chelabase_time::human`: jiff's friendly format on write, jiff then humantime's grammar on
//! read.

use std::time::Duration;

use chelabase_time::{human, parse_duration};
use clap::Parser;
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
    #[serde(with = "human::serde::required")]
    interval: Duration,
    #[serde(with = "human::serde::optional", default)]
    timeout: Option<Duration>,
}

/// The same struct on jiff's own helpers, to show `None` behaves the same.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct JiffConfig {
    #[serde(with = "jiff::fmt::serde::unsigned_duration::friendly::compact::required")]
    interval: Duration,
    #[serde(
        with = "jiff::fmt::serde::unsigned_duration::friendly::compact::optional",
        default
    )]
    timeout: Option<Duration>,
}

const fn d(secs: u64, nanos: u32) -> Duration {
    Duration::new(secs, nanos)
}

/// What jiff's friendly compact format writes for each value.
const WRITTEN: &[(Duration, &str)] = &[
    (d(0, 0), "0s"),
    (d(5400, 0), "1h 30m"),
    (d(7 * 86_400, 0), "168h"),
    (d(1, 500_000_000), "1s 500ms"),
    (d(30 * 86_400 + 1, 0), "720h 1s"),
    (d(0, 100_000_000), "100ms"),
    (d(0, 3_000), "3µs"),
    (d(0, 1), "1ns"),
    (d(3661, 1_001_001), "1h 1m 1s 1ms 1µs 1ns"),
    (d(2_630_016, 0), "730h 33m 36s"),
];

#[test]
fn round_trip_json_and_toml() {
    for &(value, text) in WRITTEN {
        assert_eq!(
            human::Duration::from(value).to_string(),
            text,
            "Display of {value:?}"
        );
        for timeout in [None, Some(value)] {
            let ours = Config {
                interval: value,
                timeout,
            };
            let jiffs = JiffConfig {
                interval: value,
                timeout,
            };

            let json = serde_json::to_string(&ours).unwrap();
            assert_eq!(json, serde_json::to_string(&jiffs).unwrap());
            let expected_timeout = match timeout {
                None => "null".to_owned(),
                Some(_) => format!("\"{text}\""),
            };
            assert_eq!(
                json,
                format!("{{\"interval\":\"{text}\",\"timeout\":{expected_timeout}}}")
            );
            assert_eq!(serde_json::from_str::<Config>(&json).unwrap(), ours);
            assert_eq!(serde_json::from_str::<JiffConfig>(&json).unwrap(), jiffs);

            let toml_text = toml::to_string(&ours).unwrap();
            assert_eq!(toml_text, toml::to_string(&jiffs).unwrap());
            assert!(
                toml_text.contains(&format!("interval = \"{text}\"")),
                "{toml_text}"
            );
            assert_eq!(toml::from_str::<Config>(&toml_text).unwrap(), ours);
            assert_eq!(toml::from_str::<JiffConfig>(&toml_text).unwrap(), jiffs);
        }
    }

    // A missing optional field is `None` with `default`, for both helpers.
    let ours: Config = serde_json::from_str(r#"{"interval":"10m"}"#).unwrap();
    let jiffs: JiffConfig = serde_json::from_str(r#"{"interval":"10m"}"#).unwrap();
    assert_eq!(ours.timeout, None);
    assert_eq!(jiffs.timeout, None);
    let ours: Config = toml::from_str("interval = \"10m\"").unwrap();
    assert_eq!(ours.timeout, None);
}

/// Spellings humantime 2.x wrote or accepted, with humantime's value.
const HUMANTIME: &[(&str, Duration)] = &[
    ("7days", d(7 * 86_400, 0)),
    ("2weeks", d(14 * 86_400, 0)),
    ("1d 2h", d(26 * 3600, 0)),
    ("1month", d(2_630_016, 0)),
    ("1year", d(31_557_600, 0)),
    ("1M", d(2_630_016, 0)),
    ("1y", d(31_557_600, 0)),
    ("1w", d(7 * 86_400, 0)),
    ("1h 30m", d(5400, 0)),
    ("1h30m", d(5400, 0)),
    ("10m", d(600, 0)),
    ("0s", d(0, 0)),
    ("0", d(0, 0)),
    ("5s 300ms", d(5, 300_000_000)),
    ("100ms", d(0, 100_000_000)),
    ("3us", d(0, 3_000)),
    ("3µs", d(0, 3_000)),
    ("15ns", d(0, 15)),
    ("1.5s", d(1, 500_000_000)),
    ("604800s", d(604_800, 0)),
    ("2h 37min", d(2 * 3600 + 37 * 60, 0)),
];

#[test]
fn reads_old_humantime_spellings() {
    for &(text, value) in HUMANTIME {
        assert_eq!(human::parse(text).unwrap(), value, "parse {text:?}");
        assert_eq!(
            text.parse::<human::Duration>().unwrap(),
            value.into(),
            "FromStr {text:?}"
        );
        let json = format!("{{\"interval\":\"{text}\",\"timeout\":\"{text}\"}}");
        let c: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(
            (c.interval, c.timeout),
            (value, Some(value)),
            "serde {text:?}"
        );
    }
    // ISO 8601, which jiff reads.
    assert_eq!(human::parse("PT1H30M").unwrap(), d(5400, 0));
    let c: Config = toml::from_str("interval = \"PT1H30M\"").unwrap();
    assert_eq!(c.interval, d(5400, 0));
}

#[test]
fn rejects_malformed() {
    for text in ["5parsecs", "", "-1s", "   ", "1h -30m", "5", "1.5.5s"] {
        let err = human::parse(text).unwrap_err();
        assert!(!err.to_string().is_empty());
        assert!(text.parse::<human::Duration>().is_err(), "FromStr {text:?}");
        let json = format!("{{\"interval\":{}}}", serde_json::to_string(text).unwrap());
        assert!(
            serde_json::from_str::<Config>(&json).is_err(),
            "serde {text:?}"
        );
    }
    // A non-string is refused too.
    assert!(serde_json::from_str::<Config>(r#"{"interval":5}"#).is_err());
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    x: human::Duration,
}

#[test]
fn clap_value_parser() {
    for (arg, value) in [
        ("1h30m", d(5400, 0)),
        ("1h 30m", d(5400, 0)),
        ("7days", d(7 * 86_400, 0)),
    ] {
        let args = Args::try_parse_from(["t", "--x", arg]).unwrap();
        assert_eq!(Duration::from(args.x), value, "--x {arg:?}");
    }
    assert!(Args::try_parse_from(["t", "--x", "5parsecs"]).is_err());
}

#[test]
fn parse_duration_reads_jiff_output() {
    for &(value, text) in WRITTEN {
        assert_eq!(
            parse_duration(text).unwrap(),
            value,
            "parse_duration({text:?})"
        );
        assert_eq!(human::parse(text).unwrap(), value, "human::parse({text:?})");
    }
}

/// Every unit spelling `parse_duration` knows (its module docs).
const UNITS: &[&str] = &[
    "nanos", "nsec", "ns", "usec", "us", "µs", "millis", "msec", "ms", "seconds", "second", "secs",
    "sec", "s", "minutes", "minute", "min", "mins", "m", "hours", "hour", "hr", "hrs", "h", "days",
    "day", "d", "weeks", "week", "wk", "wks", "w", "months", "month", "M", "years", "year", "yr",
    "yrs", "y",
];

/// jiff alone, as its own serde helper reads.
fn jiff_only(text: &str) -> Option<Duration> {
    use serde::de::value::{Error, StrDeserializer};
    jiff::fmt::serde::unsigned_duration::required::deserialize(StrDeserializer::<Error>::new(text))
        .ok()
}

/// Where both grammars read a spelling, they agree: `human::parse` equals `parse_duration`
/// for every unit alias, and jiff alone, when it accepts, gives the same value.
#[test]
fn precedence_table() {
    // The rows of the module docs' table where only one grammar reads the spelling.
    for jiff_only_text in ["PT1H30M", "2 hrs, 30 mins"] {
        assert!(jiff_only(jiff_only_text).is_some(), "{jiff_only_text:?}");
        assert!(
            parse_duration(jiff_only_text).is_err(),
            "{jiff_only_text:?}"
        );
    }
    for humantime_only in [
        "1d", "1day", "2days", "1w", "1wk", "2weeks", "1M", "1month", "1y", "1yr", "1year", "0",
    ] {
        assert!(jiff_only(humantime_only).is_none(), "{humantime_only:?}");
        assert!(parse_duration(humantime_only).is_ok(), "{humantime_only:?}");
    }
    assert!(jiff_only("-1s").is_none() && parse_duration("-1s").is_err());

    for unit in UNITS {
        for number in ["0", "1", "2", "1.5", "250"] {
            for sep in ["", " "] {
                let text = format!("{number}{sep}{unit}");
                let Ok(expected) = parse_duration(&text) else {
                    continue;
                };
                assert_eq!(human::parse(&text).unwrap(), expected, "{text:?}");
                if let Some(jiff) = jiff_only(&text) {
                    assert_eq!(jiff, expected, "jiff reads {text:?} differently");
                }
            }
        }
    }
}

fn humantime_item() -> impl Strategy<Value = String> {
    (
        0u32..10_000,
        prop::option::of(0u32..1000),
        prop::sample::select(UNITS),
        prop::sample::select(&["", " "][..]),
    )
        .prop_map(|(n, frac, unit, sep)| match frac {
            Some(f) => format!("{n}.{f}{sep}{unit}"),
            None => format!("{n}{sep}{unit}"),
        })
}

proptest! {
    /// Any string in humantime's grammar that `parse_duration` reads, `human::parse` reads
    /// to the same value, whichever parser takes it.
    #[test]
    fn humantime_grammar_reads_the_same(
        items in prop::collection::vec(humantime_item(), 1..5),
        sep in prop::sample::select(&["", " "][..]),
    ) {
        let text = items.join(sep);
        if let Ok(expected) = parse_duration(&text) {
            prop_assert_eq!(human::parse(&text).unwrap(), expected, "{:?}", text);
        }
    }

    /// Whatever the wrapper writes, it and `parse_duration` read back.
    #[test]
    fn written_reads_back(secs in 0u64..=u64::MAX / 2, nanos in 0u32..1_000_000_000) {
        let value = d(secs, nanos);
        let text = human::Duration::from(value).to_string();
        prop_assert_eq!(human::parse(&text).unwrap(), value);
        prop_assert_eq!(parse_duration(&text).unwrap(), value);
    }
}
