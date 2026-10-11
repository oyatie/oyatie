use super::*;

pub(super) fn rfc3339(unix_seconds: i64) -> String {
    // Minimal RFC 3339 formatter — avoids a chrono dependency in the kernel.
    // For dates outside [1970-01-01, 9999-12-31] returns "1970-01-01T00:00:00Z".
    let secs = unix_seconds.max(0);
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let s = rem % 60;
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

pub(super) fn days_to_ymd(mut days: i64) -> (i64, i64, i64) {
    let mut year = 1970_i64;
    loop {
        let dly = if is_leap(year) { 366 } else { 365 };
        if days < dly {
            break;
        }
        days -= dly;
        year += 1;
    }
    let dim = [
        31,
        if is_leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut mo = 0;
    while days >= dim[mo] {
        days -= dim[mo];
        mo += 1;
    }
    (year, (mo + 1) as i64, days + 1)
}

pub(super) fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}
