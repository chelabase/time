//! The oracle builds with the pinned chrono and humantime, the versions the differential tests
//! compare against.

#[test]
fn chrono_and_humantime_link() {
    let t = chrono::DateTime::parse_from_rfc3339("2026-09-30T12:00:00.5Z").unwrap();
    assert_eq!(t.timestamp_millis(), 1_790_769_600_500);
    let d = humantime::parse_duration("1h 30m").unwrap();
    assert_eq!(humantime::format_duration(d).to_string(), "1h 30m");
    let ours = chelabase_time::parse_duration("1h 30m").unwrap();
    assert_eq!(ours, d);
}
