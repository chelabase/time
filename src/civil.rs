//! Civil (time-zone-less) date-times in chrono's `NaiveDateTime` formats (epic neon-drop-chrono).
//!
//! [`NaiveDateTime`] is a date and a time of day with no offset, a newtype over
//! [`jiff::civil::DateTime`]. The Neon fork's pageserver stores it in S3 (`index_part.json`,
//! the tenant manifest and the import index hold `"2025-03-07T11:07:11.373105434"`), so what
//! it writes and reads is chrono 0.4.45's `NaiveDateTime`, byte for byte, errors included.
//! `tests/civil_goldens.rs` holds chrono's results as literals.
//!
//! # The contract
//!
//! - **serde** writes a string: `YYYY-MM-DDTHH:MM:SS` and a fraction of 0, 3, 6 or 9 digits, as
//!   few as hold the value (`…11.373`, `…11.373100`, `…11.373105434`), with no `Z` or offset.
//!   A year outside 0..=9999 is signed (`-0042-01-02T03:04:05`). It reads what `FromStr` reads,
//!   and only from a string (a number or `null` is serde's "invalid type … expected a formatted
//!   date and time string").
//! - **`Debug`** is the serde form. **`Display`** is the same with a space for the `T`
//!   (`2025-03-07 11:07:11.373105434`).
//! - **`FromStr`** is chrono's, which is more lenient than what it writes:
//!   - whitespace (any Unicode whitespace) is skipped before each number, before each `-`, the
//!     `T` and each `:`, and at the end; not before the fraction's `.`;
//!   - each number may have fewer digits (`2026-9-3T1:2:3`), and never more than its width
//!     (4 for the year, 2 for the rest);
//!   - a year with a sign may have any number of digits (`+02026`, `-1`); without one, at most
//!     four;
//!   - the fraction may have any number of digits; those past the ninth are dropped, not
//!     rounded;
//!   - the separator is an uppercase `T` only: `t` and a space are errors (chrono's RFC 3339
//!     parsers take both, its `NaiveDateTime` doesn't);
//!   - a `Z`, an offset or any other trailing character is an error.
//! - **Errors** are [`ParseError`](crate::ParseError) with chrono's messages: "input contains
//!   invalid characters", "premature end of input", "trailing input" and "input is out of
//!   range".
//! - `Copy`, `Eq`, `Ord` (calendar order), `Hash`.
//! - [`NaiveDateTime::timestamp_millis`] is chrono's `and_utc().timestamp_millis()`.
//!   `NaiveDateTime::from(timestamp)` is chrono's `naive_utc()`, `Timestamp::try_from(civil)`
//!   its `and_utc()`, and [`NaiveDateTime::now_utc`] its `Utc::now().naive_utc()`.
//!
//! ## Differences from chrono
//!
//! - **Range.** The type holds jiff's civil range, `-9999-01-01T00:00:00` to
//!   `9999-12-31T23:59:59.999999999` (chrono's years run to ±262,143). A string chrono reads
//!   whose value lies outside it is an error, never a panic or a clamped value: chrono's "input
//!   is out of range" with [`ParseError::is_beyond_range`](crate::ParseError::is_beyond_range)
//!   true. A string chrono itself refuses as out of range (`2026-02-30T00:00:00`, a year past
//!   ±262,143) has the same message with `is_beyond_range()` false.
//! - **`Timestamp` is narrower.** [`Timestamp`](crate::Timestamp) ends at
//!   `9999-12-30T22:00:00.999999999Z` and starts at `-9999-01-02T01:59:59Z`, so
//!   `Timestamp::try_from(civil)` fails with [`OutOfTimestampRange`] for the civil values
//!   outside that (`9999-12-31T23:00:00`). There is no infallible conversion.
//!   `timestamp_millis` works over the whole civil range.
//! - **Leap seconds.** chrono reads a second of `60` (at any minute) as a leap second and
//!   writes `:60` back. This type has no leap second: it loads `2016-12-31T23:59:60.5` as
//!   `2017-01-01T00:00:00.500`, the next minute's second 0, which is the rule `Timestamp` uses.
//!   It loads, and never panics; the range check comes after the carry
//!   (`9999-12-31T23:59:60` is beyond the range). `Utc::now()` never gives `:60`, so no stored
//!   object holds one.
//! - `Hash` values differ from chrono's.
//!
//! # Old call → new call
//!
//! | Old (chrono) | New |
//! |---|---|
//! | `chrono::NaiveDateTime` | [`NaiveDateTime`] |
//! | `Utc::now().naive_utc()` | `NaiveDateTime::now_utc()` |
//! | `t.naive_utc()` | `NaiveDateTime::from(t)` |
//! | `n.and_utc()` | `Timestamp::try_from(n)?` (fallible) |
//! | `n.and_utc().timestamp_millis()`, `n.timestamp_millis()` | `n.timestamp_millis()` |
//! | `NaiveDate::from_ymd_opt(y, m, d)?.and_hms_nano_opt(h, mi, s, n)` | `NaiveDateTime::from_ymd_hms_nano(y, m, d, h, mi, s, n)` |
//! | `NaiveDateTime::from_str(s)`, `s.parse()` | the same |
//! | `NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S.%f")` (a test helper) | `s.parse()`: nine fraction digits read as the same value |
//! | `t.to_rfc3339_opts(SecondsFormat::Nanos, true)` | [`Timestamp::to_rfc3339_nanos`](crate::Timestamp::to_rfc3339_nanos) |
//! | `t.format("%Y-%m-%d %H:%M:%S")` on a `DateTime<Utc>` | [`Timestamp::strftime`](crate::Timestamp::strftime) (`Option`; the fork's directives only) |

pub use crate::jiff_backed::{NaiveDateTime, OutOfTimestampRange};
