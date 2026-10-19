//! The shared value-conformance station: the FINAL per-property and
//! per-parameter step of both conformance checks.
//!
//! `Some(declaration)` walks the carrier in declaration lockstep with zero
//! coercion; `None` requires the legacy [`PropertyValue::String`] carrier —
//! byte-identical to what every bridge constructor produces, closing the
//! typed-value-under-untyped-declaration loophole.

use crate::error::OntologyEngineError;
use crate::property::ObjectProperty;
use crate::value::PropertyValue;
use crate::value_type::{ValueTypeDeclaration, ValueTypeViolation};

/// Which conformance surface is asking — selects the error variant.
pub(crate) enum ValueCheckSubject {
    Property,
    Parameter,
}

pub(crate) fn check_declared_value(
    declared: Option<&ValueTypeDeclaration>,
    carrier: &ObjectProperty,
    subject: ValueCheckSubject,
) -> Result<(), OntologyEngineError> {
    let Err(violation) = admits_declared_value(declared, &carrier.value.value) else {
        return Ok(());
    };
    Err(match subject {
        ValueCheckSubject::Property => OntologyEngineError::PropertyValueTypeMismatch {
            name: carrier.name.clone(),
            path: violation.path,
            expected: violation.expected,
            found: violation.found,
        },
        ValueCheckSubject::Parameter => OntologyEngineError::ParameterValueTypeMismatch {
            name: carrier.name.clone(),
            path: violation.path,
            expected: violation.expected,
            found: violation.found,
        },
    })
}

pub(super) fn admits_declared_value(
    declared: Option<&ValueTypeDeclaration>,
    value: &PropertyValue,
) -> Result<(), ValueTypeViolation> {
    match declared {
        Some(declaration) => declaration.admits_value(value),
        None if matches!(value, PropertyValue::String(_)) => Ok(()),
        None => Err(ValueTypeViolation {
            path: String::new(),
            expected: "string",
            found: value.type_label(),
        }),
    }
}
