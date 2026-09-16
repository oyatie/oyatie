use crate::{Account, Error, valid_address};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityAddress {
    pub name: Option<String>,
    pub email: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IdentitySettings {
    pub name: String,
    pub text_signature: String,
    pub html_signature: String,
    pub reply_to: Option<Vec<IdentityAddress>>,
    pub bcc: Option<Vec<IdentityAddress>>,
}

impl IdentitySettings {
    pub fn validate(&self) -> Result<(), Error> {
        let valid_name = |name: &str| name.len() <= 1024 && !name.chars().any(char::is_control);
        if !valid_name(&self.name)
            || self.text_signature.len() > 65536
            || self.html_signature.len() > 65536
            || self.text_signature.contains('\0')
            || self.html_signature.contains('\0')
        {
            return Err(Error::Invalid);
        }
        for addresses in [&self.reply_to, &self.bcc].into_iter().flatten() {
            if addresses.is_empty()
                || addresses.len() > 32
                || addresses.iter().any(|address| {
                    !valid_address(&address.email)
                        || address
                            .name
                            .as_deref()
                            .is_some_and(|name| !valid_name(name))
                })
            {
                return Err(Error::Invalid);
            }
        }
        Ok(())
    }
}

impl Account {
    pub(super) fn set_identity(&mut self, settings: IdentitySettings) -> Result<(), Error> {
        settings.validate()?;
        if self.identity != settings {
            let revision = self
                .identity_revision
                .checked_add(1)
                .ok_or(Error::OverQuota)?;
            self.identity = settings;
            self.identity_revision = revision;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;
    #[test]
    fn identity_overflow_and_invalid_addresses_leave_snapshot_unchanged() {
        let mut account = Account::new("a", "t", "owner", "a@example.org").unwrap();
        account.identity_revision = u64::MAX;
        let before = account.clone();
        let settings = IdentitySettings {
            name: "new".into(),
            ..IdentitySettings::default()
        };
        assert_eq!(
            account.apply(Command::SetIdentity { settings }),
            Err(Error::OverQuota)
        );
        assert_eq!(account, before);
        let settings = IdentitySettings {
            bcc: Some(vec![IdentityAddress {
                name: None,
                email: "invalid".into(),
            }]),
            ..IdentitySettings::default()
        };
        assert_eq!(
            account.apply(Command::SetIdentity { settings }),
            Err(Error::Invalid)
        );
        assert_eq!(account, before);
    }
    #[test]
    fn no_op_and_unrelated_mutations_preserve_identity_revision() {
        let mut account = Account::new("a", "t", "owner", "a@example.org").unwrap();
        let settings = IdentitySettings {
            name: "Alice".into(),
            ..IdentitySettings::default()
        };
        account
            .apply(Command::SetIdentity {
                settings: settings.clone(),
            })
            .unwrap();
        let revision = account.identity_revision;
        account.apply(Command::SetIdentity { settings }).unwrap();
        account
            .apply(Command::CreateMailbox {
                name: "Archive".into(),
            })
            .unwrap();
        assert_eq!(account.identity_revision, revision);
        assert_eq!(account.mail_modseq, 0);
    }
}
