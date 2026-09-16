use crate::{Account, Error};
use serde::{Deserialize, Serialize};

const MAX_SUBJECT: usize = 511;
const MAX_BODY: usize = 2047;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VacationSettings {
    pub is_enabled: bool,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub subject: Option<String>,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
}

impl VacationSettings {
    pub fn validate(&self) -> Result<(), Error> {
        if self
            .from_date
            .as_deref()
            .is_some_and(|value| !valid_date(value))
            || self
                .to_date
                .as_deref()
                .is_some_and(|value| !valid_date(value))
            || !valid_text(self.subject.as_deref(), MAX_SUBJECT)
            || !valid_text(self.text_body.as_deref(), MAX_BODY)
            || !valid_text(self.html_body.as_deref(), MAX_BODY)
        {
            return Err(Error::Invalid);
        }
        Ok(())
    }
}

fn valid_text(value: Option<&str>, max: usize) -> bool {
    value.is_none_or(|value| value.len() <= max && !value.contains('\0'))
}

fn valid_date(value: &str) -> bool {
    let Some(value) = value.strip_suffix('Z') else {
        return false;
    };
    let Some((date, time)) = value.split_once('T') else {
        return false;
    };
    let Some((hour, rest)) = time.split_once(':') else {
        return false;
    };
    let Some((minute, second)) = rest.split_once(':') else {
        return false;
    };
    date.len() == 10
        && date.as_bytes()[4] == b'-'
        && date.as_bytes()[7] == b'-'
        && date[..4].bytes().all(|b| b.is_ascii_digit())
        && in_range(&date[5..7], 1, 12)
        && in_range(&date[8..10], 1, 31)
        && in_range(hour, 0, 23)
        && in_range(minute, 0, 59)
        && valid_second(second)
}

fn valid_second(value: &str) -> bool {
    let (second, fraction) = value.split_once('.').unwrap_or((value, ""));
    in_range(second, 0, 60)
        && (fraction.is_empty()
            || (fraction.len() <= 9 && fraction.bytes().all(|b| b.is_ascii_digit())))
}

fn in_range(value: &str, min: u32, max: u32) -> bool {
    value.len() == 2
        && value.bytes().all(|b| b.is_ascii_digit())
        && value.parse::<u32>().is_ok_and(|n| (min..=max).contains(&n))
}

impl Account {
    pub(super) fn set_vacation(&mut self, settings: VacationSettings) -> Result<(), Error> {
        settings.validate()?;
        if self.vacation != settings {
            self.vacation_revision = self
                .vacation_revision
                .checked_add(1)
                .ok_or(Error::OverQuota)?;
            self.vacation = settings;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;

    #[test]
    fn invalid_and_overflow_leave_the_snapshot_unchanged() {
        let mut account = Account::new("a", "t", "owner", "a@example.org").unwrap();
        account.vacation_revision = u64::MAX;
        let before = account.clone();
        let settings = VacationSettings {
            is_enabled: true,
            ..VacationSettings::default()
        };
        assert_eq!(
            account.apply(Command::SetVacation {
                settings: settings.clone()
            }),
            Err(Error::OverQuota)
        );
        assert_eq!(account, before);
        account.vacation_revision = 0;
        assert_eq!(
            account.apply(Command::SetVacation {
                settings: VacationSettings {
                    from_date: Some("not-a-date".into()),
                    ..settings
                }
            }),
            Err(Error::Invalid)
        );
        assert_eq!(account.vacation_revision, 0);
    }

    #[test]
    fn no_op_and_unrelated_mutations_preserve_vacation_revision() {
        let mut account = Account::new("a", "t", "owner", "a@example.org").unwrap();
        let settings = VacationSettings {
            is_enabled: true,
            subject: Some("Away".into()),
            from_date: Some("2026-03-01T00:00:00Z".into()),
            to_date: Some("2026-03-15T00:00:00Z".into()),
            ..VacationSettings::default()
        };
        account
            .apply(Command::SetVacation {
                settings: settings.clone(),
            })
            .unwrap();
        let revision = account.vacation_revision;
        account.apply(Command::SetVacation { settings }).unwrap();
        account
            .apply(Command::CreateMailbox {
                name: "Archive".into(),
            })
            .unwrap();
        assert_eq!(account.vacation_revision, revision);
        assert_eq!(account.mail_modseq, 0);
    }
}
