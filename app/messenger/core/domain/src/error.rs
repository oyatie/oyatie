#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    Invalid(String),
    Denied,
    ActionRejected,
    Unencrypted,
    ArchiveNotReady,
    Unavailable(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) | Self::Unavailable(message) => f.write_str(message),
            Self::Denied => f.write_str("access denied"),
            Self::ActionRejected => f.write_str(
                "Foundry rejected this action before submission; correct the action or credential",
            ),
            Self::Unencrypted => {
                f.write_str("this room does not have end-to-end encryption enabled")
            }
            Self::ArchiveNotReady => f.write_str("enterprise archive participant is not ready"),
        }
    }
}

impl std::error::Error for Error {}
