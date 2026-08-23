#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cell_clock_api::{Clock, ClockBindError, ClockSource, NTP_DEFAULT_UNCERTAINTY, NtpClock, bind};
use std::time::SystemTime;

#[test]
fn closed_sources_are_ntp_ptp_gnss() {
    assert_eq!(
        ClockSource::CLOSED,
        [
            ClockSource::Ntp,
            ClockSource::PtpPhc,
            ClockSource::GnssAtomic
        ]
    );
}

#[test]
fn ntp_now_is_an_interval_containing_wall_time() {
    let clock = NtpClock::default();
    let before = SystemTime::now();
    let interval = clock.now();
    let after = SystemTime::now();
    assert_eq!(clock.source(), ClockSource::Ntp);
    assert!(interval.contains(before) || interval.earliest <= before);
    assert!(interval.contains(after) || after <= interval.latest);
    assert!(interval.latest.duration_since(interval.earliest).unwrap() >= NTP_DEFAULT_UNCERTAINTY);
    assert_eq!(interval.logical, 0);
}

#[test]
fn bind_ntp_works_without_hardware() {
    let clock = bind(ClockSource::Ntp).expect("ntp is v1");
    let _ = clock.now();
}

#[test]
fn bind_ptp_and_gnss_fail_closed_until_wired() {
    assert_eq!(
        bind(ClockSource::PtpPhc).unwrap_err(),
        ClockBindError::AdapterNotWired(ClockSource::PtpPhc)
    );
    assert_eq!(
        bind(ClockSource::GnssAtomic).unwrap_err(),
        ClockBindError::AdapterNotWired(ClockSource::GnssAtomic)
    );
}
