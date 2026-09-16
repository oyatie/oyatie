use super::{VacationSettings, valid_address};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VacationReply {
    pub to: String,
    pub subject: String,
    pub text: String,
}

impl VacationSettings {
    pub fn reply(&self, now: i64, raw: &[u8], account: &str) -> Option<VacationReply> {
        if !self.active_at(now) {
            return None;
        }
        let to = from_address(raw)?;
        if !eligible(&to, account, raw) {
            return None;
        }
        Some(VacationReply {
            to,
            subject: self.subject.clone().unwrap_or_else(|| "Auto: Away".into()),
            text: self
                .text_body
                .clone()
                .or_else(|| self.html_body.clone())
                .unwrap_or_else(|| "I am away.".into()),
        })
    }

    pub fn active_at(&self, now: i64) -> bool {
        self.is_enabled
            && self
                .from_date
                .as_deref()
                .map(unix_utc)
                .unwrap_or(Some(i64::MIN))
                .is_some_and(|start| start <= now)
            && self
                .to_date
                .as_deref()
                .map(unix_utc)
                .unwrap_or(Some(i64::MAX))
                .is_some_and(|end| now <= end)
    }
}

pub fn unix_utc(value: &str) -> Option<i64> {
    let value = value.strip_suffix('Z')?;
    let (date, time) = value.split_once('T')?;
    if date.len() != 10 {
        return None;
    }
    let year: i64 = date[..4].parse().ok()?;
    let month: u32 = date[5..7].parse().ok()?;
    let day: u32 = date[8..10].parse().ok()?;
    let (hour, rest) = time.split_once(':')?;
    let (minute, second) = rest.split_once(':')?;
    let hour: i64 = hour.parse().ok()?;
    let minute: i64 = minute.parse().ok()?;
    let second: i64 = second.split('.').next()?.parse().ok()?;
    Some(days_from_civil(year, month, day) * 86400 + hour * 3600 + minute * 60 + second)
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let month = month as i64;
    let day = day as i64;
    let year = year - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let yoe = (year - era * 400) as u64;
    let mp = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy as u64;
    era * 146097 + doe as i64 - 719468
}

fn eligible(from: &str, account: &str, raw: &[u8]) -> bool {
    !from.eq_ignore_ascii_case(account)
        && !from.to_ascii_lowercase().starts_with("mailer-daemon@")
        && header(raw, "auto-submitted").is_none_or(|v| v.eq_ignore_ascii_case("no"))
        && header(raw, "list-id").is_none()
        && header(raw, "list-unsubscribe").is_none()
        && header(raw, "precedence")
            .is_none_or(|v| !matches!(v.to_ascii_lowercase().as_str(), "bulk" | "list" | "junk"))
}

fn from_address(raw: &[u8]) -> Option<String> {
    address(&header(raw, "from")?)
}

fn address(value: &str) -> Option<String> {
    let value = value.trim();
    let extracted = if let (Some(start), Some(end)) = (value.rfind('<'), value.rfind('>')) {
        (end > start).then(|| value[start + 1..end].trim())?
    } else {
        value
    };
    valid_address(extracted).then(|| extracted.to_ascii_lowercase())
}

fn header(raw: &[u8], name: &str) -> Option<String> {
    let text = std::str::from_utf8(raw).ok()?;
    let headers = text.split("\r\n\r\n").next()?;
    for line in headers.split("\r\n") {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if key.eq_ignore_ascii_case(name) {
            return Some(value.trim().into());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_epoch_and_vacation_window_and_skips() {
        assert_eq!(unix_utc("1970-01-01T00:00:00Z"), Some(0));
        let mut settings = VacationSettings {
            is_enabled: true,
            subject: Some("Away".into()),
            text_body: Some("back later".into()),
            from_date: Some("2026-03-01T00:00:00Z".into()),
            to_date: Some("2026-03-15T00:00:00Z".into()),
            ..VacationSettings::default()
        };
        let raw = b"From: Bill <bill@remote.org>\r\nSubject: hi\r\n\r\nbody";
        let now = unix_utc("2026-03-10T12:00:00Z").unwrap();
        let reply = settings.reply(now, raw, "alice@example.org").unwrap();
        assert_eq!(reply.to, "bill@remote.org");
        assert_eq!(reply.subject, "Away");
        assert!(
            settings
                .reply(
                    unix_utc("2026-02-01T00:00:00Z").unwrap(),
                    raw,
                    "alice@example.org"
                )
                .is_none()
        );
        settings.is_enabled = false;
        assert!(settings.reply(now, raw, "alice@example.org").is_none());
        settings.is_enabled = true;
        let list = b"From: bill@remote.org\r\nList-Id: <list.example.org>\r\n\r\nx";
        assert!(settings.reply(now, list, "alice@example.org").is_none());
        let daemon = b"From: MAILER-DAEMON@remote.org\r\n\r\nx";
        assert!(settings.reply(now, daemon, "alice@example.org").is_none());
        let self_from = b"From: alice@example.org\r\n\r\nx";
        assert!(
            settings
                .reply(now, self_from, "alice@example.org")
                .is_none()
        );
    }
}
