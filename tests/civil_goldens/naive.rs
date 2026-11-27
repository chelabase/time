//! chrono 0.4.45's `NaiveDateTime`, frozen as literals: what its `FromStr` and serde accept and
//! reject, and what it writes. Every row was printed by chrono through the oracle crate, and
//! `oracle/tests/civil.rs` checks each one against chrono for as long as the oracle exists.
//! `chelabase_time::civil::NaiveDateTime` must keep doing exactly this.

/// A string chrono accepts, and the value it reads, in each form chrono writes it.
pub struct Accepted {
    pub input: &'static str,
    /// `serde_json::to_string` (chrono's `Debug` form in quotes).
    pub json: &'static str,
    /// `Display`.
    pub display: &'static str,
    /// `and_utc().timestamp_millis()`.
    pub millis: i64,
}

/// A second of `60`. chrono keeps it as a leap second and writes `:60` back; this crate loads it
/// as the next minute's second 0 (`loaded`), the one stated exception to "the same value".
pub struct Leap {
    pub input: &'static str,
    /// chrono's `Debug` of what it read.
    pub chrono: &'static str,
    /// The same instant with the leap second carried.
    pub loaded: &'static str,
}

/// A string chrono accepts whose value lies outside jiff's civil range
/// (`-9999-01-01T00:00:00` to `9999-12-31T23:59:59.999999999`): an error here.
pub struct Beyond {
    pub input: &'static str,
    /// chrono's `Debug` of what it read.
    pub chrono: &'static str,
}

/// A string chrono rejects, and its error's `Display`.
pub struct Rejected {
    pub input: &'static str,
    pub error: &'static str,
}

/// Strings chrono accepts.
#[rustfmt::skip]
pub const ACCEPTED: &[Accepted] = &[
    Accepted { input: "2025-03-07T11:07:11.373105434", json: "\"2025-03-07T11:07:11.373105434\"", display: "2025-03-07 11:07:11.373105434", millis: 1741345631373 },
    Accepted { input: "2025-03-05T11:10:22.257901390", json: "\"2025-03-05T11:10:22.257901390\"", display: "2025-03-05 11:10:22.257901390", millis: 1741173022257 },
    Accepted { input: "2023-07-31T09:00:00.123", json: "\"2023-07-31T09:00:00.123\"", display: "2023-07-31 09:00:00.123", millis: 1690794000123 },
    Accepted { input: "2023-07-31T09:00:00.123000000", json: "\"2023-07-31T09:00:00.123\"", display: "2023-07-31 09:00:00.123", millis: 1690794000123 },
    Accepted { input: "2023-04-29T09:00:00.123", json: "\"2023-04-29T09:00:00.123\"", display: "2023-04-29 09:00:00.123", millis: 1682758800123 },
    Accepted { input: "2023-04-29T09:00:00.123000000", json: "\"2023-04-29T09:00:00.123\"", display: "2023-04-29 09:00:00.123", millis: 1682758800123 },
    Accepted { input: "2024-05-07T18:52:36.322426563", json: "\"2024-05-07T18:52:36.322426563\"", display: "2024-05-07 18:52:36.322426563", millis: 1715107956322 },
    Accepted { input: "2024-07-19T09:00:00.123", json: "\"2024-07-19T09:00:00.123\"", display: "2024-07-19 09:00:00.123", millis: 1721379600123 },
    Accepted { input: "2024-07-19T09:00:00.123000000", json: "\"2024-07-19T09:00:00.123\"", display: "2024-07-19 09:00:00.123", millis: 1721379600123 },
    Accepted { input: "2024-11-13T09:23:42.123", json: "\"2024-11-13T09:23:42.123\"", display: "2024-11-13 09:23:42.123", millis: 1731489822123 },
    Accepted { input: "2024-11-13T09:23:42.123000000", json: "\"2024-11-13T09:23:42.123\"", display: "2024-11-13 09:23:42.123", millis: 1731489822123 },
    Accepted { input: "2024-11-13T09:42:23.123", json: "\"2024-11-13T09:42:23.123\"", display: "2024-11-13 09:42:23.123", millis: 1731490943123 },
    Accepted { input: "2024-11-13T09:42:23.123000000", json: "\"2024-11-13T09:42:23.123\"", display: "2024-11-13 09:42:23.123", millis: 1731490943123 },
    Accepted { input: "2026-09-30T12:00:00", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026-09-30T12:00:00.5", json: "\"2026-09-30T12:00:00.500\"", display: "2026-09-30 12:00:00.500", millis: 1790769600500 },
    Accepted { input: "2026-09-30T12:00:00.12", json: "\"2026-09-30T12:00:00.120\"", display: "2026-09-30 12:00:00.120", millis: 1790769600120 },
    Accepted { input: "2026-09-30T12:00:00.123", json: "\"2026-09-30T12:00:00.123\"", display: "2026-09-30 12:00:00.123", millis: 1790769600123 },
    Accepted { input: "2026-09-30T12:00:00.1234", json: "\"2026-09-30T12:00:00.123400\"", display: "2026-09-30 12:00:00.123400", millis: 1790769600123 },
    Accepted { input: "2026-09-30T12:00:00.12345", json: "\"2026-09-30T12:00:00.123450\"", display: "2026-09-30 12:00:00.123450", millis: 1790769600123 },
    Accepted { input: "2026-09-30T12:00:00.123456", json: "\"2026-09-30T12:00:00.123456\"", display: "2026-09-30 12:00:00.123456", millis: 1790769600123 },
    Accepted { input: "2026-09-30T12:00:00.1234567", json: "\"2026-09-30T12:00:00.123456700\"", display: "2026-09-30 12:00:00.123456700", millis: 1790769600123 },
    Accepted { input: "2026-09-30T12:00:00.12345678", json: "\"2026-09-30T12:00:00.123456780\"", display: "2026-09-30 12:00:00.123456780", millis: 1790769600123 },
    Accepted { input: "2026-09-30T12:00:00.123456789", json: "\"2026-09-30T12:00:00.123456789\"", display: "2026-09-30 12:00:00.123456789", millis: 1790769600123 },
    Accepted { input: "2026-09-30T12:00:00.0", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026-09-30T12:00:00.000", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026-09-30T12:00:00.000000000", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026-09-30T12:00:00.100000000", json: "\"2026-09-30T12:00:00.100\"", display: "2026-09-30 12:00:00.100", millis: 1790769600100 },
    Accepted { input: "2026-09-30T12:00:00.001", json: "\"2026-09-30T12:00:00.001\"", display: "2026-09-30 12:00:00.001", millis: 1790769600001 },
    Accepted { input: "2026-09-30T12:00:00.000001", json: "\"2026-09-30T12:00:00.000001\"", display: "2026-09-30 12:00:00.000001", millis: 1790769600000 },
    Accepted { input: "2026-09-30T12:00:00.000000001", json: "\"2026-09-30T12:00:00.000000001\"", display: "2026-09-30 12:00:00.000000001", millis: 1790769600000 },
    Accepted { input: "2026-09-30T12:00:00.000001000", json: "\"2026-09-30T12:00:00.000001\"", display: "2026-09-30 12:00:00.000001", millis: 1790769600000 },
    Accepted { input: "2026-09-30T12:00:00.010000000", json: "\"2026-09-30T12:00:00.010\"", display: "2026-09-30 12:00:00.010", millis: 1790769600010 },
    Accepted { input: "2026-09-30T12:00:00.1234567891", json: "\"2026-09-30T12:00:00.123456789\"", display: "2026-09-30 12:00:00.123456789", millis: 1790769600123 },
    Accepted { input: "2026-09-30T12:00:00.9999999999", json: "\"2026-09-30T12:00:00.999999999\"", display: "2026-09-30 12:00:00.999999999", millis: 1790769600999 },
    Accepted { input: "2026-09-30T12:00:00.1234567891234567890123", json: "\"2026-09-30T12:00:00.123456789\"", display: "2026-09-30 12:00:00.123456789", millis: 1790769600123 },
    Accepted { input: "2026-09-30T12:00:00.00000000099", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026-09-30 T12:00:00", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026-09-30T 12:00:00", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026-09-30  T  12:00:00", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: " 2026-09-30T12:00:00", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "\t\n 2026-09-30T12:00:00", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026 -09-30T12:00:00", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026- 09-30T12:00:00", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026-09 -30T12:00:00", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026-09- 30T12:00:00", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026-09-30T12 :00:00", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026-09-30T12: 00:00", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026-09-30T12:00 :00", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026-09-30T12:00: 00", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026-09-30T12:00:00 ", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026-09-30T12:00:00.5 ", json: "\"2026-09-30T12:00:00.500\"", display: "2026-09-30 12:00:00.500", millis: 1790769600500 },
    Accepted { input: "2026-09-30T12:00:00.5 \n", json: "\"2026-09-30T12:00:00.500\"", display: "2026-09-30 12:00:00.500", millis: 1790769600500 },
    Accepted { input: "\u{a0}2026-09-30T12:00:00", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026-09-30\u{3000}T\u{2003}12:00:00\u{a0}", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026 - 09 - 30 T 12 : 00 : 00 ", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "2026-9-3T1:2:3", json: "\"2026-09-03T01:02:03\"", display: "2026-09-03 01:02:03", millis: 1788397323000 },
    Accepted { input: "2026-9-3T1:2:3.4", json: "\"2026-09-03T01:02:03.400\"", display: "2026-09-03 01:02:03.400", millis: 1788397323400 },
    Accepted { input: "026-09-30T12:00:00", json: "\"0026-09-30T12:00:00\"", display: "0026-09-30 12:00:00", millis: -61323134400000 },
    Accepted { input: "26-09-30T12:00:00", json: "\"0026-09-30T12:00:00\"", display: "0026-09-30 12:00:00", millis: -61323134400000 },
    Accepted { input: "6-09-30T12:00:00", json: "\"0006-09-30T12:00:00\"", display: "0006-09-30 12:00:00", millis: -61954286400000 },
    Accepted { input: "+2026-09-30T12:00:00", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "-2026-09-30T12:00:00", json: "\"-2026-09-30T12:00:00\"", display: "-2026-09-30 12:00:00", millis: -126078033600000 },
    Accepted { input: "+02026-09-30T12:00:00", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
    Accepted { input: "+26-09-30T12:00:00", json: "\"0026-09-30T12:00:00\"", display: "0026-09-30 12:00:00", millis: -61323134400000 },
    Accepted { input: "-1-01-01T00:00:00", json: "\"-0001-01-01T00:00:00\"", display: "-0001-01-01 00:00:00", millis: -62198755200000 },
    Accepted { input: "-0001-01-01T00:00:00", json: "\"-0001-01-01T00:00:00\"", display: "-0001-01-01 00:00:00", millis: -62198755200000 },
    Accepted { input: "0000-01-01T00:00:00", json: "\"0000-01-01T00:00:00\"", display: "0000-01-01 00:00:00", millis: -62167219200000 },
    Accepted { input: "-0000-01-01T00:00:00", json: "\"0000-01-01T00:00:00\"", display: "0000-01-01 00:00:00", millis: -62167219200000 },
    Accepted { input: "0001-01-01T00:00:00", json: "\"0001-01-01T00:00:00\"", display: "0001-01-01 00:00:00", millis: -62135596800000 },
    Accepted { input: "-9999-01-01T00:00:00", json: "\"-9999-01-01T00:00:00\"", display: "-9999-01-01 00:00:00", millis: -377705116800000 },
    Accepted { input: "-9999-01-01T00:00:00.000000001", json: "\"-9999-01-01T00:00:00.000000001\"", display: "-9999-01-01 00:00:00.000000001", millis: -377705116800000 },
    Accepted { input: "9999-12-31T23:59:59.999999999", json: "\"9999-12-31T23:59:59.999999999\"", display: "9999-12-31 23:59:59.999999999", millis: 253402300799999 },
    Accepted { input: "9999-12-31T23:00:00", json: "\"9999-12-31T23:00:00\"", display: "9999-12-31 23:00:00", millis: 253402297200000 },
    Accepted { input: "9999-12-30T22:00:00.999999999", json: "\"9999-12-30T22:00:00.999999999\"", display: "9999-12-30 22:00:00.999999999", millis: 253402207200999 },
    Accepted { input: "9999-12-30T22:00:01", json: "\"9999-12-30T22:00:01\"", display: "9999-12-30 22:00:01", millis: 253402207201000 },
    Accepted { input: "-9999-01-02T01:59:59", json: "\"-9999-01-02T01:59:59\"", display: "-9999-01-02 01:59:59", millis: -377705023201000 },
    Accepted { input: "-9999-01-02T01:59:58.999999999", json: "\"-9999-01-02T01:59:58.999999999\"", display: "-9999-01-02 01:59:58.999999999", millis: -377705023201001 },
    Accepted { input: "+9999-12-31T23:59:59", json: "\"9999-12-31T23:59:59\"", display: "9999-12-31 23:59:59", millis: 253402300799000 },
    Accepted { input: "2024-02-29T12:00:00", json: "\"2024-02-29T12:00:00\"", display: "2024-02-29 12:00:00", millis: 1709208000000 },
    Accepted { input: "2000-02-29T12:00:00", json: "\"2000-02-29T12:00:00\"", display: "2000-02-29 12:00:00", millis: 951825600000 },
    Accepted { input: "2026-09-30T23:59:59", json: "\"2026-09-30T23:59:59\"", display: "2026-09-30 23:59:59", millis: 1790812799000 },
    Accepted { input: "1970-01-01T00:00:00", json: "\"1970-01-01T00:00:00\"", display: "1970-01-01 00:00:00", millis: 0 },
    Accepted { input: "1970-01-01T00:00:00.001", json: "\"1970-01-01T00:00:00.001\"", display: "1970-01-01 00:00:00.001", millis: 1 },
    Accepted { input: "1969-12-31T23:59:59.999", json: "\"1969-12-31T23:59:59.999\"", display: "1969-12-31 23:59:59.999", millis: -1 },
    Accepted { input: "1969-12-31T23:59:59.999999999", json: "\"1969-12-31T23:59:59.999999999\"", display: "1969-12-31 23:59:59.999999999", millis: -1 },
    Accepted { input: "1969-12-31T23:59:59.9995", json: "\"1969-12-31T23:59:59.999500\"", display: "1969-12-31 23:59:59.999500", millis: -1 },
    Accepted { input: "1969-12-31T23:59:58.0001", json: "\"1969-12-31T23:59:58.000100\"", display: "1969-12-31 23:59:58.000100", millis: -2000 },
    Accepted { input: "1600-01-01T00:00:00.0009", json: "\"1600-01-01T00:00:00.000900\"", display: "1600-01-01 00:00:00.000900", millis: -11676096000000 },
    Accepted { input: "2026-09-30T12:00:0", json: "\"2026-09-30T12:00:00\"", display: "2026-09-30 12:00:00", millis: 1790769600000 },
];

/// Strings with a second of 60 that land inside jiff's civil range.
#[rustfmt::skip]
pub const LEAP: &[Leap] = &[
    Leap { input: "2016-12-31T23:59:60", chrono: "2016-12-31T23:59:60", loaded: "2017-01-01T00:00:00" },
    Leap { input: "2016-12-31T23:59:60.5", chrono: "2016-12-31T23:59:60.500", loaded: "2017-01-01T00:00:00.500" },
    Leap { input: "2016-12-31T23:59:60.999999999", chrono: "2016-12-31T23:59:60.999999999", loaded: "2017-01-01T00:00:00.999999999" },
    Leap { input: "2026-09-30T12:30:60", chrono: "2026-09-30T12:30:60", loaded: "2026-09-30T12:31:00" },
    Leap { input: "2026-02-28T23:59:60.123", chrono: "2026-02-28T23:59:60.123", loaded: "2026-03-01T00:00:00.123" },
    Leap { input: "2024-02-28T23:59:60", chrono: "2024-02-28T23:59:60", loaded: "2024-02-29T00:00:00" },
    Leap { input: "9999-12-31T23:58:60", chrono: "9999-12-31T23:58:60", loaded: "9999-12-31T23:59:00" },
    Leap { input: "-9999-01-01T00:00:60", chrono: "-9999-01-01T00:00:60", loaded: "-9999-01-01T00:01:00" },
    Leap { input: "-10000-12-31T23:59:60", chrono: "-10000-12-31T23:59:60", loaded: "-9999-01-01T00:00:00" },
    Leap { input: "-10000-12-31T23:59:60.25", chrono: "-10000-12-31T23:59:60.250", loaded: "-9999-01-01T00:00:00.250" },
];

/// Strings chrono accepts that jiff's civil range can't hold.
#[rustfmt::skip]
pub const BEYOND: &[Beyond] = &[
    Beyond { input: "+12345-01-01T00:00:00", chrono: "+12345-01-01T00:00:00" },
    Beyond { input: "-10000-12-31T23:59:59.999999999", chrono: "-10000-12-31T23:59:59.999999999" },
    Beyond { input: "+10000-01-01T00:00:00", chrono: "+10000-01-01T00:00:00" },
    Beyond { input: "+262142-12-31T23:59:59.999999999", chrono: "+262142-12-31T23:59:59.999999999" },
    Beyond { input: "-262143-01-01T00:00:00", chrono: "-262143-01-01T00:00:00" },
    Beyond { input: "+20000-02-29T12:00:00", chrono: "+20000-02-29T12:00:00" },
    Beyond { input: "9999-12-31T23:59:60", chrono: "9999-12-31T23:59:60" },
    Beyond { input: "9999-12-31T23:59:60.5", chrono: "9999-12-31T23:59:60.500" },
    Beyond { input: "+10000-01-01T00:00:60", chrono: "+10000-01-01T00:00:60" },
];

/// Strings chrono rejects.
#[rustfmt::skip]
pub const REJECTED: &[Rejected] = &[
    Rejected { input: "2026-09-30t12:00:00", error: "input contains invalid characters" },
    Rejected { input: "2026-09-30 12:00:00", error: "input contains invalid characters" },
    Rejected { input: "2026-09-30_12:00:00", error: "input contains invalid characters" },
    Rejected { input: "2026-09-3012:00:00", error: "input contains invalid characters" },
    Rejected { input: "2026-09-30T12:00:00 .5", error: "trailing input" },
    Rejected { input: "2026-09-30T12:00:00. 5", error: "input contains invalid characters" },
    Rejected { input: "2026-009-30T12:00:00", error: "input is out of range" },
    Rejected { input: "2026-09-030T12:00:00", error: "input contains invalid characters" },
    Rejected { input: "2026-09-30T012:00:00", error: "input contains invalid characters" },
    Rejected { input: "2026-09-30T12:000:00", error: "input contains invalid characters" },
    Rejected { input: "2026-09-30T12:00:000", error: "trailing input" },
    Rejected { input: "20260930T120000", error: "input contains invalid characters" },
    Rejected { input: "2026-09-30T120000", error: "input contains invalid characters" },
    Rejected { input: "12345-01-01T00:00:00", error: "input contains invalid characters" },
    Rejected { input: "+262143-01-01T00:00:00", error: "input is out of range" },
    Rejected { input: "-262144-12-31T23:59:59", error: "input is out of range" },
    Rejected { input: "+2147483647-01-01T00:00:00", error: "input is out of range" },
    Rejected { input: "+2147483648-01-01T00:00:00", error: "input is out of range" },
    Rejected { input: "-2147483648-01-01T00:00:00", error: "input is out of range" },
    Rejected { input: "-2147483649-01-01T00:00:00", error: "input is out of range" },
    Rejected { input: "+9223372036854775807-01-01T00:00:00", error: "input is out of range" },
    Rejected { input: "+9223372036854775808-01-01T00:00:00", error: "input is out of range" },
    Rejected { input: "-9223372036854775808-01-01T00:00:00", error: "input is out of range" },
    Rejected { input: "99999999999999999999-01-01T00:00:00", error: "input contains invalid characters" },
    Rejected { input: "+-2026-09-30T12:00:00", error: "input contains invalid characters" },
    Rejected { input: "+ 2026-09-30T12:00:00", error: "input contains invalid characters" },
    Rejected { input: "+-", error: "input contains invalid characters" },
    Rejected { input: "+", error: "premature end of input" },
    Rejected { input: "-", error: "premature end of input" },
    Rejected { input: "2026-00-30T12:00:00", error: "input is out of range" },
    Rejected { input: "2026-13-30T12:00:00", error: "input is out of range" },
    Rejected { input: "2026-09-00T12:00:00", error: "input is out of range" },
    Rejected { input: "2026-09-31T12:00:00", error: "input is out of range" },
    Rejected { input: "2026-09-32T12:00:00", error: "input is out of range" },
    Rejected { input: "2026-02-29T12:00:00", error: "input is out of range" },
    Rejected { input: "1900-02-29T12:00:00", error: "input is out of range" },
    Rejected { input: "+20000-02-30T12:00:00", error: "input is out of range" },
    Rejected { input: "2026-09-30T24:00:00", error: "input is out of range" },
    Rejected { input: "2026-09-30T12:60:00", error: "input is out of range" },
    Rejected { input: "2026-09-30T12:00:61", error: "input is out of range" },
    Rejected { input: "2026-09-30T99:99:99", error: "input is out of range" },
    Rejected { input: "2026-09-30T12:00:00Z", error: "trailing input" },
    Rejected { input: "2026-09-30T12:00:00z", error: "trailing input" },
    Rejected { input: "2026-09-30T12:00:00.5Z", error: "trailing input" },
    Rejected { input: "2026-09-30T12:00:00+00:00", error: "trailing input" },
    Rejected { input: "2026-09-30T12:00:00-05:30", error: "trailing input" },
    Rejected { input: "2026-09-30T12:00:00 UTC", error: "trailing input" },
    Rejected { input: "2026-09-30T12:00:00 +00:00", error: "trailing input" },
    Rejected { input: "2026-09-30T12:00:00x", error: "trailing input" },
    Rejected { input: "2026-09-30T12:00:00.5x", error: "trailing input" },
    Rejected { input: "2026-09-30T12:00:00 x", error: "trailing input" },
    Rejected { input: "2026-09-30T12:00:00.5\u{e9}", error: "trailing input" },
    Rejected { input: "2026-09-30T12:00:00\u{e9}", error: "trailing input" },
    Rejected { input: "2026-09-30T12:00:00.", error: "premature end of input" },
    Rejected { input: "2026-09-30T12:00:00.x", error: "input contains invalid characters" },
    Rejected { input: "2026-09-30T12:00:00,5", error: "trailing input" },
    Rejected { input: "2026-09-30T12:00:00..5", error: "input contains invalid characters" },
    Rejected { input: "2026-09-30T12:00:00.5.5", error: "trailing input" },
    Rejected { input: "2026-09-30T12:00:00.-5", error: "input contains invalid characters" },
    Rejected { input: "", error: "premature end of input" },
    Rejected { input: " ", error: "premature end of input" },
    Rejected { input: "2026", error: "premature end of input" },
    Rejected { input: "2026-", error: "premature end of input" },
    Rejected { input: "2026-09", error: "premature end of input" },
    Rejected { input: "2026-09-30", error: "premature end of input" },
    Rejected { input: "2026-09-30T", error: "premature end of input" },
    Rejected { input: "2026-09-30T12", error: "premature end of input" },
    Rejected { input: "2026-09-30T12:", error: "premature end of input" },
    Rejected { input: "2026-09-30T12:00", error: "premature end of input" },
    Rejected { input: "2026-09-30T12:00:", error: "premature end of input" },
    Rejected { input: "T12:00:00", error: "input contains invalid characters" },
    Rejected { input: "12:00:00", error: "input contains invalid characters" },
    Rejected { input: "x", error: "input contains invalid characters" },
    Rejected { input: "\u{e9}", error: "input contains invalid characters" },
    Rejected { input: "2026-09-30T12:00:\u{e9}0", error: "input contains invalid characters" },
    Rejected { input: "2026\u{2212}09\u{2212}30T12:00:00", error: "input contains invalid characters" },
    Rejected { input: "2026/09/30T12:00:00", error: "input contains invalid characters" },
    Rejected { input: "2026-09-30T12.00.00", error: "input contains invalid characters" },
    Rejected { input: "\u{ff12}\u{ff10}\u{ff12}\u{ff16}-09-30T12:00:00", error: "input contains invalid characters" },
    Rejected { input: "2026-09-30T1\u{662}:00:00", error: "input contains invalid characters" },
    Rejected { input: "2026-09-30\u{442}12:00:00", error: "input contains invalid characters" },
    Rejected { input: "now", error: "input contains invalid characters" },
    Rejected { input: "1790769600", error: "input contains invalid characters" },
];

/// JSON documents chrono's `Deserialize` refuses, and `serde_json`'s message for each.
#[rustfmt::skip]
pub const JSON_ERRORS: &[(&str, &str)] = &[
    ("1790769600", "invalid type: integer `1790769600`, expected a formatted date and time string at line 1 column 10"),
    ("1.5", "invalid type: floating point `1.5`, expected a formatted date and time string at line 1 column 3"),
    ("null", "invalid type: null, expected a formatted date and time string at line 1 column 4"),
    ("true", "invalid type: boolean `true`, expected a formatted date and time string at line 1 column 4"),
    ("[]", "invalid type: sequence, expected a formatted date and time string at line 1 column 0"),
    ("{}", "invalid type: map, expected a formatted date and time string at line 1 column 0"),
    ("[\"2026-09-30T12:00:00\"]", "invalid type: sequence, expected a formatted date and time string at line 1 column 0"),
    ("\"2026-09-30T12:00:00Z\"", "trailing input at line 1 column 22"),
    ("\"\"", "premature end of input at line 1 column 2"),
    ("\"2026-09-30T12:00:00\" x", "trailing characters at line 1 column 23"),
    ("", "EOF while parsing a value at line 1 column 0"),
];
