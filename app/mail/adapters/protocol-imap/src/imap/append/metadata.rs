use super::super::syntax;
use mail_kernel::{MAX_MESSAGE_BYTES, valid_keywords};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) struct Literal {
    pub size: usize,
    pub received_at: i64,
    pub keywords: Vec<String>,
    pub utf8: bool,
    pub nonsync: bool,
}

impl Literal {
    pub fn next(suffix: &[u8], utf8_enabled: bool) -> Result<Self, &'static str> {
        let text = std::str::from_utf8(suffix).map_err(|_| "BAD")?;
        if !text.starts_with(' ') {
            return Err("BAD");
        }
        let parts = syntax::command_parts(&format!("a APPEND INBOX{text}")).ok_or("BAD")?;
        Self::parse(parts.get(3..).ok_or("BAD")?, utf8_enabled)
    }

    pub fn parse(parts: &[String], utf8_enabled: bool) -> Result<Self, &'static str> {
        let marker = parts.last().ok_or("BAD")?;
        let utf8 = marker.starts_with("(~{") || marker.starts_with("({");
        if utf8
            && (!utf8_enabled
                || !parts
                    .get(parts.len().saturating_sub(2))
                    .is_some_and(|s| s.eq_ignore_ascii_case("UTF8")))
        {
            return Err("BAD");
        }
        let marker = if utf8 { &marker[1..] } else { marker };
        let marker = marker.strip_prefix('~').unwrap_or(marker);
        let (size, nonsync) = super::super::literal::specifier(marker.as_bytes()).ok_or("BAD")?;
        if size > MAX_MESSAGE_BYTES {
            return Err("NO [TOOBIG]");
        }
        let end = parts
            .len()
            .checked_sub(if utf8 { 2 } else { 1 })
            .ok_or("BAD")?;
        let mut options = &parts[..end];
        let mut keywords = vec![];
        if options.first().is_some_and(|s| s.starts_with('(')) {
            let end = options.iter().position(|s| s.ends_with(')')).ok_or("BAD")?;
            let flags = options[..=end].join(" ");
            keywords = flags[1..flags.len() - 1]
                .split_ascii_whitespace()
                .map(syntax::keyword)
                .collect::<Option<_>>()
                .ok_or("BAD")?;
            if !valid_keywords(&keywords) {
                return Err("BAD");
            }
            options = &options[end + 1..];
        }
        let received_at = match options {
            [] => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| "NO")?
                .as_secs() as i64,
            [date] => date_time(date).ok_or("BAD")?,
            _ => return Err("BAD"),
        };
        Ok(Self {
            size,
            received_at,
            keywords,
            utf8,
            nonsync,
        })
    }
}

pub(super) fn ascii_headers(raw: &[u8]) -> bool {
    let end = if raw.starts_with(b"\r\n") {
        0
    } else {
        raw.windows(4)
            .position(|w| w == b"\r\n\r\n")
            .unwrap_or(raw.len())
    };
    raw[..end].is_ascii()
}

fn date_time(input: &str) -> Option<i64> {
    let mut fields = input.trim_start().split(' ');
    let mut date = fields.next()?.split('-');
    let (day, month, year) = (date.next()?, date.next()?, date.next()?);
    let (time, zone) = (fields.next()?, fields.next()?);
    if fields.next().is_some()
        || date.next().is_some()
        || !(1..=2).contains(&day.len())
        || year.len() != 4
        || month.len() != 3
        || time.len() != 8
        || zone.len() != 5
        || ![day, year]
            .iter()
            .all(|s| s.bytes().all(|b| b.is_ascii_digit()))
        || !time.bytes().enumerate().all(|(i, b)| {
            if i == 2 || i == 5 {
                b == b':'
            } else {
                b.is_ascii_digit()
            }
        })
        || !zone.bytes().enumerate().all(|(i, b)| {
            if i == 0 {
                b == b'+' || b == b'-'
            } else {
                b.is_ascii_digit()
            }
        })
    {
        return None;
    }
    let month = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ]
    .iter()
    .position(|m| m.eq_ignore_ascii_case(month))? as u8
        + 1;
    let date = mail_parser::DateTime {
        year: year.parse().ok()?,
        month,
        day: day.parse().ok()?,
        hour: time[..2].parse().ok()?,
        minute: time[3..5].parse().ok()?,
        second: time[6..].parse().ok()?,
        tz_before_gmt: zone.starts_with('-'),
        tz_hour: zone[1..3].parse().ok()?,
        tz_minute: zone[3..].parse().ok()?,
    };
    if !date.is_valid() {
        return None;
    }
    let normalized = mail_parser::DateTime::from_timestamp(date.to_timestamp_local());
    if (date.year, date.month, date.day) != (normalized.year, normalized.month, normalized.day) {
        return None;
    }
    Some(date.to_timestamp())
}
