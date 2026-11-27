//! serde for `std::time::Duration` fields: `#[serde(with = "chelabase_time::human::serde::required")]`
//! and, for `Option<Duration>`, `…::optional`.
//!
//! Writes jiff's friendly compact format (jiff's
//! `unsigned_duration::friendly::compact` helpers do the writing); reads with
//! [`super::parse`], so humantime's spellings (`7days`) still read. `None` is written and read
//! as jiff's `optional` helper does (`null` in JSON, an absent key in TOML, which needs
//! `#[serde(default)]` on the field).

use std::fmt;
use std::time::Duration;

use serde::de::{self, Deserializer, Visitor};

struct DurationVisitor;

impl Visitor<'_> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a duration string such as \"1h 30m\"")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Duration, E> {
        super::parse(value).map_err(E::custom)
    }

    fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Duration, E> {
        let value = std::str::from_utf8(value)
            .map_err(|_| E::invalid_value(de::Unexpected::Bytes(value), &self))?;
        self.visit_str(value)
    }
}

struct OptionalVisitor;

impl<'de> Visitor<'de> for OptionalVisitor {
    type Value = Option<Duration>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a duration string such as \"1h 30m\", or nothing")
    }

    fn visit_some<D: Deserializer<'de>>(self, de: D) -> Result<Option<Duration>, D::Error> {
        de.deserialize_str(DurationVisitor).map(Some)
    }

    fn visit_none<E: de::Error>(self) -> Result<Option<Duration>, E> {
        Ok(None)
    }

    fn visit_unit<E: de::Error>(self) -> Result<Option<Duration>, E> {
        Ok(None)
    }
}

/// For a `std::time::Duration` field.
pub mod required {
    use std::time::Duration;

    use serde::{Deserializer, Serializer};

    /// Writes jiff's friendly compact format (`1h 30m`).
    pub fn serialize<S: Serializer>(duration: &Duration, se: S) -> Result<S::Ok, S::Error> {
        jiff::fmt::serde::unsigned_duration::friendly::compact::required::serialize(duration, se)
    }

    /// Reads with [`crate::human::parse`].
    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<Duration, D::Error> {
        de.deserialize_str(super::DurationVisitor)
    }
}

/// For an `Option<std::time::Duration>` field.
pub mod optional {
    use std::time::Duration;

    use serde::{Deserializer, Serializer};

    /// Writes jiff's friendly compact format, or none.
    pub fn serialize<S: Serializer>(duration: &Option<Duration>, se: S) -> Result<S::Ok, S::Error> {
        jiff::fmt::serde::unsigned_duration::friendly::compact::optional::serialize(duration, se)
    }

    /// Reads with [`crate::human::parse`], or none.
    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<Option<Duration>, D::Error> {
        de.deserialize_option(super::OptionalVisitor)
    }
}
