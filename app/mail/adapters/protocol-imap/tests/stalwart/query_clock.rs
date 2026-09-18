// Minimal stand-in for the `chrono` surface the pinned Email/query suite and
// its seed use: `Utc::now() - Duration::days(n)` rendered as RFC 3339 / 2822.
use std::ops::Sub;

pub struct Utc;
#[derive(Clone, Copy)]
pub struct Duration(i64);
#[derive(Clone, Copy)]
pub struct DateTime(i64);

impl Utc {
    pub fn now() -> DateTime {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        DateTime(now.as_secs() as i64)
    }
}
impl Duration {
    pub fn days(days: i64) -> Self {
        Self(days * 86_400)
    }
    pub fn hours(hours: i64) -> Self {
        Self(hours * 3_600)
    }
}
impl Sub<Duration> for DateTime {
    type Output = DateTime;
    fn sub(self, rhs: Duration) -> DateTime {
        DateTime(self.0 - rhs.0)
    }
}

/// Proleptic Gregorian civil date from days since 1970-01-01.
fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

impl DateTime {
    pub fn timestamp(&self) -> i64 {
        self.0
    }
    fn parts(&self) -> (i64, u32, u32, i64, i64, i64, usize) {
        let days = self.0.div_euclid(86_400);
        let secs = self.0.rem_euclid(86_400);
        let (year, month, day) = civil(days);
        let weekday = (days + 4).rem_euclid(7) as usize;
        (
            year,
            month,
            day,
            secs / 3600,
            secs % 3600 / 60,
            secs % 60,
            weekday,
        )
    }
    pub fn to_rfc3339(&self) -> String {
        let (y, mo, d, h, mi, s, _) = self.parts();
        format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}+00:00")
    }
    pub fn to_rfc2822(&self) -> String {
        let (y, mo, d, h, mi, s, weekday) = self.parts();
        let day = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][weekday];
        let month = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ][(mo - 1) as usize];
        format!("{day}, {d:02} {month} {y:04} {h:02}:{mi:02}:{s:02} +0000")
    }
}

#[test]
fn clock_renders_known_instants() {
    assert_eq!(DateTime(0).to_rfc3339(), "1970-01-01T00:00:00+00:00");
    assert_eq!(
        DateTime(1_789_387_200).to_rfc2822(),
        "Mon, 14 Sep 2026 12:00:00 +0000"
    );
}
