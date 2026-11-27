//! `format_duration`: a port of humantime 2.2.0's.

use std::fmt;
use std::time::Duration;

/// Formats a duration as humantime 2.2.0 did: `1h 30m`, `7days`, `1month`, `0s`.
///
/// The parts are years (365.25 days), months (30.44 days), days, `h`, `m`, `s`, `ms`, `us` and
/// `ns`, largest first, separated by one space; a part that is zero is left out, and the zero
/// duration is `0s`. Years, months and days take a plural `s`. [`parse_duration`] reads every
/// string this writes back to the same duration.
///
/// ```
/// use std::time::Duration;
/// use chelabase_time::human::format_duration;
///
/// assert_eq!(format_duration(Duration::from_secs(5400)).to_string(), "1h 30m");
/// assert_eq!(format_duration(Duration::from_secs(7 * 86_400)).to_string(), "7days");
/// assert_eq!(format_duration(Duration::from_millis(1500)).to_string(), "1s 500ms");
/// assert_eq!(format_duration(Duration::ZERO).to_string(), "0s");
/// ```
///
/// [`parse_duration`]: crate::human::parse_duration
pub fn format_duration(val: Duration) -> FormattedDuration {
    FormattedDuration(val)
}

/// A duration that displays in humantime's form; see [`format_duration`].
#[derive(Debug, Clone)]
pub struct FormattedDuration(Duration);

impl FormattedDuration {
    /// The duration being formatted.
    pub fn get_ref(&self) -> &Duration {
        &self.0
    }
}

/// `<value><name>`, with an `s` when the value is above one (`2days`).
fn item_plural(
    f: &mut fmt::Formatter<'_>,
    started: &mut bool,
    name: &str,
    value: u64,
) -> fmt::Result {
    if value > 0 {
        if *started {
            f.write_str(" ")?;
        }
        write!(f, "{value}{name}")?;
        if value > 1 {
            f.write_str("s")?;
        }
        *started = true;
    }
    Ok(())
}

/// `<value><name>` (`30m`).
fn item(f: &mut fmt::Formatter<'_>, started: &mut bool, name: &str, value: u32) -> fmt::Result {
    if value > 0 {
        if *started {
            f.write_str(" ")?;
        }
        write!(f, "{value}{name}")?;
        *started = true;
    }
    Ok(())
}

impl fmt::Display for FormattedDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let secs = self.0.as_secs();
        let nanos = self.0.subsec_nanos();

        if secs == 0 && nanos == 0 {
            return f.write_str("0s");
        }

        let years = secs / 31_557_600; // 365.25 days
        let ydays = secs % 31_557_600;
        let months = ydays / 2_630_016; // 30.44 days
        let mdays = ydays % 2_630_016;
        let days = mdays / 86_400;
        let day_secs = mdays % 86_400;
        let hours = day_secs / 3600;
        let minutes = day_secs % 3600 / 60;
        let seconds = day_secs % 60;

        let millis = nanos / 1_000_000;
        let micros = nanos / 1000 % 1000;
        let nanosec = nanos % 1000;

        let started = &mut false;
        item_plural(f, started, "year", years)?;
        item_plural(f, started, "month", months)?;
        item_plural(f, started, "day", days)?;
        // Below a day, so these fit a `u32`.
        item(f, started, "h", hours as u32)?;
        item(f, started, "m", minutes as u32)?;
        item(f, started, "s", seconds as u32)?;
        item(f, started, "ms", millis)?;
        item(f, started, "us", micros)?;
        item(f, started, "ns", nanosec)?;
        Ok(())
    }
}
