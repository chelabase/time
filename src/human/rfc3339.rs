//! `SystemTime` as RFC 3339 in UTC: ports of humantime 2.2.0's formatter and strict parser.
//!
//! These don't go through [`Timestamp`](crate::Timestamp): humantime's range runs to the end
//! of year 9999, past `Timestamp::MAX` (`9999-12-30T22:00:00.999999999Z`).

use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// The last second these functions format and parse: `9999-12-31T23:59:59Z`.
const MAX_SECONDS: u64 = 253_402_300_800 - 1;

/// Why [`parse_rfc3339`] refused a string, or a `format_rfc3339*` function a time. The names
/// and the `Display` text are humantime's (`humantime::TimestampError`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimestampError {
    /// A field is out of range: a year before 1970, month 13, hour 24, second 61. From a
    /// `format_rfc3339*` function: the time is before 1970 or after 9999.
    OutOfRange,
    /// A character that isn't a digit where one is expected.
    InvalidDigit,
    /// Any other shape error: too short, no `T`, no final `Z`, an offset.
    InvalidFormat,
}

impl std::error::Error for TimestampError {}

impl fmt::Display for TimestampError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::OutOfRange => "numeric component is out of range",
            Self::InvalidDigit => "bad character where digit is expected",
            Self::InvalidFormat => "timestamp format is invalid",
        })
    }
}

/// Two ASCII digits as a number.
fn two_digits(b1: u8, b2: u8) -> Result<u64, TimestampError> {
    let digit = |b: u8| {
        char::from(b)
            .to_digit(10)
            .ok_or(TimestampError::InvalidDigit)
    };
    Ok(u64::from(digit(b1)? * 10 + digit(b2)?))
}

/// Parses an RFC 3339 timestamp in UTC, `2018-02-14T00:28:07Z`, as humantime 2.2.0's
/// `parse_rfc3339` did.
///
/// - The shape is fixed: four-digit year, `T`, and a final `Z`. An offset (`+01:00`, even
///   `+00:00`), a lower-case `t` or `z`, or a space for the `T` is an error.
/// - A fraction of any length may follow the seconds; digits past the ninth are dropped
///   (`.Z`, with no digit, is accepted too).
/// - The range is 1970-01-01 to 9999-12-31, wider at the top than
///   [`Timestamp`](crate::Timestamp)'s. A year before 1970 is [`TimestampError::OutOfRange`].
/// - Second `60` is accepted in any minute and read as second `59`
///   (`2016-12-31T23:59:60Z` is `2016-12-31T23:59:59Z`).
///
/// It never panics.
///
/// ```
/// use std::time::{Duration, UNIX_EPOCH};
/// use chelabase_time::human::{TimestampError, parse_rfc3339};
///
/// let t = parse_rfc3339("2018-02-14T00:28:07.133Z").unwrap();
/// assert_eq!(t, UNIX_EPOCH + Duration::new(1_518_568_087, 133_000_000));
/// assert!(parse_rfc3339("9999-12-31T23:59:60Z").is_ok());
/// assert_eq!(parse_rfc3339("2018-02-14T00:28:07+01:00"), Err(TimestampError::InvalidFormat));
/// assert_eq!(parse_rfc3339("1969-12-31T23:59:59Z"), Err(TimestampError::OutOfRange));
/// ```
pub fn parse_rfc3339(s: &str) -> Result<SystemTime, TimestampError> {
    if s.len() < "2018-02-14T00:28:07Z".len() {
        return Err(TimestampError::InvalidFormat);
    }
    // Bytes throughout, never `str` slices, so a multi-byte character can't panic.
    let b = s.as_bytes();
    if b[10] != b'T' || b.last() != Some(&b'Z') {
        return Err(TimestampError::InvalidFormat);
    }
    parse_weak(b)
}

/// humantime's `parse_rfc3339_weak`, which its strict parser ends in: at least 19 bytes, the
/// `T` may be a space and the `Z` may be missing ([`parse_rfc3339`] has refused both).
fn parse_weak(b: &[u8]) -> Result<SystemTime, TimestampError> {
    if b.len() < "2018-02-14T00:28:07".len() {
        return Err(TimestampError::InvalidFormat);
    }
    if b[4] != b'-'
        || b[7] != b'-'
        || (b[10] != b'T' && b[10] != b' ')
        || b[13] != b':'
        || b[16] != b':'
    {
        return Err(TimestampError::InvalidFormat);
    }
    let year = two_digits(b[0], b[1])? * 100 + two_digits(b[2], b[3])?;
    let month = two_digits(b[5], b[6])?;
    let day = two_digits(b[8], b[9])?;
    let hour = two_digits(b[11], b[12])?;
    let minute = two_digits(b[14], b[15])?;
    let mut second = two_digits(b[17], b[18])?;

    if year < 1970 || hour > 23 || minute > 59 || second > 60 {
        return Err(TimestampError::OutOfRange);
    }
    // A leap second, in any minute, is the second before it.
    if second == 60 {
        second = 59;
    }

    let leap = is_leap_year(year);
    let (mut ydays, mdays) = match month {
        1 => (0, 31),
        2 if leap => (31, 29),
        2 => (31, 28),
        3 => (59, 31),
        4 => (90, 30),
        5 => (120, 31),
        6 => (151, 30),
        7 => (181, 31),
        8 => (212, 31),
        9 => (243, 30),
        10 => (273, 31),
        11 => (304, 30),
        12 => (334, 31),
        _ => return Err(TimestampError::OutOfRange),
    };
    if day > mdays || day == 0 {
        return Err(TimestampError::OutOfRange);
    }
    ydays += day - 1;
    if leap && month > 2 {
        ydays += 1;
    }

    // `year` is 1970..=9999, so none of these underflow.
    let leap_years =
        ((year - 1) - 1968) / 4 - ((year - 1) - 1900) / 100 + ((year - 1) - 1600) / 400;
    let days = (year - 1970) * 365 + leap_years + ydays;

    let time = second + minute * 60 + hour * 3600;

    let mut nanos = 0u32;
    let mut mult = 100_000_000u32;
    if b.get(19) == Some(&b'.') {
        for (idx, &c) in b.iter().enumerate().skip(20) {
            if c == b'Z' {
                if idx == b.len() - 1 {
                    break;
                }
                return Err(TimestampError::InvalidDigit);
            }
            let digit = char::from(c)
                .to_digit(10)
                .ok_or(TimestampError::InvalidDigit)?;
            // `mult` is zero past the ninth digit, which drops the rest.
            nanos += mult * digit;
            mult /= 10;
        }
    } else if b.len() != 19 && (b.len() > 20 || b[19] != b'Z') {
        return Err(TimestampError::InvalidFormat);
    }

    let total_seconds = time + days * 86_400;
    if total_seconds > MAX_SECONDS {
        return Err(TimestampError::OutOfRange);
    }

    UNIX_EPOCH
        .checked_add(Duration::new(total_seconds, nanos))
        .ok_or(TimestampError::OutOfRange)
}

fn is_leap_year(y: u64) -> bool {
    y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Precision {
    /// No fraction for a whole second, nine digits otherwise.
    Smart,
    Seconds,
    Millis,
}

/// A time that displays as RFC 3339; made by [`format_rfc3339`], [`format_rfc3339_seconds`]
/// or [`format_rfc3339_millis`]. The text is built when it is made, so `Display` and
/// `to_string` can't fail.
#[derive(Debug, Clone)]
pub struct Rfc3339Timestamp {
    time: SystemTime,
    /// `0000-00-00T00:00:00.000000000Z`, of which the first `len` bytes are the text.
    buf: [u8; 30],
    len: usize,
}

impl Rfc3339Timestamp {
    /// The time being formatted.
    pub fn get_ref(&self) -> &SystemTime {
        &self.time
    }

    /// The formatted text.
    pub fn as_str(&self) -> &str {
        // Only ASCII digits and punctuation are written to `buf`.
        std::str::from_utf8(&self.buf[..self.len]).unwrap_or_default()
    }
}

impl fmt::Display for Rfc3339Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Formats a time as RFC 3339 in UTC with humantime's "smart" precision: no fraction when the
/// time is a whole second (`2018-02-14T00:28:07Z`), all nine digits otherwise
/// (`2018-02-14T00:28:07.133000000Z`).
///
/// humantime 2.2.0 panicked on a time before 1970, and its `Display` failed (a panic in
/// `to_string`) from year 10000 on. Here that is the error [`TimestampError::OutOfRange`], which
/// is why this returns a `Result` where humantime returned the formatter itself.
///
/// ```
/// use std::time::{Duration, UNIX_EPOCH};
/// use chelabase_time::human::{TimestampError, format_rfc3339};
///
/// let t = UNIX_EPOCH + Duration::new(1_518_568_087, 133_000_000);
/// assert_eq!(format_rfc3339(t).unwrap().to_string(), "2018-02-14T00:28:07.133000000Z");
/// let before = UNIX_EPOCH - Duration::from_secs(1);
/// assert_eq!(format_rfc3339(before).map(|f| f.to_string()), Err(TimestampError::OutOfRange));
/// ```
pub fn format_rfc3339(system_time: SystemTime) -> Result<Rfc3339Timestamp, TimestampError> {
    format(system_time, Precision::Smart)
}

/// Formats a time as RFC 3339 in UTC without a fraction (`2018-02-14T00:28:07Z`); the
/// sub-second part is dropped. Errors as [`format_rfc3339`].
///
/// ```
/// use std::time::{Duration, UNIX_EPOCH};
/// use chelabase_time::human::format_rfc3339_seconds;
///
/// let t = UNIX_EPOCH + Duration::new(1_518_568_087, 999_999_999);
/// assert_eq!(format_rfc3339_seconds(t).unwrap().to_string(), "2018-02-14T00:28:07Z");
/// ```
pub fn format_rfc3339_seconds(system_time: SystemTime) -> Result<Rfc3339Timestamp, TimestampError> {
    format(system_time, Precision::Seconds)
}

/// Formats a time as RFC 3339 in UTC with exactly three fraction digits, truncated
/// (`2018-02-14T00:28:07.000Z`). Errors as [`format_rfc3339`].
///
/// ```
/// use std::time::{Duration, UNIX_EPOCH};
/// use chelabase_time::human::format_rfc3339_millis;
///
/// let t = UNIX_EPOCH + Duration::new(1_518_568_087, 133_999_999);
/// assert_eq!(format_rfc3339_millis(t).unwrap().to_string(), "2018-02-14T00:28:07.133Z");
/// ```
pub fn format_rfc3339_millis(system_time: SystemTime) -> Result<Rfc3339Timestamp, TimestampError> {
    format(system_time, Precision::Millis)
}

/// humantime 2.2.0's `Display for Rfc3339Timestamp`, run when the value is made.
fn format(time: SystemTime, precision: Precision) -> Result<Rfc3339Timestamp, TimestampError> {
    let dur = time
        .duration_since(UNIX_EPOCH)
        .map_err(|_| TimestampError::OutOfRange)?;
    let secs_since_epoch = dur.as_secs();
    let nanos = dur.subsec_nanos();

    if secs_since_epoch > MAX_SECONDS {
        return Err(TimestampError::OutOfRange);
    }

    // 2000-03-01, the day after a 400-year cycle's leap day.
    const LEAPOCH: i64 = 11017;
    const DAYS_PER_400Y: i64 = 365 * 400 + 97;
    const DAYS_PER_100Y: i64 = 365 * 100 + 24;
    const DAYS_PER_4Y: i64 = 365 * 4 + 1;

    // Below 2^38, so the cast is exact.
    let days = (secs_since_epoch / 86_400) as i64 - LEAPOCH;
    let secs_of_day = secs_since_epoch % 86_400;

    let mut qc_cycles = days / DAYS_PER_400Y;
    let mut remdays = days % DAYS_PER_400Y;

    if remdays < 0 {
        remdays += DAYS_PER_400Y;
        qc_cycles -= 1;
    }

    let mut c_cycles = remdays / DAYS_PER_100Y;
    if c_cycles == 4 {
        c_cycles -= 1;
    }
    remdays -= c_cycles * DAYS_PER_100Y;

    let mut q_cycles = remdays / DAYS_PER_4Y;
    if q_cycles == 25 {
        q_cycles -= 1;
    }
    remdays -= q_cycles * DAYS_PER_4Y;

    let mut remyears = remdays / 365;
    if remyears == 4 {
        remyears -= 1;
    }
    remdays -= remyears * 365;

    let mut year = 2000 + remyears + 4 * q_cycles + 100 * c_cycles + 400 * qc_cycles;

    // March first: the year above starts on 1 March.
    let months = [31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 29];
    let mut mon = 0;
    for mon_len in months {
        mon += 1;
        if remdays < mon_len {
            break;
        }
        remdays -= mon_len;
    }
    let mday = remdays + 1;
    let mon = if mon + 2 > 12 {
        year += 1;
        mon - 10
    } else {
        mon + 2
    };

    // Every value below is a single decimal digit, so the casts can't truncate.
    let mut buf: [u8; 30] = *b"0000-00-00T00:00:00.000000000Z";
    buf[0] = b'0' + (year / 1000) as u8;
    buf[1] = b'0' + (year / 100 % 10) as u8;
    buf[2] = b'0' + (year / 10 % 10) as u8;
    buf[3] = b'0' + (year % 10) as u8;
    buf[5] = b'0' + (mon / 10) as u8;
    buf[6] = b'0' + (mon % 10) as u8;
    buf[8] = b'0' + (mday / 10) as u8;
    buf[9] = b'0' + (mday % 10) as u8;
    buf[11] = b'0' + (secs_of_day / 3600 / 10) as u8;
    buf[12] = b'0' + (secs_of_day / 3600 % 10) as u8;
    buf[14] = b'0' + (secs_of_day / 60 / 10 % 6) as u8;
    buf[15] = b'0' + (secs_of_day / 60 % 10) as u8;
    buf[17] = b'0' + (secs_of_day / 10 % 6) as u8;
    buf[18] = b'0' + (secs_of_day % 10) as u8;

    // The index of the final `Z`.
    let offset = if precision == Precision::Seconds || nanos == 0 && precision == Precision::Smart {
        buf[19] = b'Z';
        19
    } else if precision == Precision::Millis {
        buf[20] = b'0' + (nanos / 100_000_000) as u8;
        buf[21] = b'0' + (nanos / 10_000_000 % 10) as u8;
        buf[22] = b'0' + (nanos / 1_000_000 % 10) as u8;
        buf[23] = b'Z';
        23
    } else {
        buf[20] = b'0' + (nanos / 100_000_000) as u8;
        buf[21] = b'0' + (nanos / 10_000_000 % 10) as u8;
        buf[22] = b'0' + (nanos / 1_000_000 % 10) as u8;
        buf[23] = b'0' + (nanos / 100_000 % 10) as u8;
        buf[24] = b'0' + (nanos / 10_000 % 10) as u8;
        buf[25] = b'0' + (nanos / 1_000 % 10) as u8;
        buf[26] = b'0' + (nanos / 100 % 10) as u8;
        buf[27] = b'0' + (nanos / 10 % 10) as u8;
        buf[28] = b'0' + (nanos % 10) as u8;
        // The 29th is already `Z`.
        29
    };

    Ok(Rfc3339Timestamp {
        time,
        buf,
        len: offset + 1,
    })
}

/// `#[serde(with = "chelabase_time::human::rfc3339_millis")]` for a `SystemTime` field: written
/// with [`format_rfc3339_millis`] and read with [`parse_rfc3339`], as the fork's
/// `SystemTimeAsRfc3339Millis` and `utils::serde_system_time` did with humantime.
///
/// A time before 1970 or after 9999 is a serialization error (humantime panicked there), and a
/// string [`parse_rfc3339`] refuses is a deserialization error with its message.
///
/// ```
/// use std::time::{Duration, SystemTime, UNIX_EPOCH};
///
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct Lease {
///     #[serde(with = "chelabase_time::human::rfc3339_millis")]
///     valid_until: SystemTime,
/// }
///
/// let lease = Lease { valid_until: UNIX_EPOCH + Duration::from_millis(1_518_568_087_133) };
/// let json = serde_json::to_string(&lease).unwrap();
/// assert_eq!(json, r#"{"valid_until":"2018-02-14T00:28:07.133Z"}"#);
/// let back: Lease = serde_json::from_str(&json).unwrap();
/// assert_eq!(back.valid_until, lease.valid_until);
///
/// let before = Lease { valid_until: UNIX_EPOCH - Duration::from_secs(1) };
/// assert!(serde_json::to_string(&before).is_err());
/// ```
pub mod rfc3339_millis {
    use std::time::SystemTime;

    use ::serde::{Deserialize, Deserializer, Serializer, de, ser};

    use super::{format_rfc3339_millis, parse_rfc3339};

    /// Writes the time as `2018-02-14T00:28:07.133Z`.
    pub fn serialize<S: Serializer>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error> {
        let text = format_rfc3339_millis(*time).map_err(ser::Error::custom)?;
        serializer.serialize_str(text.as_str())
    }

    /// Reads a string with [`parse_rfc3339`].
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<SystemTime, D::Error> {
        let s = String::deserialize(deserializer)?;
        parse_rfc3339(&s).map_err(de::Error::custom)
    }
}
