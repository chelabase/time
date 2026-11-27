//! Human-readable durations: written in jiff's friendly compact format (`1h 30m`, `168h`,
//! `1s 500ms`, `3µs`), read with jiff and, for spellings jiff refuses, with humantime's grammar
//! ([`crate::parse_duration`]), so values stored by humantime still read.
//!
//! - [`serde::required`] and [`serde::optional`] go on `std::time::Duration` and
//!   `Option<std::time::Duration>` fields (`#[serde(with = "chelabase_time::human::serde::required")]`),
//!   in place of `humantime_serde`.
//! - [`Duration`] is the type for a clap argument, in place of `humantime::Duration`. Its
//!   `Display` replaces `humantime::format_duration`.
//! - [`parse`] is the reader both use.
//!
//! # Reading
//!
//! [`parse`] tries jiff first: its friendly format, or ISO 8601 when the string starts with
//! `P` (as jiff's own serde helper does). If jiff refuses the string, [`crate::parse_duration`]
//! reads it. Where both grammars accept a spelling, they give the same value
//! (`tests/human.rs` checks every unit alias and generated strings), so the order only decides
//! which error a bad string gets.
//!
//! | Spelling | jiff | `parse_duration` |
//! |---|---|---|
//! | `ns`, `us`, `µs`, `ms`, `s`, `m`, `h` and the long forms (`secs`, `mins`, `hours`, …) | reads | reads, same value |
//! | `1h30m`, `1h 30m`, `1.5s`, `0s` | reads | reads, same value |
//! | `PT1H30M` (ISO 8601), `2 hrs, 30 mins` | reads | refuses |
//! | `d`, `day`, `days`, `w`, `wk`, `weeks`, … | refuses | reads (24-hour days, 7-day weeks) |
//! | `M`, `month`, `months` | refuses | reads (30.44 days) |
//! | `y`, `yr`, `year`, `years` | refuses | reads (365.25 days) |
//! | `0` | refuses | reads (zero) |
//!
//! A negative duration (`-1s`) is refused by both.
//!
//! # Writing
//!
//! jiff's friendly compact format, as `jiff::fmt::serde::unsigned_duration::friendly::compact`
//! writes it: hours are the largest unit (7 days is `168h`), sub-second parts are written as
//! `ms`, `µs` and `ns`, and zero is `0s`. [`crate::parse_duration`] reads all of it back.

pub mod serde;

use std::fmt;
use std::str::FromStr;

use jiff::fmt::friendly::{Designator, SpanPrinter};

static PRINTER: SpanPrinter = SpanPrinter::new().designator(Designator::Compact);

/// Why [`parse`] refused a string: jiff's error, and humantime's grammar's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDurationError {
    jiff: String,
    humantime: crate::DurationError,
}

impl fmt::Display for ParseDurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (humantime's spellings were tried too: {})",
            self.jiff, self.humantime
        )
    }
}

impl std::error::Error for ParseDurationError {}

/// Reads a duration: jiff's friendly format or ISO 8601, else humantime's grammar (see the
/// module docs).
///
/// ```
/// use std::time::Duration;
/// use chelabase_time::human;
///
/// assert_eq!(human::parse("1h 30m").unwrap(), Duration::from_secs(5400));
/// assert_eq!(human::parse("PT1H30M").unwrap(), Duration::from_secs(5400));
/// assert_eq!(human::parse("7days").unwrap(), Duration::from_secs(7 * 86_400));
/// assert!(human::parse("5parsecs").is_err());
/// ```
pub fn parse(s: &str) -> Result<std::time::Duration, ParseDurationError> {
    use ::serde::de::value::{Error, StrDeserializer};
    // jiff's own serde reader: ISO 8601 when the string starts with `P`, friendly otherwise.
    let jiff = match jiff::fmt::serde::unsigned_duration::required::deserialize(StrDeserializer::<
        Error,
    >::new(s))
    {
        Ok(d) => return Ok(d),
        Err(e) => e,
    };
    // Spellings only humantime wrote (`7days`, `2weeks`, `1month`).
    crate::parse_duration(s).map_err(|humantime| ParseDurationError {
        jiff: jiff.to_string(),
        humantime,
    })
}

/// A duration for a clap argument (`--interval 1h30m`, `--retention 7days`): [`FromStr`] is
/// [`parse`], and `Display` writes jiff's friendly compact format.
///
/// ```
/// use chelabase_time::human;
///
/// let d: human::Duration = "90m".parse().unwrap();
/// assert_eq!(d.to_string(), "1h 30m");
/// assert_eq!(std::time::Duration::from(d).as_secs(), 5400);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Duration(std::time::Duration);

impl Duration {
    /// The `std::time::Duration`.
    pub const fn get(self) -> std::time::Duration {
        self.0
    }
}

impl FromStr for Duration {
    type Err = ParseDurationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse(s).map(Duration)
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&PRINTER.unsigned_duration_to_string(&self.0))
    }
}

impl std::ops::Deref for Duration {
    type Target = std::time::Duration;

    fn deref(&self) -> &std::time::Duration {
        &self.0
    }
}

impl From<std::time::Duration> for Duration {
    fn from(d: std::time::Duration) -> Self {
        Duration(d)
    }
}

impl From<Duration> for std::time::Duration {
    fn from(d: Duration) -> Self {
        d.0
    }
}
