//! `chelabase_time::human` against humantime 2.2.0 and humantime-serde 1.1.1, the versions the
//! Neon fork locks: duration formatting and parsing, the serde module, and the `SystemTime`
//! RFC 3339 helpers. `frozen_tables_are_humantimes` holds humantime to the literal tables the
//! library's own goldens use (`tests/humantime_tables.rs`), which stay when this crate goes.

use std::panic::catch_unwind;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chelabase_time::human;
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

#[path = "../../tests/humantime_tables.rs"]
mod tables;

const MINUTE: u64 = 60;
const HOUR: u64 = 3600;
const DAY: u64 = 86_400;
const MONTH: u64 = 2_630_016; // 30.44 days
const YEAR: u64 = 31_557_600; // 365.25 days

/// One past the last second humantime formats and parses: `10000-01-01T00:00:00Z`.
const YEAR_10000: u64 = 253_402_300_800;

/// Seconds over all of `u64`, crowded around the multiples of each unit and both ends.
fn seconds() -> impl Strategy<Value = u64> {
    let unit = prop::sample::select(vec![YEAR, MONTH, DAY, HOUR, MINUTE]);
    prop_oneof![
        any::<u64>(),
        0..253_370_764_800u64,
        0..200_000u64,
        (unit, 0..1000u64, -2i64..=2).prop_map(|(u, k, d)| (u * k).saturating_add_signed(d)),
        (0..3u64).prop_map(|d| u64::MAX - d),
    ]
}

fn nanoseconds() -> impl Strategy<Value = u32> {
    prop_oneof![
        Just(0u32),
        0..1_000_000_000u32,
        (1..1000u32, 0..3u32).prop_map(|(k, p)| k * 1000u32.pow(p)),
        Just(999_999_999u32),
    ]
}

fn durations() -> impl Strategy<Value = Duration> {
    (seconds(), nanoseconds()).prop_map(|(s, n)| Duration::new(s, n))
}

/// Duration strings: well-formed ones, sums at the top of the range, and strings near the
/// grammar (numbers small, huge and just past `u64`; every unit spelling of humantime 2.2.0 and
/// 2.4.0 plus wrong ones; ASCII and Unicode whitespace; stray punctuation), with some arbitrary
/// strings mixed in.
fn duration_strings() -> impl Strategy<Value = String> {
    let number = prop_oneof![
        (0..100u64).prop_map(|n| n.to_string()),
        any::<u64>().prop_map(|n| n.to_string()),
        (0..3u64).prop_map(|d| (u64::MAX - d).to_string()),
        prop::sample::select(vec![
            "18446744073709551616",
            "1000000000",
            "999999999",
            "1000000",
            "1000",
            "00",
            "",
        ])
        .prop_map(str::to_owned),
    ];
    let unit = prop::sample::select(vec![
        "nanos", "nsec", "ns", "usec", "us", "µs", "millis", "msec", "ms", "seconds", "second",
        "secs", "sec", "s", "minutes", "minute", "min", "mins", "m", "hours", "hour", "hr", "hrs",
        "h", "days", "day", "d", "weeks", "week", "wk", "wks", "w", "months", "month", "M",
        "years", "year", "yr", "yrs", "y", "parsecs", "S", "H", "Ms", "sm", "µ", "",
    ]);
    let gaps = vec![
        "", "", "", " ", " ", "  ", "\t", "\n", "\u{a0}", "\u{2003}", ".", ",", "-", "+", "_", "é",
    ];
    let item = (
        prop::sample::select(gaps.clone()),
        number,
        prop::sample::select(gaps.clone()),
        unit,
    )
        .prop_map(|(a, n, b, u)| format!("{a}{n}{b}{u}"));
    // Well-formed: humantime 2.2.0's units, modest numbers, at most a space between.
    let good_unit = prop::sample::select(vec![
        "nanos", "nsec", "ns", "usec", "us", "millis", "msec", "ms", "seconds", "second", "secs",
        "sec", "s", "minutes", "minute", "min", "mins", "m", "hours", "hour", "hr", "hrs", "h",
        "days", "day", "d", "weeks", "week", "w", "months", "month", "M", "years", "year", "y",
    ]);
    let space = || prop::sample::select(vec!["", " "]);
    let good_item = (0..2_000_000_000u64, space(), good_unit, space())
        .prop_map(|(n, a, u, b)| format!("{n}{a}{u}{b}"));
    // Sums at the top of the range. humantime 2.2.0 carries nanoseconds into seconds only
    // above one second, so exactly 1e9 ns stays pending (see `pending_second`): seconds near
    // `u64::MAX`, sub-second items around one second and near `u64::MAX`, then sometimes a
    // malformed item.
    let sub_second = || {
        (
            prop_oneof![
                prop::sample::select(vec![0u64, 1, 999_999_999, 1_000_000_000, 1_000_000_001]),
                (0..3u64).prop_map(|d| u64::MAX - d),
                (0..3u64).prop_map(|d| u64::MAX - 1_000_000_000 + 1 - d),
            ],
            prop::sample::select(vec![("ns", 1u64), ("us", 1000), ("ms", 1_000_000)]),
        )
            .prop_map(|(n, (unit, per))| format!("{}{unit}", n / per))
    };
    let top_piece = prop_oneof![
        3 => sub_second(),
        1 => (0..3u64).prop_map(|below| format!("{}s", u64::MAX - below)),
        1 => (0..3u64).prop_map(|n| format!("{n}s")),
    ];
    let top = (
        prop::collection::vec(top_piece, 1..5),
        prop::sample::select(vec!["", "", "", " 5parsecs", " 5", " !", " 1.5s", " 1wk"]),
    )
        .prop_map(|(pieces, tail)| pieces.join(" ") + tail);
    prop_oneof![
        4 => prop::collection::vec(good_item, 1..6).prop_map(|items| items.concat()),
        2 => top,
        4 => (prop::collection::vec(item, 0..5), prop::sample::select(gaps))
            .prop_map(|(items, end)| items.concat() + end),
        1 => "\\PC{0,12}",
        1 => "[0-9a-zµ. ]{0,10}",
    ]
}

/// True for a string in the grammar `parse_duration` has and humantime 2.2.0 lacks (the
/// differences the `human` module doc lists): the bare `0`, a `.` (fractions), a `µ`, or one of
/// the units `wk`, `wks`, `yr`, `yrs`.
fn wider_grammar(s: &str) -> bool {
    s == "0"
        || s.contains('.')
        || s.contains('µ')
        || s.split(|c: char| c.is_ascii_digit() || c.is_whitespace())
            .any(|unit| matches!(unit, "wk" | "wks" | "yr" | "yrs"))
}

/// The pending second, the one place the two parsers' arithmetic differs. humantime 2.2.0
/// carries its nanosecond counter into seconds only *above* one second, so a sum of exactly
/// 1,000,000,000 ns rests there with the second not yet counted; `parse_duration` carries at
/// once, so its counter is always below a second and its seconds are one higher. This is a
/// model of humantime's counter over well-formed items: true when it rests at exactly 1e9
/// after some item of `s`. From there on:
///
/// - a sub-second item above `u64::MAX - 1e9` ns overflows humantime's counter and not ours
///   (we accept, or go on to a later error);
/// - at `u64::MAX` seconds our carry overflows at once, where humantime reads on to a later
///   error, or to the end, where it panics.
///
/// Whatever humantime accepts we still accept with the same value.
fn pending_second(s: &str) -> bool {
    const SECOND: u64 = 1_000_000_000;
    let (mut secs, mut nanos) = (0u64, 0u64);
    let mut rest = s;
    loop {
        // A number: digits, with whitespace before and between them.
        let end = rest
            .find(|c: char| !(c.is_ascii_digit() || c.is_whitespace()))
            .unwrap_or(rest.len());
        let digits: String = rest[..end].chars().filter(char::is_ascii_digit).collect();
        let Ok(n) = digits.parse::<u64>() else {
            return false;
        };
        rest = &rest[end..];
        // A unit: letters up to whitespace, a digit or the end.
        let end = rest
            .find(|c: char| !c.is_ascii_alphabetic())
            .unwrap_or(rest.len());
        let (unit_secs, unit_nanos) = match &rest[..end] {
            "nanos" | "nsec" | "ns" => (0, 1),
            "usec" | "us" => (0, 1000),
            "millis" | "msec" | "ms" => (0, 1_000_000),
            "seconds" | "second" | "secs" | "sec" | "s" => (1, 0),
            "minutes" | "minute" | "min" | "mins" | "m" => (MINUTE, 0),
            "hours" | "hour" | "hr" | "hrs" | "h" => (HOUR, 0),
            "days" | "day" | "d" => (DAY, 0),
            "weeks" | "week" | "w" => (7 * DAY, 0),
            "months" | "month" | "M" => (MONTH, 0),
            "years" | "year" | "y" => (YEAR, 0),
            _ => return false,
        };
        rest = &rest[end..];
        // humantime's `parse_unit`: any overflow is its error, and the parse ends.
        let step = || {
            let mut sec = n.checked_mul(unit_secs)?;
            let mut nsec = nanos.checked_add(n.checked_mul(unit_nanos)?)?;
            if nsec > SECOND {
                sec = sec.checked_add(nsec / SECOND)?;
                nsec %= SECOND;
            }
            Some((secs.checked_add(sec)?, nsec))
        };
        let Some(next) = step() else {
            return false;
        };
        (secs, nanos) = next;
        if nanos == SECOND {
            return true;
        }
    }
}

/// humantime 2.2.0's result, or `None` where it panics.
fn humantime_parse(s: &str) -> Option<Result<Duration, humantime::DurationError>> {
    let s = s.to_owned();
    catch_unwind(move || humantime::parse_duration(&s)).ok()
}

/// Compares `parse_duration` with humantime 2.2.0 on one string.
fn check_parse(s: &str) -> Result<(), TestCaseError> {
    let ours = human::parse_duration(s);
    let Some(theirs) = humantime_parse(s) else {
        // humantime panics; we return the overflow error.
        prop_assert_eq!(ours, Err(human::DurationError::NumberOverflow), "{:?}", s);
        return Ok(());
    };
    let pending = pending_second(s);
    match (theirs, ours) {
        (Ok(theirs), ours) => prop_assert_eq!(ours, Ok(theirs), "{:?}", s),
        (Err(theirs), Ok(ours)) => {
            prop_assert!(
                wider_grammar(s) || pending,
                "{s:?}: humantime rejects ({theirs:?}), we accept ({ours:?}), and it is neither \
                 in the listed wider grammar nor a pending second"
            );
        }
        (Err(_), Err(_)) if wider_grammar(s) => {}
        (Err(theirs), Err(ours)) => {
            let same = format!("{ours:?}") == format!("{theirs:?}");
            if pending {
                // One stopped at the pending second with the overflow error; the other read on.
                let overflow = ours == human::DurationError::NumberOverflow
                    || theirs == humantime::DurationError::NumberOverflow;
                prop_assert!(same || overflow, "{s:?}: {ours:?} and {theirs:?}");
            } else {
                prop_assert!(same, "{s:?}: {ours:?} and {theirs:?}");
            }
            // Two messages are humantime 2.4.0's wording (listed in the module doc).
            let reworded = match &ours {
                human::DurationError::NumberOverflow => true,
                human::DurationError::UnknownUnit { unit, .. } => !unit.is_empty(),
                _ => false,
            };
            if same && !reworded {
                prop_assert_eq!(ours.to_string(), theirs.to_string(), "{:?}", s);
            }
        }
    }
    Ok(())
}

#[test]
fn format_duration_matches() {
    proptest!(ProptestConfig::with_cases(20_000), |(d in durations())| {
        let theirs = humantime::format_duration(d).to_string();
        let ours = human::format_duration(d).to_string();
        prop_assert_eq!(&ours, &theirs);
        // The clap newtype prints the same text as humantime's.
        prop_assert_eq!(human::Duration::from(d).to_string(), humantime::Duration::from(d).to_string());
        // What humantime wrote reads back as the duration, with either parser.
        prop_assert_eq!(human::parse_duration(&theirs), Ok(d));
        prop_assert_eq!(humantime::parse_duration(&theirs), Ok(d));
        prop_assert_eq!(theirs.parse::<human::Duration>().map(Duration::from), Ok(d));
    });
}

#[test]
fn parse_duration_matches_accept_and_reject() {
    proptest!(ProptestConfig::with_cases(40_000), |(s in duration_strings())| {
        check_parse(&s)?;
    });
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Theirs {
    #[serde(with = "humantime_serde")]
    d: Duration,
    #[serde(with = "humantime_serde")]
    o: Option<Duration>,
    #[serde(default, with = "humantime_serde")]
    od: Option<Duration>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Ours {
    #[serde(with = "chelabase_time::human::serde")]
    d: Duration,
    #[serde(with = "chelabase_time::human::serde")]
    o: Option<Duration>,
    #[serde(default, with = "chelabase_time::human::serde")]
    od: Option<Duration>,
}

/// The shape of the proxy's `ConcurrencyLockOptions`.
#[derive(Deserialize)]
struct TheirEpoch {
    #[serde(deserialize_with = "humantime_serde::deserialize")]
    epoch: Duration,
}

#[derive(Deserialize)]
struct OurEpoch {
    #[serde(deserialize_with = "chelabase_time::human::serde::deserialize")]
    epoch: Duration,
}

/// The same fields as plain strings, to write documents with arbitrary duration text.
#[derive(Serialize)]
struct Raw<'a> {
    d: &'a str,
    o: &'a str,
}

type Fields = (Duration, Option<Duration>, Option<Duration>);

fn text<T: ToString, E: ToString>(r: Result<T, E>) -> Result<String, String> {
    r.map(|v| v.to_string()).map_err(|e| e.to_string())
}

fn their_fields<E: ToString>(r: Result<Theirs, E>) -> Result<Fields, String> {
    r.map(|c| (c.d, c.o, c.od)).map_err(|e| e.to_string())
}

fn our_fields<E: ToString>(r: Result<Ours, E>) -> Result<Fields, String> {
    r.map(|c| (c.d, c.o, c.od)).map_err(|e| e.to_string())
}

#[test]
fn serde_module_matches_humantime_serde() {
    let option = || prop::option::of(durations());
    proptest!(ProptestConfig::with_cases(5_000), |(d in durations(), o in option(), od in option())| {
        let theirs = Theirs { d, o, od };
        let ours = Ours { d, o, od };

        let json = text(serde_json::to_string(&theirs));
        prop_assert_eq!(&text(serde_json::to_string(&ours)), &json);
        let json = json.unwrap();
        prop_assert_eq!(our_fields(serde_json::from_str(&json)), Ok((d, o, od)));
        prop_assert_eq!(their_fields(serde_json::from_str(&json)), Ok((d, o, od)));

        let toml = text(toml::to_string(&theirs));
        prop_assert_eq!(&text(toml::to_string(&ours)), &toml);
        let toml = toml.unwrap();
        // TOML writes no key for `None`, so `o` (no `default`) reads back only when it is
        // `Some`; either way both read the same.
        let back = our_fields(toml::from_str(&toml));
        prop_assert_eq!(&back, &their_fields(toml::from_str(&toml)));
        if o.is_some() {
            prop_assert_eq!(back, Ok((d, o, od)));
        }
    });

    // Arbitrary duration text, malformed included: the same value or the same error text,
    // outside the wider grammar, the pending second and the strings humantime panics on.
    proptest!(ProptestConfig::with_cases(40_000), |(d in duration_strings(), o in duration_strings())| {
        // Skipped, not rejected: about half the generated pairs remain.
        if [&d, &o]
            .iter()
            .any(|s| wider_grammar(s) || pending_second(s) || humantime_parse(s).is_none())
        {
            return Ok(());
        }
        let raw = Raw { d: &d, o: &o };

        let json = serde_json::to_string(&raw).unwrap();
        prop_assert_eq!(
            our_fields(serde_json::from_str(&json)),
            their_fields(serde_json::from_str(&json)),
            "{}", json
        );
        let epoch = serde_json::json!({ "epoch": d }).to_string();
        prop_assert_eq!(
            serde_json::from_str::<OurEpoch>(&epoch).map(|e| e.epoch).map_err(|e| e.to_string()),
            serde_json::from_str::<TheirEpoch>(&epoch).map(|e| e.epoch).map_err(|e| e.to_string()),
            "{}", epoch
        );

        let toml = toml::to_string(&raw).unwrap();
        prop_assert_eq!(
            our_fields(toml::from_str(&toml)),
            their_fields(toml::from_str(&toml)),
            "{}", toml
        );
    });
}

/// Times humantime formats: the epoch to the end of 9999, crowded at both ends and around
/// `Timestamp::MAX` (`9999-12-30T22:00:00Z`), which these helpers go past.
fn times() -> impl Strategy<Value = (u64, u32)> {
    let secs = prop_oneof![
        0..YEAR_10000,
        0..4_200_000_000u64,
        (YEAR_10000 - 30 * HOUR)..YEAR_10000,
        (0..200_000u64).prop_map(|d| YEAR_10000 - 1 - d),
        0..200_000u64,
    ];
    (secs, nanoseconds())
}

/// Well-shaped timestamps whose fields run a little past their ranges (day 0 and 32, month 13,
/// hour 24, second 61, years before 1970), with and without a fraction.
fn field_strings() -> impl Strategy<Value = String> {
    let year = prop_oneof![0..=9999u32, 1965..=1975u32, 9990..=9999u32];
    let fraction = prop_oneof![
        3 => Just(String::new()),
        1 => "\\.[0-9]{0,12}",
    ];
    (
        year,
        0..=13u32,
        0..=32u32,
        0..=24u32,
        0..=60u32,
        0..=61u32,
        fraction,
    )
        .prop_map(|(y, mo, d, h, mi, s, f)| {
            format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}{f}Z")
        })
}

/// A formatted timestamp with a few characters replaced, inserted or removed.
fn damaged_strings() -> impl Strategy<Value = String> {
    let edit = (
        any::<prop::sample::Index>(),
        prop::sample::select(vec![
            '0', '1', '5', '9', 'T', 'Z', 't', 'z', ' ', '.', ':', '-', '+', ',', 'é', '١',
        ]),
        0..3u8,
    );
    (times(), 0..3u8, prop::collection::vec(edit, 0..3)).prop_map(|((secs, nanos), form, edits)| {
        let t = UNIX_EPOCH + Duration::new(secs, nanos);
        let base = match form {
            0 => humantime::format_rfc3339(t).to_string(),
            1 => humantime::format_rfc3339_seconds(t).to_string(),
            _ => humantime::format_rfc3339_millis(t).to_string(),
        };
        let mut chars: Vec<char> = base.chars().collect();
        for (at, c, kind) in edits {
            let i = at.index(chars.len() + 1);
            match kind {
                0 if i < chars.len() => chars[i] = c,
                1 if i < chars.len() => {
                    chars.remove(i);
                }
                _ => chars.insert(i, c),
            }
        }
        chars.into_iter().collect()
    })
}

/// A parse result as seconds and nanoseconds since the epoch, or the error's `Debug` and
/// `Display`.
fn since_epoch<E: std::fmt::Debug + ToString>(
    r: Result<SystemTime, E>,
) -> Result<(u64, u32), (String, String)> {
    match r {
        Ok(t) => {
            let d = t.duration_since(UNIX_EPOCH).expect("not before the epoch");
            Ok((d.as_secs(), d.subsec_nanos()))
        }
        Err(e) => Err((format!("{e:?}"), e.to_string())),
    }
}

fn check_rfc3339_parse(s: &str) -> Result<(), TestCaseError> {
    prop_assert_eq!(
        since_epoch(human::parse_rfc3339(s)),
        since_epoch(humantime::parse_rfc3339(s)),
        "{:?}",
        s
    );
    Ok(())
}

/// The fork's `utils::serde_system_time` and `SystemTimeAsRfc3339Millis`, as they are written
/// with humantime.
#[derive(Serialize, Deserialize)]
struct TheirLease {
    #[serde(
        serialize_with = "their_ser_rfc3339_millis",
        deserialize_with = "their_deser_rfc3339_millis"
    )]
    valid_until: SystemTime,
}

fn their_ser_rfc3339_millis<S: serde::Serializer>(
    ts: &SystemTime,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_str(&humantime::format_rfc3339_millis(*ts))
}

fn their_deser_rfc3339_millis<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<SystemTime, D::Error> {
    let s: String = Deserialize::deserialize(deserializer)?;
    humantime::parse_rfc3339(&s).map_err(serde::de::Error::custom)
}

#[derive(Serialize, Deserialize)]
struct OurLease {
    #[serde(with = "chelabase_time::human::rfc3339_millis")]
    valid_until: SystemTime,
}

#[test]
fn rfc3339_formats_and_strict_parse_match() {
    proptest!(ProptestConfig::with_cases(20_000), |((secs, nanos) in times())| {
        let t = UNIX_EPOCH + Duration::new(secs, nanos);
        let smart = humantime::format_rfc3339(t).to_string();
        let seconds = humantime::format_rfc3339_seconds(t).to_string();
        let millis = humantime::format_rfc3339_millis(t).to_string();
        prop_assert_eq!(&human::format_rfc3339(t).unwrap().to_string(), &smart);
        prop_assert_eq!(&human::format_rfc3339_seconds(t).unwrap().to_string(), &seconds);
        prop_assert_eq!(&human::format_rfc3339_millis(t).unwrap().to_string(), &millis);
        // The smart form keeps every nanosecond, so it reads back as the same time.
        prop_assert_eq!(human::parse_rfc3339(&smart), Ok(t));
        for s in [&smart, &seconds, &millis] {
            check_rfc3339_parse(s)?;
        }

        // The serde module writes and reads what the fork's humantime code does.
        let json = serde_json::to_string(&TheirLease { valid_until: t }).unwrap();
        prop_assert_eq!(&serde_json::to_string(&OurLease { valid_until: t }).unwrap(), &json);
        let back: OurLease = serde_json::from_str(&json).unwrap();
        let their_back: TheirLease = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.valid_until, their_back.valid_until);
    });

    proptest!(ProptestConfig::with_cases(40_000), |(s in field_strings())| {
        check_rfc3339_parse(&s)?;
    });

    proptest!(ProptestConfig::with_cases(40_000), |(s in damaged_strings())| {
        check_rfc3339_parse(&s)?;
        // Through serde: the same time or the same error text.
        let json = serde_json::json!({ "valid_until": s }).to_string();
        prop_assert_eq!(
            serde_json::from_str::<OurLease>(&json).map(|l| l.valid_until).map_err(|e| e.to_string()),
            serde_json::from_str::<TheirLease>(&json).map(|l| l.valid_until).map_err(|e| e.to_string()),
            "{}", json
        );
    });

    proptest!(ProptestConfig::with_cases(5_000), |(s in "\\PC{0,32}")| {
        check_rfc3339_parse(&s)?;
    });

    // Offsets other than `Z` are refused by both.
    for s in [
        "2018-02-14T00:28:07+01:00",
        "2018-02-14T00:28:07+00:00",
        "2018-02-14T00:28:07-00:00",
    ] {
        assert_eq!(
            since_epoch(human::parse_rfc3339(s)),
            Err((
                "InvalidFormat".to_owned(),
                "timestamp format is invalid".to_owned()
            )),
            "{s}"
        );
        assert!(humantime::parse_rfc3339(s).is_err(), "{s}");
    }

    // Outside humantime's domain it panics (before the epoch) or its `Display` fails, which
    // panics in `to_string` (year 10000 on). We return an error from all three.
    let outside = [
        UNIX_EPOCH - Duration::new(0, 1),
        UNIX_EPOCH - Duration::from_secs(1),
        UNIX_EPOCH - Duration::from_secs(YEAR_10000),
        UNIX_EPOCH + Duration::from_secs(YEAR_10000),
        UNIX_EPOCH + Duration::new(YEAR_10000, 999_999_999),
        UNIX_EPOCH + Duration::from_secs(1 << 40),
    ];
    for t in outside {
        assert!(
            catch_unwind(|| humantime::format_rfc3339(t).to_string()).is_err(),
            "{t:?}"
        );
        assert!(catch_unwind(|| humantime::format_rfc3339_seconds(t).to_string()).is_err());
        assert!(catch_unwind(|| humantime::format_rfc3339_millis(t).to_string()).is_err());
        for ours in [
            human::format_rfc3339(t),
            human::format_rfc3339_seconds(t),
            human::format_rfc3339_millis(t),
        ] {
            assert_eq!(
                ours.map(|f| f.to_string()),
                Err(human::TimestampError::OutOfRange)
            );
        }
    }
}

fn std_duration((secs, nanos): tables::D) -> Duration {
    Duration::new(secs, nanos)
}

fn expected_fields(
    row: &Result<(tables::D, Option<tables::D>, Option<tables::D>), &str>,
) -> Result<Fields, String> {
    match row {
        Ok((d, o, od)) => Ok((std_duration(*d), o.map(std_duration), od.map(std_duration))),
        Err(e) => Err((*e).to_owned()),
    }
}

fn expected_text(row: &Result<&str, &str>) -> Result<String, String> {
    row.map(str::to_owned).map_err(str::to_owned)
}

/// Every frozen row is what humantime 2.2.0 and humantime-serde 1.1.1 do. The library's
/// `tests/humantime_goldens.rs` holds `chelabase_time::human` to the same rows.
#[test]
fn frozen_tables_are_humantimes() {
    for &(secs, nanos, text) in tables::FORMAT_DURATION {
        let d = Duration::new(secs, nanos);
        assert_eq!(humantime::format_duration(d).to_string(), text);
        assert_eq!(humantime::parse_duration(text), Ok(d), "{text}");
    }
    for &(text, secs, nanos) in tables::PARSE_DURATION_OK {
        assert_eq!(
            humantime::parse_duration(text),
            Ok(Duration::new(secs, nanos)),
            "{text:?}"
        );
    }
    for &(text, debug) in tables::PARSE_DURATION_ERR {
        let e = humantime::parse_duration(text).unwrap_err();
        assert_eq!(format!("{e:?}"), debug, "{text:?}");
    }
    for &(text, _, _, debug) in tables::PARSE_DURATION_WIDER {
        let e = humantime::parse_duration(text).unwrap_err();
        assert_eq!(format!("{e:?}"), debug, "{text:?}");
        assert!(wider_grammar(text), "{text:?}");
    }
    for &(text, _, debug) in tables::PARSE_DURATION_OTHER_ERR {
        let e = humantime::parse_duration(text).unwrap_err();
        assert_eq!(format!("{e:?}"), debug, "{text:?}");
        assert!(wider_grammar(text), "{text:?}");
    }
    for &(text, _, _, debug) in tables::PARSE_DURATION_PENDING_SECOND_OK {
        let e = humantime::parse_duration(text).unwrap_err();
        assert_eq!(format!("{e:?}"), debug, "{text:?}");
        assert!(pending_second(text) && !wider_grammar(text), "{text:?}");
    }
    for &(text, _, debug) in tables::PARSE_DURATION_PENDING_SECOND_ERR {
        let e = humantime::parse_duration(text).unwrap_err();
        assert_eq!(format!("{e:?}"), debug, "{text:?}");
        assert!(pending_second(text) && !wider_grammar(text), "{text:?}");
    }
    for &text in tables::PARSE_DURATION_HUMANTIME_PANICS {
        assert!(humantime_parse(text).is_none(), "{text:?}");
        assert!(pending_second(text), "{text:?}");
    }
    for &(text, _, display) in tables::DURATION_ERROR_DISPLAY {
        assert_eq!(
            humantime::parse_duration(text).unwrap_err().to_string(),
            display,
            "{text:?}"
        );
    }

    for (d, o, od, json, toml) in tables::SERDE_SERIALIZE {
        let conf = Theirs {
            d: std_duration(*d),
            o: o.map(std_duration),
            od: od.map(std_duration),
        };
        assert_eq!(text(serde_json::to_string(&conf)), expected_text(json));
        assert_eq!(text(toml::to_string(&conf)), expected_text(toml));
    }
    for (doc, row) in tables::SERDE_DESERIALIZE_JSON {
        assert_eq!(
            their_fields(serde_json::from_str(doc)),
            expected_fields(row),
            "{doc}"
        );
    }
    for (doc, row) in tables::SERDE_DESERIALIZE_TOML {
        assert_eq!(
            their_fields(toml::from_str(doc)),
            expected_fields(row),
            "{doc}"
        );
    }
    for (doc, row) in tables::SERDE_DESERIALIZE_WITH_JSON {
        assert_eq!(
            serde_json::from_str::<TheirEpoch>(doc)
                .map(|e| e.epoch)
                .map_err(|e| e.to_string()),
            row.map(std_duration).map_err(str::to_owned),
            "{doc}"
        );
    }

    for &(secs, nanos, smart, seconds, millis) in tables::RFC3339_FORMAT {
        let t = UNIX_EPOCH + Duration::new(secs, nanos);
        assert_eq!(humantime::format_rfc3339(t).to_string(), smart);
        assert_eq!(humantime::format_rfc3339_seconds(t).to_string(), seconds);
        assert_eq!(humantime::format_rfc3339_millis(t).to_string(), millis);
    }
    for (doc, row) in tables::RFC3339_MILLIS_DESERIALIZE_JSON {
        assert_eq!(
            serde_json::from_str::<TheirLease>(doc)
                .map(|l| l.valid_until)
                .map_err(|e| e.to_string()),
            row.map(|d| UNIX_EPOCH + std_duration(d))
                .map_err(str::to_owned),
            "{doc}"
        );
    }
    for &(text, secs, nanos) in tables::RFC3339_PARSE_OK {
        assert_eq!(
            since_epoch(humantime::parse_rfc3339(text)),
            Ok((secs, nanos)),
            "{text:?}"
        );
    }
    for &(text, debug, display) in tables::RFC3339_PARSE_ERR {
        assert_eq!(
            since_epoch(humantime::parse_rfc3339(text)),
            Err((debug.to_owned(), display.to_owned())),
            "{text:?}"
        );
    }
}
