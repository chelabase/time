//! `#[serde(with = "chelabase_time::human::serde")]` for a `Duration` or an `Option<Duration>`
//! field: the text humantime-serde 1.1.1 wrote and read.
//!
//! ```
//! use std::time::Duration;
//!
//! #[derive(serde::Serialize, serde::Deserialize)]
//! struct Config {
//!     #[serde(with = "chelabase_time::human::serde")]
//!     pitr_interval: Duration,
//!     #[serde(default, with = "chelabase_time::human::serde")]
//!     lsn_lease_length: Option<Duration>,
//! }
//!
//! let config = Config {
//!     pitr_interval: Duration::from_secs(7 * 86_400),
//!     lsn_lease_length: None,
//! };
//! let json = serde_json::to_string(&config).unwrap();
//! assert_eq!(json, r#"{"pitr_interval":"7days","lsn_lease_length":null}"#);
//! let back: Config = serde_json::from_str(r#"{"pitr_interval":"1h 30m"}"#).unwrap();
//! assert_eq!(back.pitr_interval, Duration::from_secs(5400));
//! assert_eq!(back.lsn_lease_length, None);
//! ```
//!
//! - A `Duration` is written as the string [`format_duration`] gives and read with
//!   [`parse_duration`] from a string only. A string that doesn't parse is the error
//!   `invalid value: string "…", expected a duration`; a number, `null` or anything else is
//!   `invalid type: …, expected a duration`.
//! - `Some(d)` is written and read as `d`. `None` is written as the format's none (`null` in
//!   JSON; TOML leaves the key out) and read from it. A missing key is an error unless the field
//!   also has `#[serde(default)]`, as with humantime-serde, so a `None` written to TOML reads
//!   back only into a field with `default`.
//!
//! [`deserialize`] alone also works as `deserialize_with`.
//!
//! [`format_duration`]: crate::human::format_duration
//! [`parse_duration`]: crate::human::parse_duration

use std::fmt;
use std::time::Duration;

use ::serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use super::{format_duration, parse_duration};

mod sealed {
    pub trait Sealed {}
}

/// The field types the module serves: `Duration` and `Option<Duration>`.
pub trait HumanDuration: sealed::Sealed + Sized {
    #[doc(hidden)]
    fn serialize_human<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
    #[doc(hidden)]
    fn deserialize_human<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>;
}

/// Writes a `Duration` or an `Option<Duration>` in humantime's form (see the module docs).
pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: HumanDuration,
    S: Serializer,
{
    value.serialize_human(serializer)
}

/// Reads a `Duration` or an `Option<Duration>` from humantime's form (see the module docs).
pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: HumanDuration,
    D: Deserializer<'de>,
{
    T::deserialize_human(deserializer)
}

/// A duration as its humantime string, for the value inside `Some`.
struct Human(Duration);

impl Serialize for Human {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format_duration(self.0).to_string())
    }
}

impl<'de> Deserialize<'de> for Human {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;

        impl de::Visitor<'_> for V {
            type Value = Duration;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a duration")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Duration, E> {
                parse_duration(v).map_err(|_| E::invalid_value(de::Unexpected::Str(v), &self))
            }
        }

        deserializer.deserialize_str(V).map(Human)
    }
}

impl sealed::Sealed for Duration {}

impl HumanDuration for Duration {
    fn serialize_human<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Human(*self).serialize(serializer)
    }

    fn deserialize_human<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Human::deserialize(deserializer).map(|h| h.0)
    }
}

impl sealed::Sealed for Option<Duration> {}

impl HumanDuration for Option<Duration> {
    fn serialize_human<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            Some(d) => serializer.serialize_some(&Human(d)),
            None => serializer.serialize_none(),
        }
    }

    fn deserialize_human<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<Human>::deserialize(deserializer).map(|h| h.map(|h| h.0))
    }
}
