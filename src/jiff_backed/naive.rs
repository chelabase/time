//! [`NaiveDateTime`], a civil date-time over jiff that writes and reads chrono 0.4.45's
//! `NaiveDateTime` formats. The contract is in the [`crate::civil`] module docs.

use std::fmt;
use std::str::FromStr;

use jiff::tz::Offset;

use super::{Timestamp, beyond_range, kind_error, parse, write_auto_fraction, write_year};
use crate::{ParseError, SignedDuration, TimestampExt};

/// `1970-01-01T00:00:00`, the civil date-time the Unix epoch is in UTC.
const EPOCH: jiff::civil::DateTime = jiff::civil::DateTime::constant(1970, 1, 1, 0, 0, 0, 0);

/// A date and time of day without a time zone or offset, nanosecond precision, backed by
/// [`jiff::civil::DateTime`]. It stands where chrono's `NaiveDateTime` stood: its serde form,
/// `FromStr`, `Display` and `Debug` are chrono 0.4.45's (see [`crate::civil`]).
///
/// Its range is jiff's civil range, [`NaiveDateTime::MIN`] (`-9999-01-01T00:00:00`) to
/// [`NaiveDateTime::MAX`] (`9999-12-31T23:59:59.999999999`). That is wider than
/// [`Timestamp`]'s at both ends, so the conversion to a `Timestamp` is a `TryFrom`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NaiveDateTime(jiff::civil::DateTime);

/// A [`NaiveDateTime`] that, read as UTC, lies outside [`Timestamp`]'s range (before
/// [`Timestamp::MIN`] or after [`Timestamp::MAX`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutOfTimestampRange;

impl fmt::Display for OutOfTimestampRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the date and time is outside the range of Timestamp")
    }
}

impl std::error::Error for OutOfTimestampRange {}

impl NaiveDateTime {
    /// The earliest value the type holds, `-9999-01-01T00:00:00`.
    pub const MIN: NaiveDateTime = NaiveDateTime(jiff::civil::DateTime::MIN);

    /// The latest value the type holds, `9999-12-31T23:59:59.999999999`.
    pub const MAX: NaiveDateTime = NaiveDateTime(jiff::civil::DateTime::MAX);

    /// `year-month-day hour:minute:second` and `nano` nanoseconds; `None` if the date or time
    /// doesn't exist or is outside the range. A `second` of 60 is `None`.
    ///
    /// ```
    /// use chelabase_time::civil::NaiveDateTime;
    /// let t = NaiveDateTime::from_ymd_hms_nano(2025, 3, 7, 11, 7, 11, 373_105_434).unwrap();
    /// assert_eq!(format!("{t:?}"), "2025-03-07T11:07:11.373105434");
    /// assert!(NaiveDateTime::from_ymd_hms_nano(2025, 2, 29, 0, 0, 0, 0).is_none());
    /// ```
    pub fn from_ymd_hms_nano(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        nano: u32,
    ) -> Option<Self> {
        jiff::civil::DateTime::new(
            i16::try_from(year).ok()?,
            i8::try_from(month).ok()?,
            i8::try_from(day).ok()?,
            i8::try_from(hour).ok()?,
            i8::try_from(minute).ok()?,
            i8::try_from(second).ok()?,
            i32::try_from(nano).ok()?,
        )
        .ok()
        .map(NaiveDateTime)
    }

    /// The current time in UTC as a civil date-time (chrono's `Utc::now().naive_utc()`).
    ///
    /// ```
    /// use chelabase_time::civil::NaiveDateTime;
    /// use chelabase_time::{Timestamp, TimestampExt};
    /// let now = Timestamp::try_from(NaiveDateTime::now_utc()).unwrap();
    /// assert!(now <= Timestamp::now());
    /// ```
    pub fn now_utc() -> Self {
        Timestamp::now().into()
    }

    /// Whole milliseconds since `1970-01-01T00:00:00`, reading the value as UTC and rounding
    /// toward the past: chrono's `and_utc().timestamp_millis()` (and the method `parquet_derive`
    /// calls for a `TIMESTAMP_MILLIS` column). It never fails: the whole range fits.
    ///
    /// ```
    /// use chelabase_time::civil::NaiveDateTime;
    /// let t: NaiveDateTime = "1969-12-31T23:59:59.9995".parse().unwrap();
    /// assert_eq!(t.timestamp_millis(), -1);
    /// ```
    pub fn timestamp_millis(&self) -> i64 {
        // At most ±3.8e14, and the nanosecond is never negative, so this is a floor.
        self.unix_seconds() * 1_000 + i64::from(self.0.subsec_nanosecond() / 1_000_000)
    }

    /// The jiff value underneath.
    pub fn to_jiff(self) -> jiff::civil::DateTime {
        self.0
    }

    /// Wraps a jiff value.
    pub fn from_jiff(dt: jiff::civil::DateTime) -> Self {
        NaiveDateTime(dt)
    }

    /// Whole seconds from `1970-01-01T00:00:00` to this value's second, from the civil fields
    /// (not through a `Timestamp`, whose range is narrower).
    fn unix_seconds(&self) -> i64 {
        let date = parse::Ymd {
            year: i64::from(self.0.year()),
            // jiff's months, days and times of day are positive.
            month: self.0.month() as u32,
            day: self.0.day() as u32,
        };
        date.unix_days() * 86_400
            + i64::from(self.0.hour()) * 3_600
            + i64::from(self.0.minute()) * 60
            + i64::from(self.0.second())
    }

    /// chrono's fields as a value. chrono holds years to ±262,143 and a second of 60; the range
    /// check here is the type's one bound, jiff's civil range, applied after the leap second is
    /// carried into the next second.
    fn from_fields(fields: parse::NaiveFields) -> Result<Self, ParseError> {
        let seconds = fields.date.unix_days() * 86_400
            + i64::from(fields.hour) * 3_600
            + i64::from(fields.minute) * 60
            + i64::from(fields.second);
        // `nano` is below 1_000_000_000.
        let since_epoch = SignedDuration::new(seconds, fields.nano as i32);
        EPOCH
            .checked_add(since_epoch)
            .map(NaiveDateTime)
            .map_err(|_| beyond_range())
    }

    /// chrono's `NaiveDate` and `NaiveTime` `Debug`, joined by `sep`.
    fn write(&self, f: &mut fmt::Formatter<'_>, sep: char) -> fmt::Result {
        let dt = self.0;
        write_year(f, dt.year())?;
        write!(
            f,
            "-{:02}-{:02}{sep}{:02}:{:02}:{:02}",
            dt.month(),
            dt.day(),
            dt.hour(),
            dt.minute(),
            dt.second()
        )?;
        // A civil time's fraction is never negative.
        write_auto_fraction(f, dt.subsec_nanosecond() as u32)
    }
}

/// The UTC date and time of an instant (chrono's `naive_utc`). Every instant has one.
impl From<Timestamp> for NaiveDateTime {
    fn from(t: Timestamp) -> Self {
        NaiveDateTime(Offset::UTC.to_datetime(t.to_jiff()))
    }
}

/// The instant a civil date-time is in UTC (chrono's `and_utc`). It fails for the values
/// [`Timestamp`] can't hold: before `-9999-01-02T01:59:59` and after
/// `9999-12-30T22:00:00.999999999`.
impl TryFrom<NaiveDateTime> for Timestamp {
    type Error = OutOfTimestampRange;

    fn try_from(dt: NaiveDateTime) -> Result<Timestamp, OutOfTimestampRange> {
        // Through nanoseconds, as the `Timestamp` constructors go, so the range's last second
        // is accepted with its fraction.
        let nanos =
            i128::from(dt.unix_seconds()) * 1_000_000_000 + i128::from(dt.0.subsec_nanosecond());
        Timestamp::from_nanos(nanos).ok_or(OutOfTimestampRange)
    }
}

/// `2025-03-07 11:07:11.373105434`: chrono's `Display`, a space between date and time and the
/// fraction as 0, 3, 6 or 9 digits.
impl fmt::Display for NaiveDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f, ' ')
    }
}

/// `2025-03-07T11:07:11.373105434`: chrono's `Debug`, which is also the serde form.
impl fmt::Debug for NaiveDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f, 'T')
    }
}

/// chrono's `NaiveDateTime` `FromStr`; see [`crate::civil`] for what it accepts.
impl FromStr for NaiveDateTime {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        Self::from_fields(parse::naive_datetime(s).map_err(kind_error)?)
    }
}

/// The `Debug` form as a string, as chrono writes it.
impl serde::Serialize for NaiveDateTime {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&format_args!("{self:?}"))
    }
}

impl<'de> serde::Deserialize<'de> for NaiveDateTime {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = NaiveDateTime;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a formatted date and time string")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<NaiveDateTime, E> {
                value.parse().map_err(E::custom)
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}
