//! `SystemTime` as RFC 3339 in UTC, through `jiff::Timestamp`, in place of humantime's
//! `format_rfc3339*` and `parse_rfc3339`.
//!
//! Output is jiff's spelling with `Z`: [`format()`] writes as many fraction digits as the value
//! needs (none, `.5`, up to 9), [`format_millis`] exactly 3 and [`format_seconds`] none. Both
//! drop digits toward the past (`1969-12-31T23:59:59.9995Z` is `…59.999Z` in millis), never
//! round. [`parse`] reads what jiff and humantime write (`Z` with or without a fraction) and
//! any other offset, converted to UTC.
//!
//! The range is jiff's, `-009999-01-02T01:59:59Z` to `9999-12-30T22:00:00.999999999Z`: a
//! `SystemTime` outside it is an error from the `format*` functions, and a string outside it
//! (or malformed) is an error from [`parse`]. Nothing panics or clamps.

use std::time::SystemTime;

use jiff::Timestamp;

/// RFC 3339 with `Z` and as many fraction digits as the value needs: `2018-02-14T00:28:07Z`,
/// `2018-02-14T00:28:07.5Z`. [`parse`] returns `t` exactly.
///
/// ```
/// use std::time::{Duration, UNIX_EPOCH};
/// use chelabase_time::rfc3339;
///
/// let t = UNIX_EPOCH + Duration::from_millis(1_518_568_087_500);
/// assert_eq!(rfc3339::format(t).unwrap(), "2018-02-14T00:28:07.5Z");
/// assert_eq!(rfc3339::parse("2018-02-14T00:28:07.5Z").unwrap(), t);
/// ```
pub fn format(t: SystemTime) -> Result<String, jiff::Error> {
    Ok(Timestamp::try_from(t)?.to_string())
}

/// RFC 3339 with `Z` and exactly 3 fraction digits, truncated: `2018-02-14T00:28:07.500Z`.
///
/// ```
/// use std::time::{Duration, UNIX_EPOCH};
/// use chelabase_time::rfc3339;
///
/// let t = UNIX_EPOCH + Duration::from_micros(1_518_568_087_123_999);
/// assert_eq!(rfc3339::format_millis(t).unwrap(), "2018-02-14T00:28:07.123Z");
/// ```
pub fn format_millis(t: SystemTime) -> Result<String, jiff::Error> {
    Ok(format!("{:.3}", Timestamp::try_from(t)?))
}

/// RFC 3339 with `Z` and no fraction, truncated: `2018-02-14T00:28:07Z`.
///
/// ```
/// use std::time::{Duration, UNIX_EPOCH};
/// use chelabase_time::rfc3339;
///
/// let t = UNIX_EPOCH + Duration::from_millis(1_518_568_087_999);
/// assert_eq!(rfc3339::format_seconds(t).unwrap(), "2018-02-14T00:28:07Z");
/// ```
pub fn format_seconds(t: SystemTime) -> Result<String, jiff::Error> {
    Ok(format!("{:.0}", Timestamp::try_from(t)?))
}

/// Reads an RFC 3339 timestamp with `Z` or an offset (jiff's parser). A string without an
/// offset, a malformed one or one outside jiff's range is an error.
///
/// ```
/// use std::time::{Duration, UNIX_EPOCH};
/// use chelabase_time::rfc3339;
///
/// let t = UNIX_EPOCH + Duration::from_secs(1_518_568_087);
/// assert_eq!(rfc3339::parse("2018-02-14T00:28:07Z").unwrap(), t);
/// assert_eq!(rfc3339::parse("2018-02-14T02:28:07+02:00").unwrap(), t);
/// assert!(rfc3339::parse("2018-02-14T00:28:07").is_err());
/// ```
pub fn parse(s: &str) -> Result<SystemTime, jiff::Error> {
    SystemTime::try_from(s.parse::<Timestamp>()?)
}
