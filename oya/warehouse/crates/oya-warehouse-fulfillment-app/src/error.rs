use std::error::Error;
use std::fmt::{Display, Formatter};

pub type Result<T> = std::result::Result<T, ServiceError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceError {
    kind: ServiceErrorKind,
    message: String,
    field: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceErrorKind {
    Configuration,
    InvariantViolation,
    Authorization,
    Validation,
    Conflict,
    NotFound,
    AdapterUnavailable,
    ContractStub,
}

impl ServiceError {
    pub fn new(kind: ServiceErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            field: None,
        }
    }

    pub fn with_field(
        kind: ServiceErrorKind,
        field: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            field: Some(field),
        }
    }

    pub fn configuration(message: impl Into<String>) -> Self {
        Self::new(ServiceErrorKind::Configuration, message)
    }

    pub fn invariant(field: &'static str, message: impl Into<String>) -> Self {
        Self::with_field(ServiceErrorKind::InvariantViolation, field, message)
    }

    pub fn validation(field: &'static str, message: impl Into<String>) -> Self {
        Self::with_field(ServiceErrorKind::Validation, field, message)
    }

    pub fn contract_stub(surface: &'static str) -> Self {
        Self::with_field(
            ServiceErrorKind::ContractStub,
            surface,
            "handler is intentionally scaffolded until implementation packet lands",
        )
    }
}

impl Display for ServiceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(field) = self.field {
            write!(formatter, "{:?} at {}: {}", self.kind, field, self.message)
        } else {
            write!(formatter, "{:?}: {}", self.kind, self.message)
        }
    }
}

impl Error for ServiceError {}

impl From<std::io::Error> for ServiceError {
    fn from(error: std::io::Error) -> Self {
        Self::configuration(error.to_string())
    }
}

impl From<toml::de::Error> for ServiceError {
    fn from(error: toml::de::Error) -> Self {
        Self::configuration(error.to_string())
    }
}
