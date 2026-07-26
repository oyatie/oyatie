//! Validation traits and the [`ValidationError`] taxonomy, mirroring the Talos
//! `config.Validator` interface and `RuntimeMode` validation behavior.

use core::fmt;
use os_kernel::error::{Error, Result};

/// The runtime mode validation runs against. Talos validates a config
/// differently depending on whether it is applied at install time, in
/// container mode, or in metal/cloud mode (some fields are required only on
/// real hardware).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    /// Cloud/metal: full validation including install disk, network, etc.
    Metal,
    /// Container mode: relaxes install/disk requirements.
    Container,
    /// Validation while generating config (least strict).
    Generate,
}

impl ValidationMode {
    /// Whether an install disk must be specified in this mode.
    pub fn requires_install_disk(self) -> bool {
        matches!(self, ValidationMode::Metal)
    }
}

/// A single validation problem. Mirrors the way Talos accumulates a multierror
/// of warnings and errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// A required field was missing.
    MissingField(String),
    /// A field held an invalid value.
    InvalidValue { field: String, reason: String },
    /// Two settings conflict.
    Conflict(String),
    /// A non-fatal warning (deprecated field, etc.).
    Warning(String),
}

impl ValidationError {
    /// Convenience constructor for a missing field.
    pub fn missing(field: impl Into<String>) -> Self {
        ValidationError::MissingField(field.into())
    }

    /// Convenience constructor for an invalid value.
    pub fn invalid(field: impl Into<String>, reason: impl Into<String>) -> Self {
        ValidationError::InvalidValue {
            field: field.into(),
            reason: reason.into(),
        }
    }

    /// Whether this entry is fatal (anything but a warning).
    pub fn is_fatal(&self) -> bool {
        !matches!(self, ValidationError::Warning(_))
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::MissingField(field) => write!(f, "missing required field: {field}"),
            ValidationError::InvalidValue { field, reason } => {
                write!(f, "invalid value for {field}: {reason}")
            }
            ValidationError::Conflict(msg) => write!(f, "conflict: {msg}"),
            ValidationError::Warning(msg) => write!(f, "warning: {msg}"),
        }
    }
}

impl From<ValidationError> for Error {
    fn from(v: ValidationError) -> Error {
        if v.is_fatal() {
            Error::invalid(v.to_string())
        } else {
            Error::Other(v.to_string())
        }
    }
}

/// The accumulated result of a validation pass: zero or more warnings and zero
/// or more fatal errors. Mirrors Talos `Validate` returning `(warnings, error)`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationReport {
    entries: Vec<ValidationError>,
}

impl ValidationReport {
    /// An empty report.
    pub fn new() -> Self {
        ValidationReport {
            entries: Vec::new(),
        }
    }

    /// Record an entry.
    pub fn push(&mut self, e: ValidationError) {
        self.entries.push(e);
    }

    /// Merge another report's entries into this one.
    pub fn extend(&mut self, other: ValidationReport) {
        self.entries.extend(other.entries);
    }

    /// All entries.
    pub fn entries(&self) -> &[ValidationError] {
        &self.entries
    }

    /// The non-fatal warnings, as display strings.
    pub fn warnings(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter(|e| !e.is_fatal())
            .map(ToString::to_string)
            .collect()
    }

    /// Whether any fatal error was recorded.
    pub fn has_errors(&self) -> bool {
        self.entries.iter().any(ValidationError::is_fatal)
    }

    /// Convert into a `Result`: `Ok(warnings)` if no fatal errors, else `Err`
    /// with the first fatal error.
    pub fn into_result(self) -> Result<Vec<String>> {
        let warnings = self.warnings();
        match self.entries.into_iter().find(ValidationError::is_fatal) {
            Some(fatal) => Err(fatal.into()),
            None => Ok(warnings),
        }
    }
}

/// A type that can validate itself against a [`ValidationMode`].
///
/// Mirrors the Talos `config.Validator` interface.
pub trait Validator {
    /// Run validation, accumulating into a report.
    fn validate_into(&self, mode: ValidationMode, report: &mut ValidationReport);

    /// Run validation and collapse to a `Result<warnings>`.
    fn validate(&self, mode: ValidationMode) -> Result<Vec<String>> {
        let mut report = ValidationReport::new();
        self.validate_into(mode, &mut report);
        report.into_result()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Thing {
        disk: Option<&'static str>,
    }

    impl Validator for Thing {
        fn validate_into(&self, mode: ValidationMode, report: &mut ValidationReport) {
            if mode.requires_install_disk() && self.disk.is_none() {
                report.push(ValidationError::missing("machine.install.disk"));
            }
            if let Some(d) = self.disk
                && !d.starts_with("/dev/")
            {
                report.push(ValidationError::invalid(
                    "machine.install.disk",
                    "must be a device path",
                ));
            }
            report.push(ValidationError::Warning("example deprecation".to_string()));
        }
    }

    #[test]
    fn metal_requires_disk_container_does_not() {
        let thing = Thing { disk: None };
        assert!(thing.validate(ValidationMode::Metal).is_err());
        // Container mode only yields a warning -> Ok.
        let warnings = thing.validate(ValidationMode::Container).unwrap();
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn invalid_value_reported() {
        let thing = Thing { disk: Some("sda") };
        let err = thing.validate(ValidationMode::Metal).unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn report_separates_warnings_and_errors() {
        let mut r = ValidationReport::new();
        r.push(ValidationError::missing("x"));
        r.push(ValidationError::Warning("w".to_string()));
        assert!(r.has_errors());
        assert_eq!(r.warnings().len(), 1);
        assert!(r.into_result().is_err());
    }

    #[test]
    fn fatality_classification() {
        assert!(ValidationError::missing("a").is_fatal());
        assert!(ValidationError::invalid("a", "b").is_fatal());
        assert!(!ValidationError::Warning("w".to_string()).is_fatal());
    }
}
