//! What humantime 2.2.0 and humantime-serde 1.1.1 (the Neon fork's locked versions) write and
//! read, frozen as literals. Every row was printed by those crates.
//!
//! Two tests read this file with `#[path]`: `tests/humantime_goldens.rs` holds
//! `chelabase_time::human` to it, and `oracle/tests/humantime.rs` holds humantime itself to it,
//! so a row can't drift from what humantime did while the oracle exists. On its own this file
//! is a test target with no tests.
//!
//! A duration or a time since the Unix epoch is written `(seconds, nanoseconds)`.
#![allow(dead_code)]

/// `(seconds, nanoseconds)`.
pub type D = (u64, u32);

/// A struct's `d`, `o` and `od` fields (see `Conf` in the tests), then its JSON and its TOML.
pub type SerRow = (
    D,
    Option<D>,
    Option<D>,
    Result<&'static str, &'static str>,
    Result<&'static str, &'static str>,
);

/// A document, then the `d`, `o` and `od` it deserializes to or the error's text.
pub type DeRow = (
    &'static str,
    Result<(D, Option<D>, Option<D>), &'static str>,
);

/// A document, then the one field it deserializes to (`(seconds, nanoseconds)`) or the error's
/// text.
pub type DeWithRow = (&'static str, Result<D, &'static str>);

/// `format_duration`: seconds, nanoseconds, text. Every text also parses back to its duration.
pub const FORMAT_DURATION: &[(u64, u32, &str)] = &[
    (0, 0, "0s"),
    (0, 1, "1ns"),
    (0, 999, "999ns"),
    (0, 1000, "1us"),
    (0, 1001, "1us 1ns"),
    (0, 999999, "999us 999ns"),
    (0, 1000000, "1ms"),
    (0, 100000000, "100ms"),
    (0, 999999999, "999ms 999us 999ns"),
    (1, 0, "1s"),
    (1, 1, "1s 1ns"),
    (1, 500000000, "1s 500ms"),
    (1, 1001001, "1s 1ms 1us 1ns"),
    (10, 0, "10s"),
    (59, 0, "59s"),
    (60, 0, "1m"),
    (61, 0, "1m 1s"),
    (90, 0, "1m 30s"),
    (600, 0, "10m"),
    (3599, 0, "59m 59s"),
    (3600, 0, "1h"),
    (3601, 0, "1h 1s"),
    (5400, 0, "1h 30m"),
    (86399, 0, "23h 59m 59s"),
    (86400, 0, "1day"),
    (86401, 0, "1day 1s"),
    (172800, 0, "2days"),
    (604800, 0, "7days"),
    (2592000, 0, "30days"),
    (2630015, 0, "30days 10h 33m 35s"),
    (2630016, 0, "1month"),
    (2630017, 0, "1month 1s"),
    (2678400, 0, "1month 13h 26m 24s"),
    (2716416, 0, "1month 1day"),
    (5260032, 0, "2months"),
    (28930176, 0, "11months"),
    (31560192, 0, "1year 43m 12s"),
    (31536000, 0, "11months 30days 3h 50m 24s"),
    (31557599, 0, "11months 30days 9h 50m 23s"),
    (31557600, 0, "1year"),
    (31557601, 0, "1year 1s"),
    (34187616, 0, "1year 1month"),
    (34277677, 1001001, "1year 1month 1day 1h 1m 1s 1ms 1us 1ns"),
    (63115200, 0, "2years"),
    (
        63115199,
        999999999,
        "1year 11months 30days 9h 50m 23s 999ms 999us 999ns",
    ),
    (1000000000, 0, "31years 8months 7days 19h 17m 52s"),
    (253370764800, 0, "8028years 10months 14h 24m"),
    (
        18446744073709551615,
        0,
        "584542046090years 7months 15days 17h 5m 3s",
    ),
    (
        18446744073709551615,
        999999999,
        "584542046090years 7months 15days 17h 5m 3s 999ms 999us 999ns",
    ),
];

/// `parse_duration` accepts: text, seconds, nanoseconds.
pub const PARSE_DURATION_OK: &[(&str, u64, u32)] = &[
    ("3nanos", 0, 3),
    ("3nsec", 0, 3),
    ("3ns", 0, 3),
    ("3usec", 0, 3000),
    ("3us", 0, 3000),
    ("3millis", 0, 3000000),
    ("3msec", 0, 3000000),
    ("3ms", 0, 3000000),
    ("3seconds", 3, 0),
    ("3second", 3, 0),
    ("3secs", 3, 0),
    ("3sec", 3, 0),
    ("3s", 3, 0),
    ("3minutes", 180, 0),
    ("3minute", 180, 0),
    ("3min", 180, 0),
    ("3mins", 180, 0),
    ("3m", 180, 0),
    ("3hours", 10800, 0),
    ("3hour", 10800, 0),
    ("3hr", 10800, 0),
    ("3hrs", 10800, 0),
    ("3h", 10800, 0),
    ("3days", 259200, 0),
    ("3day", 259200, 0),
    ("3d", 259200, 0),
    ("3weeks", 1814400, 0),
    ("3week", 1814400, 0),
    ("3w", 1814400, 0),
    ("3months", 7890048, 0),
    ("3month", 7890048, 0),
    ("3M", 7890048, 0),
    ("3years", 94672800, 0),
    ("3year", 94672800, 0),
    ("3y", 94672800, 0),
    ("1h 30m", 5400, 0),
    ("1h30m", 5400, 0),
    ("7days", 604800, 0),
    ("7 days", 604800, 0),
    ("10m", 600, 0),
    ("0s", 0, 0),
    ("1month", 2630016, 0),
    (" 1h ", 3600, 0),
    ("20 min 17 nsec ", 1200, 17),
    ("2h 15m", 8100, 0),
    ("1 0s", 10, 0),
    ("1 s", 1, 0),
    ("00s", 0, 0),
    ("007s", 7, 0),
    ("0ns", 0, 0),
    ("1m1m", 120, 0),
    ("1m 1m", 120, 0),
    ("1000000000ns", 1, 0),
    ("999999999ns 1ns", 1, 0),
    ("1000ms", 1, 0),
    ("1500ms", 1, 500000000),
    ("18446744073709551615s", 18446744073709551615, 0),
    (
        "18446744073709551614s 1000000000ns",
        18446744073709551615,
        0,
    ),
    ("18446744073709551615ns", 18446744073, 709551615),
    ("12M", 31560192, 0),
    ("1y", 31557600, 0),
    ("7d", 604800, 0),
    ("1w", 604800, 0),
    ("\u{a0}1s", 1, 0),
    ("1s\t2ms", 1, 2000000),
    ("1h\n", 3600, 0),
    ("1h\u{2003}30m", 5400, 0),
    ("1year 1month 1day 1h 1m 1s 1ms 1us 1ns", 34277677, 1001001),
    ("1ns 1us 1ms 1s 1m 1h 1day 1month 1year", 34277677, 1001001),
];

/// `parse_duration` rejects: text, then the error's `Debug`, which is the same in humantime
/// 2.2.0 and here.
pub const PARSE_DURATION_ERR: &[(&str, &str)] = &[
    ("", "Empty"),
    (" ", "Empty"),
    ("\t\n", "Empty"),
    (
        "5",
        "UnknownUnit { start: 1, end: 1, unit: \"\", value: 5 }",
    ),
    (
        "5parsecs",
        "UnknownUnit { start: 1, end: 8, unit: \"parsecs\", value: 5 }",
    ),
    (
        "1h 30",
        "UnknownUnit { start: 5, end: 5, unit: \"\", value: 30 }",
    ),
    ("h", "NumberExpected(0)"),
    ("s1", "NumberExpected(0)"),
    ("-1s", "NumberExpected(0)"),
    ("+1s", "NumberExpected(0)"),
    ("1s-", "InvalidCharacter(2)"),
    ("1h,30m", "InvalidCharacter(2)"),
    (
        "1S",
        "UnknownUnit { start: 1, end: 2, unit: \"S\", value: 1 }",
    ),
    (
        "1H",
        "UnknownUnit { start: 1, end: 2, unit: \"H\", value: 1 }",
    ),
    (
        "1Ms",
        "UnknownUnit { start: 1, end: 3, unit: \"Ms\", value: 1 }",
    ),
    (
        "1e3s",
        "UnknownUnit { start: 1, end: 2, unit: \"e\", value: 1 }",
    ),
    (
        "1s2",
        "UnknownUnit { start: 3, end: 3, unit: \"\", value: 2 }",
    ),
    (
        "1 2 3",
        "UnknownUnit { start: 5, end: 5, unit: \"\", value: 123 }",
    ),
    (
        "0x10s",
        "UnknownUnit { start: 1, end: 2, unit: \"x\", value: 0 }",
    ),
    ("1_000s", "InvalidCharacter(1)"),
    ("1sec.", "InvalidCharacter(4)"),
    (".5s", "NumberExpected(0)"),
    ("１s", "NumberExpected(0)"),
    ("١s", "NumberExpected(0)"),
    ("1h 30m!", "InvalidCharacter(6)"),
    ("1 hour and 30 minutes", "NumberExpected(7)"),
    ("PT1H30M", "NumberExpected(0)"),
    ("1:30", "InvalidCharacter(1)"),
    ("1h 30 m s", "NumberExpected(8)"),
    ("18446744073709551616s", "NumberOverflow"),
    ("18446744073709551615m", "NumberOverflow"),
    ("18446744073709551615s 1s", "NumberOverflow"),
    ("100000000000000000000ns", "NumberOverflow"),
    ("18446744073709551615us", "NumberOverflow"),
    ("18446744073709551615ms", "NumberOverflow"),
    ("10000000000000y", "NumberOverflow"),
    ("584542046091years", "NumberOverflow"),
    (
        "18446744073709551615ns 18446744073709551615ns",
        "NumberOverflow",
    ),
];

/// The wider grammar: strings humantime 2.2.0 rejects and `parse_duration` accepts (2.4.0's
/// fractions, `µs`, `wk`, `wks`, `yr`, `yrs` and the bare `0`). Text, our seconds and
/// nanoseconds, then humantime 2.2.0's error `Debug`.
pub const PARSE_DURATION_WIDER: &[(&str, u64, u32, &str)] = &[
    (
        "0",
        0,
        0,
        "UnknownUnit { start: 1, end: 1, unit: \"\", value: 0 }",
    ),
    ("1.5s", 1, 500000000, "InvalidCharacter(1)"),
    ("0.5h", 1800, 0, "InvalidCharacter(1)"),
    ("1.5m", 90, 0, "InvalidCharacter(1)"),
    ("1h 1.5m", 3690, 0, "InvalidCharacter(4)"),
    ("1.25ms", 0, 1250000, "InvalidCharacter(1)"),
    (
        "1wk",
        604800,
        0,
        "UnknownUnit { start: 1, end: 3, unit: \"wk\", value: 1 }",
    ),
    (
        "2wks",
        1209600,
        0,
        "UnknownUnit { start: 1, end: 4, unit: \"wks\", value: 2 }",
    ),
    (
        "1yr",
        31557600,
        0,
        "UnknownUnit { start: 1, end: 3, unit: \"yr\", value: 1 }",
    ),
    (
        "2yrs",
        63115200,
        0,
        "UnknownUnit { start: 1, end: 4, unit: \"yrs\", value: 2 }",
    ),
    ("3µs", 0, 3000, "InvalidCharacter(1)"),
    ("1s 3µs", 1, 3000, "InvalidCharacter(4)"),
];

/// Strings with a `.` or a `µ` that both reject, where the error may differ: text, our error's
/// `Debug`, humantime 2.2.0's.
pub const PARSE_DURATION_OTHER_ERR: &[(&str, &str, &str)] = &[
    (
        "1µ",
        "UnknownUnit { start: 1, end: 3, unit: \"µ\", value: 1 }",
        "InvalidCharacter(1)",
    ),
    (
        "1µm",
        "UnknownUnit { start: 1, end: 4, unit: \"µm\", value: 1 }",
        "InvalidCharacter(1)",
    ),
    ("µs", "NumberExpected(0)", "NumberExpected(0)"),
    ("1.s", "InvalidCharacter(1)", "InvalidCharacter(1)"),
    ("1.", "InvalidCharacter(1)", "InvalidCharacter(1)"),
    ("1.5ns", "NumberOverflow", "InvalidCharacter(1)"),
    (
        "1.5",
        "UnknownUnit { start: 3, end: 3, unit: \"\", value: 1 }",
        "InvalidCharacter(1)",
    ),
    (
        "1.5parsecs",
        "UnknownUnit { start: 3, end: 10, unit: \"parsecs\", value: 1 }",
        "InvalidCharacter(1)",
    ),
    ("1..5s", "InvalidCharacter(1)", "InvalidCharacter(1)"),
    ("1.0000000001s", "NumberOverflow", "InvalidCharacter(1)"),
    (
        "1.5wkz",
        "UnknownUnit { start: 3, end: 6, unit: \"wkz\", value: 1 }",
        "InvalidCharacter(1)",
    ),
];

/// The pending second: humantime 2.2.0 carried nanoseconds into seconds only above one second,
/// so a sum of exactly 1,000,000,000 ns stayed in its nanosecond counter, where a next
/// sub-second item of more than `u64::MAX - 1e9` ns overflowed it. `parse_duration` carries at
/// once and accepts these. Text, our seconds and nanoseconds, humantime 2.2.0's error `Debug`.
pub const PARSE_DURATION_PENDING_SECOND_OK: &[(&str, u64, u32, &str)] = &[
    (
        "1000000000ns 18446744073709551615ns",
        18446744074,
        709551615,
        "NumberOverflow",
    ),
    (
        "1000ms 18446744073709551615ns",
        18446744074,
        709551615,
        "NumberOverflow",
    ),
    (
        "1s 1000000us 18446744073709551615ns",
        18446744075,
        709551615,
        "NumberOverflow",
    ),
];

/// The pending second at `u64::MAX` seconds, or before a huge sub-second item, followed by
/// more: both reject, but one stops with `NumberOverflow` where the other read on. Text, our
/// error's `Debug`, humantime 2.2.0's.
pub const PARSE_DURATION_PENDING_SECOND_ERR: &[(&str, &str, &str)] = &[
    (
        "18446744073709551615s 1000000000ns 5parsecs",
        "NumberOverflow",
        "UnknownUnit { start: 36, end: 43, unit: \"parsecs\", value: 5 }",
    ),
    (
        "18446744073709551615s 1000ms 5",
        "NumberOverflow",
        "UnknownUnit { start: 30, end: 30, unit: \"\", value: 5 }",
    ),
    (
        "18446744073709551615s 1000000us !",
        "NumberOverflow",
        "NumberExpected(32)",
    ),
    (
        "18446744073709551615s 1000000000ns 1s",
        "NumberOverflow",
        "NumberOverflow",
    ),
    (
        "1000000000ns 18446744073709551615ns 5parsecs",
        "UnknownUnit { start: 37, end: 44, unit: \"parsecs\", value: 5 }",
        "NumberOverflow",
    ),
];

/// Strings on which humantime 2.2.0 panics (it builds `Duration::new(u64::MAX, 1_000_000_000)`).
/// `parse_duration` returns `NumberOverflow`.
pub const PARSE_DURATION_HUMANTIME_PANICS: &[&str] = &[
    "18446744073709551615s 1000000000ns",
    "18446744073709551615s 999999999ns 1ns",
    "18446744073709551615s 1000ms",
    "1000000us 18446744073709551615s",
];

/// One input per error variant: text, our `Display`, humantime 2.2.0's. Two messages differ
/// (ours are humantime 2.4.0's).
pub const DURATION_ERROR_DISPLAY: &[(&str, &str, &str)] = &[
    ("1s-", "invalid character at 2", "invalid character at 2"),
    ("s", "expected number at 0", "expected number at 0"),
    (
        "5",
        "time unit needed, for example 5sec or 5ms",
        "time unit needed, for example 5sec or 5ms",
    ),
    (
        "5parsecs",
        "unknown time unit \"parsecs\", supported units: ns, us/µs, ms, sec, min, hours, days, weeks, months, years (and few variations)",
        "unknown time unit \"parsecs\", supported units: ns, us, ms, sec, min, hours, days, weeks, months, years (and few variations)",
    ),
    (
        "18446744073709551616s",
        "number is too large or cannot be represented without a lack of precision (values below 1ns are not supported)",
        "number is too large",
    ),
    ("", "value was empty", "value was empty"),
];

/// `with = "…::human::serde"` serialization of `Conf`.
pub const SERDE_SERIALIZE: &[SerRow] = &[
    (
        (5400, 0),
        None,
        None,
        Ok("{\"d\":\"1h 30m\",\"o\":null,\"od\":null}"),
        Ok("d = \"1h 30m\"\n"),
    ),
    (
        (604800, 0),
        Some((600, 0)),
        Some((0, 0)),
        Ok("{\"d\":\"7days\",\"o\":\"10m\",\"od\":\"0s\"}"),
        Ok("d = \"7days\"\no = \"10m\"\nod = \"0s\"\n"),
    ),
    (
        (0, 0),
        Some((0, 1500000)),
        None,
        Ok("{\"d\":\"0s\",\"o\":\"1ms 500us\",\"od\":null}"),
        Ok("d = \"0s\"\no = \"1ms 500us\"\n"),
    ),
    (
        (2630016, 0),
        Some((31557600, 1)),
        Some((86400, 0)),
        Ok("{\"d\":\"1month\",\"o\":\"1year 1ns\",\"od\":\"1day\"}"),
        Ok("d = \"1month\"\no = \"1year 1ns\"\nod = \"1day\"\n"),
    ),
    (
        (18446744073709551615, 999999999),
        Some((18446744073709551615, 0)),
        None,
        Ok(
            "{\"d\":\"584542046090years 7months 15days 17h 5m 3s 999ms 999us 999ns\",\"o\":\"584542046090years 7months 15days 17h 5m 3s\",\"od\":null}",
        ),
        Ok(
            "d = \"584542046090years 7months 15days 17h 5m 3s 999ms 999us 999ns\"\no = \"584542046090years 7months 15days 17h 5m 3s\"\n",
        ),
    ),
];

/// JSON read into `Conf`: `null` and a missing key for the `Option`s, malformed strings and
/// values that aren't strings.
pub const SERDE_DESERIALIZE_JSON: &[DeRow] = &[
    (
        "{\"d\":\"1h 30m\",\"o\":\"7days\",\"od\":\"10m\"}",
        Ok(((5400, 0), Some((604800, 0)), Some((600, 0)))),
    ),
    ("{\"d\":\"1h30m\",\"o\":null}", Ok(((5400, 0), None, None))),
    (
        "{\"d\":\"0s\",\"o\":null,\"od\":null}",
        Ok(((0, 0), None, None)),
    ),
    (
        "{\"d\":\" 1h \",\"o\":\"1month\"}",
        Ok(((3600, 0), Some((2630016, 0)), None)),
    ),
    (
        "{\"d\":\"1h\"}",
        Err("missing field `o` at line 1 column 10"),
    ),
    ("{\"o\":null}", Err("missing field `d` at line 1 column 10")),
    (
        "{\"d\":\"5parsecs\",\"o\":null}",
        Err("invalid value: string \"5parsecs\", expected a duration at line 1 column 15"),
    ),
    (
        "{\"d\":\"\",\"o\":null}",
        Err("invalid value: string \"\", expected a duration at line 1 column 7"),
    ),
    (
        "{\"d\":\"1h 30\",\"o\":null}",
        Err("invalid value: string \"1h 30\", expected a duration at line 1 column 12"),
    ),
    (
        "{\"d\":5400,\"o\":null}",
        Err("invalid type: integer `5400`, expected a duration at line 1 column 9"),
    ),
    (
        "{\"d\":1.5,\"o\":null}",
        Err("invalid type: floating point `1.5`, expected a duration at line 1 column 8"),
    ),
    (
        "{\"d\":null,\"o\":null}",
        Err("invalid type: null, expected a duration at line 1 column 9"),
    ),
    (
        "{\"d\":true,\"o\":null}",
        Err("invalid type: boolean `true`, expected a duration at line 1 column 9"),
    ),
    (
        "{\"d\":[\"1h\"],\"o\":null}",
        Err("invalid type: sequence, expected a duration at line 1 column 5"),
    ),
    (
        "{\"d\":{\"secs\":1,\"nanos\":0},\"o\":null}",
        Err("invalid type: map, expected a duration at line 1 column 5"),
    ),
    (
        "{\"d\":\"1h\",\"o\":\"soon\"}",
        Err("invalid value: string \"soon\", expected a duration at line 1 column 20"),
    ),
    (
        "{\"d\":\"1h\",\"o\":30}",
        Err("invalid type: integer `30`, expected a duration at line 1 column 16"),
    ),
    (
        "{\"d\":\"1h\",\"o\":null,\"od\":\"1h 30\"}",
        Err("invalid value: string \"1h 30\", expected a duration at line 1 column 31"),
    ),
    (
        "{\"d\":\"1h\",\"o\":null,\"od\":7}",
        Err("invalid type: integer `7`, expected a duration at line 1 column 25"),
    ),
    (
        "{\"d\":\"18446744073709551616s\",\"o\":null}",
        Err(
            "invalid value: string \"18446744073709551616s\", expected a duration at line 1 column 28",
        ),
    ),
    (
        "{\"d\":\"18446744073709551615s\",\"o\":null}",
        Ok(((18446744073709551615, 0), None, None)),
    ),
    (
        "{\"d\":\"1hé\",\"o\":null}",
        Err("invalid value: string \"1hé\", expected a duration at line 1 column 11"),
    ),
];

/// TOML read into `Conf`.
pub const SERDE_DESERIALIZE_TOML: &[DeRow] = &[
    (
        "d = \"1h 30m\"\no = \"7days\"\nod = \"10m\"\n",
        Ok(((5400, 0), Some((604800, 0)), Some((600, 0)))),
    ),
    (
        "d = \"1h30m\"\no = \"1month\"\n",
        Ok(((5400, 0), Some((2630016, 0)), None)),
    ),
    (
        "d = \"1h\"\n",
        Err(
            "TOML parse error at line 1, column 1\n  |\n1 | d = \"1h\"\n  | ^\nmissing field `o`\n",
        ),
    ),
    (
        "o = \"1h\"\n",
        Err(
            "TOML parse error at line 1, column 1\n  |\n1 | o = \"1h\"\n  | ^\nmissing field `d`\n",
        ),
    ),
    (
        "d = \"5parsecs\"\no = \"1s\"\n",
        Err(
            "TOML parse error at line 1, column 5\n  |\n1 | d = \"5parsecs\"\n  |     ^^^^^^^^^^\ninvalid value: string \"5parsecs\", expected a duration\n",
        ),
    ),
    (
        "d = \"\"\no = \"1s\"\n",
        Err(
            "TOML parse error at line 1, column 5\n  |\n1 | d = \"\"\n  |     ^^\ninvalid value: string \"\", expected a duration\n",
        ),
    ),
    (
        "d = 5400\no = \"1s\"\n",
        Err(
            "TOML parse error at line 1, column 5\n  |\n1 | d = 5400\n  |     ^^^^\ninvalid type: integer `5400`, expected a duration\n",
        ),
    ),
    (
        "d = 1.5\no = \"1s\"\n",
        Err(
            "TOML parse error at line 1, column 5\n  |\n1 | d = 1.5\n  |     ^^^\ninvalid type: floating point `1.5`, expected a duration\n",
        ),
    ),
    (
        "d = true\no = \"1s\"\n",
        Err(
            "TOML parse error at line 1, column 5\n  |\n1 | d = true\n  |     ^^^^\ninvalid type: boolean `true`, expected a duration\n",
        ),
    ),
    (
        "d = 1979-05-27T07:32:00Z\no = \"1s\"\n",
        Err(
            "TOML parse error at line 1, column 5\n  |\n1 | d = 1979-05-27T07:32:00Z\n  |     ^^^^^^^^^^^^^^^^^^^^\ninvalid type: map, expected a duration\n",
        ),
    ),
    (
        "d = \"1h\"\no = 30\n",
        Err(
            "TOML parse error at line 2, column 5\n  |\n2 | o = 30\n  |     ^^\ninvalid type: integer `30`, expected a duration\n",
        ),
    ),
    (
        "d = \"1h\"\no = \"soon\"\n",
        Err(
            "TOML parse error at line 2, column 5\n  |\n2 | o = \"soon\"\n  |     ^^^^^^\ninvalid value: string \"soon\", expected a duration\n",
        ),
    ),
    (
        "d = \"1h\"\no = \"1s\"\nod = \"1h 30\"\n",
        Err(
            "TOML parse error at line 3, column 6\n  |\n3 | od = \"1h 30\"\n  |      ^^^^^^^\ninvalid value: string \"1h 30\", expected a duration\n",
        ),
    ),
];

/// JSON read through `deserialize_with = "…::deserialize"` into a plain `Duration` field
/// named `epoch`.
pub const SERDE_DESERIALIZE_WITH_JSON: &[DeWithRow] = &[
    ("{\"epoch\":\"10m\"}", Ok((600, 0))),
    (
        "{\"epoch\":\"ten\"}",
        Err("invalid value: string \"ten\", expected a duration at line 1 column 14"),
    ),
    (
        "{\"epoch\":600}",
        Err("invalid type: integer `600`, expected a duration at line 1 column 12"),
    ),
    (
        "{\"epoch\":null}",
        Err("invalid type: null, expected a duration at line 1 column 13"),
    ),
    ("{}", Err("missing field `epoch` at line 1 column 2")),
];

/// A time since the epoch (seconds, nanoseconds), then `format_rfc3339`,
/// `format_rfc3339_seconds` and `format_rfc3339_millis`.
pub const RFC3339_FORMAT: &[(u64, u32, &str, &str, &str)] = &[
    (
        0,
        0,
        "1970-01-01T00:00:00Z",
        "1970-01-01T00:00:00Z",
        "1970-01-01T00:00:00.000Z",
    ),
    (
        0,
        1,
        "1970-01-01T00:00:00.000000001Z",
        "1970-01-01T00:00:00Z",
        "1970-01-01T00:00:00.000Z",
    ),
    (
        0,
        1000000,
        "1970-01-01T00:00:00.001000000Z",
        "1970-01-01T00:00:00Z",
        "1970-01-01T00:00:00.001Z",
    ),
    (
        0,
        123456789,
        "1970-01-01T00:00:00.123456789Z",
        "1970-01-01T00:00:00Z",
        "1970-01-01T00:00:00.123Z",
    ),
    (
        0,
        500000000,
        "1970-01-01T00:00:00.500000000Z",
        "1970-01-01T00:00:00Z",
        "1970-01-01T00:00:00.500Z",
    ),
    (
        0,
        999999999,
        "1970-01-01T00:00:00.999999999Z",
        "1970-01-01T00:00:00Z",
        "1970-01-01T00:00:00.999Z",
    ),
    (
        1,
        0,
        "1970-01-01T00:00:01Z",
        "1970-01-01T00:00:01Z",
        "1970-01-01T00:00:01.000Z",
    ),
    (
        59,
        0,
        "1970-01-01T00:00:59Z",
        "1970-01-01T00:00:59Z",
        "1970-01-01T00:00:59.000Z",
    ),
    (
        60,
        0,
        "1970-01-01T00:01:00Z",
        "1970-01-01T00:01:00Z",
        "1970-01-01T00:01:00.000Z",
    ),
    (
        3599,
        0,
        "1970-01-01T00:59:59Z",
        "1970-01-01T00:59:59Z",
        "1970-01-01T00:59:59.000Z",
    ),
    (
        86399,
        0,
        "1970-01-01T23:59:59Z",
        "1970-01-01T23:59:59Z",
        "1970-01-01T23:59:59.000Z",
    ),
    (
        86400,
        0,
        "1970-01-02T00:00:00Z",
        "1970-01-02T00:00:00Z",
        "1970-01-02T00:00:00.000Z",
    ),
    (
        68255999,
        0,
        "1972-02-29T23:59:59Z",
        "1972-02-29T23:59:59Z",
        "1972-02-29T23:59:59.000Z",
    ),
    (
        68256000,
        0,
        "1972-03-01T00:00:00Z",
        "1972-03-01T00:00:00Z",
        "1972-03-01T00:00:00.000Z",
    ),
    (
        951782399,
        0,
        "2000-02-28T23:59:59Z",
        "2000-02-28T23:59:59Z",
        "2000-02-28T23:59:59.000Z",
    ),
    (
        951782400,
        0,
        "2000-02-29T00:00:00Z",
        "2000-02-29T00:00:00Z",
        "2000-02-29T00:00:00.000Z",
    ),
    (
        951868799,
        999999999,
        "2000-02-29T23:59:59.999999999Z",
        "2000-02-29T23:59:59Z",
        "2000-02-29T23:59:59.999Z",
    ),
    (
        951868800,
        0,
        "2000-03-01T00:00:00Z",
        "2000-03-01T00:00:00Z",
        "2000-03-01T00:00:00.000Z",
    ),
    (
        1518563287,
        0,
        "2018-02-13T23:08:07Z",
        "2018-02-13T23:08:07Z",
        "2018-02-13T23:08:07.000Z",
    ),
    (
        1518563287,
        133000000,
        "2018-02-13T23:08:07.133000000Z",
        "2018-02-13T23:08:07Z",
        "2018-02-13T23:08:07.133Z",
    ),
    (
        1518563287,
        133999999,
        "2018-02-13T23:08:07.133999999Z",
        "2018-02-13T23:08:07Z",
        "2018-02-13T23:08:07.133Z",
    ),
    (
        1518563287,
        1000,
        "2018-02-13T23:08:07.000001000Z",
        "2018-02-13T23:08:07Z",
        "2018-02-13T23:08:07.000Z",
    ),
    (
        1790769600,
        500000000,
        "2026-09-30T12:00:00.500000000Z",
        "2026-09-30T12:00:00Z",
        "2026-09-30T12:00:00.500Z",
    ),
    (
        2147483647,
        0,
        "2038-01-19T03:14:07Z",
        "2038-01-19T03:14:07Z",
        "2038-01-19T03:14:07.000Z",
    ),
    (
        2147483648,
        0,
        "2038-01-19T03:14:08Z",
        "2038-01-19T03:14:08Z",
        "2038-01-19T03:14:08.000Z",
    ),
    (
        4102444800,
        0,
        "2100-01-01T00:00:00Z",
        "2100-01-01T00:00:00Z",
        "2100-01-01T00:00:00.000Z",
    ),
    (
        4107542399,
        0,
        "2100-02-28T23:59:59Z",
        "2100-02-28T23:59:59Z",
        "2100-02-28T23:59:59.000Z",
    ),
    (
        4107542400,
        0,
        "2100-03-01T00:00:00Z",
        "2100-03-01T00:00:00Z",
        "2100-03-01T00:00:00.000Z",
    ),
    (
        13574563199,
        0,
        "2400-02-28T23:59:59Z",
        "2400-02-28T23:59:59Z",
        "2400-02-28T23:59:59.000Z",
    ),
    (
        13574563200,
        0,
        "2400-02-29T00:00:00Z",
        "2400-02-29T00:00:00Z",
        "2400-02-29T00:00:00.000Z",
    ),
    (
        13574649600,
        0,
        "2400-03-01T00:00:00Z",
        "2400-03-01T00:00:00Z",
        "2400-03-01T00:00:00.000Z",
    ),
    (
        253402207200,
        999999999,
        "9999-12-30T22:00:00.999999999Z",
        "9999-12-30T22:00:00Z",
        "9999-12-30T22:00:00.999Z",
    ),
    (
        253402207201,
        0,
        "9999-12-30T22:00:01Z",
        "9999-12-30T22:00:01Z",
        "9999-12-30T22:00:01.000Z",
    ),
    (
        253402214400,
        0,
        "9999-12-31T00:00:00Z",
        "9999-12-31T00:00:00Z",
        "9999-12-31T00:00:00.000Z",
    ),
    (
        253402297200,
        0,
        "9999-12-31T23:00:00Z",
        "9999-12-31T23:00:00Z",
        "9999-12-31T23:00:00.000Z",
    ),
    (
        253402300799,
        0,
        "9999-12-31T23:59:59Z",
        "9999-12-31T23:59:59Z",
        "9999-12-31T23:59:59.000Z",
    ),
    (
        253402300799,
        999999999,
        "9999-12-31T23:59:59.999999999Z",
        "9999-12-31T23:59:59Z",
        "9999-12-31T23:59:59.999Z",
    ),
];

/// `parse_rfc3339` accepts: text, then seconds and nanoseconds since the epoch. Second `60`
/// reads as second `59`, and fraction digits past the ninth are dropped.
pub const RFC3339_PARSE_OK: &[(&str, u64, u32)] = &[
    ("1970-01-01T00:00:00Z", 0, 0),
    ("1970-01-01T00:00:00.000000001Z", 0, 1),
    ("2018-02-14T00:28:07Z", 1518568087, 0),
    ("2018-02-14T00:28:07.133Z", 1518568087, 133000000),
    ("2018-02-14T00:28:07.000000000Z", 1518568087, 0),
    ("2018-02-14T00:28:07.123456789Z", 1518568087, 123456789),
    ("2018-02-14T00:28:07.1234567899Z", 1518568087, 123456789),
    (
        "2018-02-14T00:28:07.12345678901234567890Z",
        1518568087,
        123456789,
    ),
    ("2018-02-14T00:28:07.Z", 1518568087, 0),
    ("2018-02-14T00:28:07.1Z", 1518568087, 100000000),
    ("2016-12-31T23:59:60Z", 1483228799, 0),
    ("2016-12-31T23:59:60.5Z", 1483228799, 500000000),
    ("2018-02-14T12:00:60Z", 1518609659, 0),
    ("2000-02-29T00:00:00Z", 951782400, 0),
    ("2400-02-29T23:59:59Z", 13574649599, 0),
    ("2026-09-30T12:00:00.500Z", 1790769600, 500000000),
    ("9999-12-30T22:00:01Z", 253402207201, 0),
    ("9999-12-31T23:00:00Z", 253402297200, 0),
    ("9999-12-31T23:59:59Z", 253402300799, 0),
    ("9999-12-31T23:59:60Z", 253402300799, 0),
    ("9999-12-31T23:59:59.999999999Z", 253402300799, 999999999),
    ("9999-12-31T23:59:60.999999999Z", 253402300799, 999999999),
];

/// `parse_rfc3339` rejects: text, the error's `Debug`, its `Display`.
pub const RFC3339_PARSE_ERR: &[(&str, &str, &str)] = &[
    ("", "InvalidFormat", "timestamp format is invalid"),
    ("Z", "InvalidFormat", "timestamp format is invalid"),
    ("2018-02-14", "InvalidFormat", "timestamp format is invalid"),
    (
        "2018-02-14T00:28:07",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14T00:28:0Z",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14 00:28:07Z",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14 00:28:07",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14t00:28:07Z",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14T00:28:07z",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14T00:28:07+00:00",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14T00:28:07+01:00",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14T00:28:07-00:00",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14T00:28:07.133+01:00",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14T00:28:07+01:00Z",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14T00:28:07.+01Z",
        "InvalidDigit",
        "bad character where digit is expected",
    ),
    (
        "1969-12-31T23:59:59Z",
        "OutOfRange",
        "numeric component is out of range",
    ),
    (
        "1969-12-31T23:59:60Z",
        "OutOfRange",
        "numeric component is out of range",
    ),
    (
        "0000-01-01T00:00:00Z",
        "OutOfRange",
        "numeric component is out of range",
    ),
    (
        "0001-01-01T00:00:00Z",
        "OutOfRange",
        "numeric component is out of range",
    ),
    (
        "2018-13-01T00:00:00Z",
        "OutOfRange",
        "numeric component is out of range",
    ),
    (
        "2018-00-01T00:00:00Z",
        "OutOfRange",
        "numeric component is out of range",
    ),
    (
        "2018-02-29T00:00:00Z",
        "OutOfRange",
        "numeric component is out of range",
    ),
    (
        "2100-02-29T00:00:00Z",
        "OutOfRange",
        "numeric component is out of range",
    ),
    (
        "2018-02-30T00:00:00Z",
        "OutOfRange",
        "numeric component is out of range",
    ),
    (
        "2018-04-31T00:00:00Z",
        "OutOfRange",
        "numeric component is out of range",
    ),
    (
        "2018-02-00T00:00:00Z",
        "OutOfRange",
        "numeric component is out of range",
    ),
    (
        "2018-01-32T00:00:00Z",
        "OutOfRange",
        "numeric component is out of range",
    ),
    (
        "2018-02-14T24:00:00Z",
        "OutOfRange",
        "numeric component is out of range",
    ),
    (
        "2018-02-14T00:60:00Z",
        "OutOfRange",
        "numeric component is out of range",
    ),
    (
        "2018-02-14T00:00:61Z",
        "OutOfRange",
        "numeric component is out of range",
    ),
    (
        "2018-02-14T99:99:99Z",
        "OutOfRange",
        "numeric component is out of range",
    ),
    (
        "2018-02-14T00:28:07ZZ",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14T00:28:07.1Z3Z",
        "InvalidDigit",
        "bad character where digit is expected",
    ),
    (
        "2018-02-14T00:28:07.13aZ",
        "InvalidDigit",
        "bad character where digit is expected",
    ),
    (
        "2018-02-14T00:28:07.-1Z",
        "InvalidDigit",
        "bad character where digit is expected",
    ),
    (
        "2018-02-14T00:28:07. 1Z",
        "InvalidDigit",
        "bad character where digit is expected",
    ),
    (
        "2018-02-14T00:28:07,133Z",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14T00:28:07 Z",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14T00:28:07Z ",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        " 2018-02-14T00:28:07Z",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018/02/14T00:28:07Z",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14T00.28.07Z",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2O18-02-14T00:28:07Z",
        "InvalidDigit",
        "bad character where digit is expected",
    ),
    (
        "+018-02-14T00:28:07Z",
        "InvalidDigit",
        "bad character where digit is expected",
    ),
    (
        "-018-02-14T00:28:07Z",
        "InvalidDigit",
        "bad character where digit is expected",
    ),
    (
        "20180214T002807ZZZZZZ",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "10000-01-01T00:00:00Z",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "+10000-01-01T00:00:00Z",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "２018-02-14T00:28:07Z",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14T00:28:07.١Z",
        "InvalidDigit",
        "bad character where digit is expected",
    ),
    (
        "2018-02-14T00:28:0éZ",
        "InvalidDigit",
        "bad character where digit is expected",
    ),
    (
        "ééééééééééZ",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14T00:28:07.133",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14T00:28:07.",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14T0:28:07ZZ",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-2-14T00:28:07ZZ",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
    (
        "2018-02-14T00:28:07\n",
        "InvalidFormat",
        "timestamp format is invalid",
    ),
];

/// JSON read into a `SystemTime` field named `valid_until` the way the fork's
/// `SystemTimeAsRfc3339Millis` and `utils::serde_system_time` read it with humantime (a string,
/// then `parse_rfc3339`, the error as the serde error): the document, then the time since the
/// epoch or the error's text.
pub const RFC3339_MILLIS_DESERIALIZE_JSON: &[DeWithRow] = &[
    (
        "{\"valid_until\":\"2026-09-30T12:00:00.500Z\"}",
        Ok((1790769600, 500000000)),
    ),
    (
        "{\"valid_until\":\"2026-09-30T12:00:00.500999999Z\"}",
        Ok((1790769600, 500999999)),
    ),
    (
        "{\"valid_until\":\"9999-12-31T23:59:60Z\"}",
        Ok((253402300799, 0)),
    ),
    (
        "{\"valid_until\":\"2026-09-30T12:00:00.500+01:00\"}",
        Err("timestamp format is invalid at line 1 column 47"),
    ),
    (
        "{\"valid_until\":\"2026-09-30T12:00:00+00:00\"}",
        Err("timestamp format is invalid at line 1 column 43"),
    ),
    (
        "{\"valid_until\":\"1969-12-31T23:59:59Z\"}",
        Err("numeric component is out of range at line 1 column 38"),
    ),
    (
        "{\"valid_until\":\"2026-09-30T12:00:0xZ\"}",
        Err("bad character where digit is expected at line 1 column 38"),
    ),
    (
        "{\"valid_until\":\"\"}",
        Err("timestamp format is invalid at line 1 column 18"),
    ),
    (
        "{\"valid_until\":1790769600}",
        Err("invalid type: integer `1790769600`, expected a string at line 1 column 25"),
    ),
    (
        "{\"valid_until\":null}",
        Err("invalid type: null, expected a string at line 1 column 19"),
    ),
    ("{}", Err("missing field `valid_until` at line 1 column 2")),
];
