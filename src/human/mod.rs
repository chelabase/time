//! Replacements for humantime 2.2.0 and humantime-serde 1.1.1, the versions the Neon fork
//! locked (epic neon-drop-chrono): durations as `1h 30m` and `7days`, and `SystemTime` as
//! RFC 3339.
//!
//! | humantime | Here |
//! |---|---|
//! | `humantime::format_duration(d)` | [`format_duration`]`(d)` |
//! | `humantime::parse_duration(s)` | [`parse_duration`]`(s)` (the crate's one parser) |
//! | `humantime::DurationError` | [`DurationError`] |
//! | `humantime::Duration` (clap arguments, `FromStr`, `Display`) | [`Duration`] |
//! | `#[serde(with = "humantime_serde")]` on `Duration` and `Option<Duration>` | `#[serde(with = "chelabase_time::human::serde")]` ([`serde`]) |
//! | `#[serde(deserialize_with = "humantime_serde::deserialize")]` | `#[serde(deserialize_with = "chelabase_time::human::serde::deserialize")]` |
//! | `humantime::format_rfc3339(t)`, `_seconds`, `_millis` | [`format_rfc3339`]`(t)?`, [`format_rfc3339_seconds`], [`format_rfc3339_millis`] (a `Result`) |
//! | `humantime::parse_rfc3339(s)` | [`parse_rfc3339`]`(s)` |
//! | `humantime::TimestampError` | [`TimestampError`] |
//! | a `SystemTime` field written with `format_rfc3339_millis` and read with `parse_rfc3339` | `#[serde(with = "chelabase_time::human::rfc3339_millis")]` ([`rfc3339_millis`]) |
//!
//! # The contract
//!
//! The storage controller keeps these duration strings in `tenant_shards.config` and chelabase
//! reads them back, so what is written is humantime 2.2.0's, byte for byte:
//!
//! - **[`format_duration`]** gives humantime 2.2.0's string for every `std::time::Duration`
//!   (a year is 365.25 days and a month 30.44 days, so 30 days is `30days` and 2,630,016
//!   seconds is `1month`). [`Duration`]'s `Display` and the [`serde`] module write the same.
//! - **[`parse_duration`]** reads every string humantime 2.2.0 reads, to the same value, and
//!   so every string [`format_duration`] writes. It reads more than 2.2.0 did; see below.
//! - **The [`serde`] module** writes and reads a `Duration` and an `Option<Duration>` as
//!   humantime-serde 1.1.1 did, in JSON and TOML, with the same error text for a value that
//!   isn't a duration string.
//! - **[`format_rfc3339`], [`format_rfc3339_seconds`] and [`format_rfc3339_millis`]** give
//!   humantime's strings for every time from 1970-01-01 to the end of 9999.
//! - **[`parse_rfc3339`]** accepts and refuses the same strings as humantime 2.2.0's strict
//!   parser, with the same value and the same error: `Z` only, 1970 to
//!   `9999-12-31T23:59:60Z`, second `60` read as `59`. Its range is humantime's, not
//!   [`Timestamp`](crate::Timestamp)'s, so `9999-12-31T23:00:00Z` parses.
//!
//! `tests/humantime_goldens.rs` holds this to literal tables printed by humantime
//! (`tests/humantime_tables.rs`); while the oracle crate exists, `oracle/tests/humantime.rs`
//! compares the two on generated input, with a model of the pending second (below).
//!
//! # Differences from humantime 2.2.0
//!
//! Nothing is formatted differently. The differences are in what is read, in two error
//! messages, and where humantime panicked.
//!
//! **[`parse_duration`] accepts more.** It has humantime 2.4.0's grammar, which chelabase
//! already relied on. Each of these is an error in humantime 2.2.0 and a value here:
//!
//! - a fraction: `1.5s`, `0.5h`, `1h 1.5m`;
//! - the unit `µs`;
//! - the units `wk`, `wks`, `yr` and `yrs`;
//! - the bare string `0`.
//!
//! - a sub-second item of more than `u64::MAX - 1_000_000_000` nanoseconds right after a sum
//!   of exactly one second of sub-second items: `1000ms 18446744073709551615ns`. See the
//!   pending second below.
//!
//! So a config value or a flag that the fork used to refuse for one of these is now accepted.
//! The same holds for [`Duration`]'s `FromStr` and the [`serde`] module's reads.
//!
//! **The pending second.** This is the one place the arithmetic differs, and the invariant
//! behind the last item above and two below. humantime 2.2.0 carried its nanosecond counter
//! into seconds only *above* one second, so a sum of exactly 1,000,000,000 ns stayed in the
//! counter with the second not yet counted. This parser carries at once: its counter is always
//! below a second. Every string humantime 2.2.0 accepts still gives the same value; the two
//! differ only while humantime's counter rests at exactly 1e9, and only at the top of the
//! range:
//!
//! - a next sub-second item that fits here overflowed humantime's counter (the accepted case
//!   above);
//! - at `u64::MAX` seconds, this parser's carry overflows at that item (`NumberOverflow`),
//!   where humantime read on: to a later error (`18446744073709551615s 1000ms 5parsecs` was
//!   `UnknownUnit` there), or to the end, where it panicked.
//!
//! **Errors of [`parse_duration`].** For a string both refuse, the [`DurationError`] variant
//! and its fields are humantime 2.2.0's, except:
//!
//! - a refused string with a `.` or a `µ` in it can carry another variant, because 2.2.0
//!   stopped at that character with `InvalidCharacter` and this parser reads on (`1.5ns` is
//!   `NumberOverflow`, `1µ` is `UnknownUnit`);
//! - a refused string with a pending second in it can carry `NumberOverflow` on one side and
//!   a later item's error on the other (above);
//! - two messages are 2.4.0's wording. `NumberOverflow` is "number is too large or cannot be
//!   represented without a lack of precision (values below 1ns are not supported)" (2.2.0:
//!   "number is too large"), and an unknown unit lists "ns, us/µs, ms, …" (2.2.0: "ns, us,
//!   ms, …"). These show in clap's and anyhow's error output, not in the serde module's.
//!
//! **Where humantime 2.2.0 panicked, this returns an error.**
//!
//! - `parse_duration("18446744073709551615s 1000000000ns")`, and any string that ends at
//!   `u64::MAX` seconds with a pending second, panicked in `Duration::new`. Here it is
//!   `NumberOverflow`.
//! - `format_rfc3339*` panicked on a time before 1970, and from year 10000 on its `Display`
//!   returned an error, which panics in `to_string`. Here the three functions return
//!   `Result<Rfc3339Timestamp, TimestampError>`, with [`TimestampError::OutOfRange`] for
//!   both, and the value they return can't fail to display. That is the one signature
//!   change: a call site adds `?` or maps the error. [`rfc3339_millis`] turns it into a
//!   serde error.
//!
//! **Not here.** `humantime::Timestamp`, `parse_rfc3339_weak`, `format_rfc3339_micros` and
//! `format_rfc3339_nanos` (the fork uses none of them), and humantime-serde's `SystemTime`
//! and `option` support (the fork's fields are all `Duration` or `Option<Duration>`).
//! humantime limited 32-bit targets to 2038; the range here is the same on every target.

mod format;
mod rfc3339;
pub mod serde;

use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

pub use crate::duration::{DurationError, parse_duration};
pub use format::{FormattedDuration, format_duration};
pub use rfc3339::{
    Rfc3339Timestamp, TimestampError, format_rfc3339, format_rfc3339_millis,
    format_rfc3339_seconds, parse_rfc3339, rfc3339_millis,
};

/// A `std::time::Duration` with `FromStr` ([`parse_duration`]) and `Display`
/// ([`format_duration`]), for clap arguments and wherever `humantime::Duration` was used.
///
/// ```
/// use chelabase_time::human;
///
/// let d: human::Duration = "1h 30m".parse().unwrap();
/// assert_eq!(d.as_secs(), 5400); // derefs to std's Duration
/// assert_eq!(d.to_string(), "1h 30m");
/// let std: std::time::Duration = d.into();
/// assert_eq!(human::Duration::from(std), d);
/// assert!("5parsecs".parse::<human::Duration>().is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Duration(std::time::Duration);

impl AsRef<std::time::Duration> for Duration {
    fn as_ref(&self) -> &std::time::Duration {
        &self.0
    }
}

impl Deref for Duration {
    type Target = std::time::Duration;

    fn deref(&self) -> &std::time::Duration {
        &self.0
    }
}

impl From<Duration> for std::time::Duration {
    fn from(d: Duration) -> Self {
        d.0
    }
}

impl From<std::time::Duration> for Duration {
    fn from(d: std::time::Duration) -> Self {
        Duration(d)
    }
}

impl FromStr for Duration {
    type Err = DurationError;

    fn from_str(s: &str) -> Result<Self, DurationError> {
        parse_duration(s).map(Duration)
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_duration(self.0).fmt(f)
    }
}
