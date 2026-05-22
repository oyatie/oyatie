use std::error::Error;
use std::fmt::{Display, Formatter};

pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ServiceError {
    InvalidIdentifier {
        field: &'static str,
        value: String,
    },
    MissingField {
        field: &'static str,
    },
    InvalidConfig {
        field: &'static str,
        details: String,
    },
    PolicyDenied {
        action: &'static str,
        reason: String,
    },
    InvariantViolation {
        invariant: &'static str,
        details: String,
    },
    PortUnavailable {
        port: &'static str,
    },
    Serialization {
        details: String,
    },
}

impl ServiceError {
    pub fn invalid_identifier(field: &'static str, value: impl Into<String>) -> Self {
        Self::InvalidIdentifier {
            field,
            value: value.into(),
        }
    }

    pub fn missing_field(field: &'static str) -> Self {
        Self::MissingField { field }
    }

    pub fn policy_denied(action: &'static str, reason: impl Into<String>) -> Self {
        Self::PolicyDenied {
            action,
            reason: reason.into(),
        }
    }

    pub fn invariant(invariant: &'static str, details: impl Into<String>) -> Self {
        Self::InvariantViolation {
            invariant,
            details: details.into(),
        }
    }
}

impl Display for ServiceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidIdentifier { field, value } => {
                write!(formatter, "invalid identifier for {field}: {value}")
            }
            Self::MissingField { field } => write!(formatter, "missing field: {field}"),
            Self::InvalidConfig { field, details } => {
                write!(formatter, "invalid config for {field}: {details}")
            }
            Self::PolicyDenied { action, reason } => {
                write!(formatter, "policy denied {action}: {reason}")
            }
            Self::InvariantViolation { invariant, details } => {
                write!(formatter, "invariant {invariant} violated: {details}")
            }
            Self::PortUnavailable { port } => write!(formatter, "port unavailable: {port}"),
            Self::Serialization { details } => write!(formatter, "serialization error: {details}"),
        }
    }
}

impl Error for ServiceError {}

impl From<serde_json::Error> for ServiceError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization {
            details: error.to_string(),
        }
    }
}
