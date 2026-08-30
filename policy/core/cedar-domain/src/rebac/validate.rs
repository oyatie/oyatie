//! Shared token validation for the ReBAC vocabulary.

use super::RebacTupleValidationError;

pub(super) fn validate_object_type(
    field: &'static str,
    value: &str,
) -> Result<(), RebacTupleValidationError> {
    validate_non_empty(field, value)?;
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        Ok(())
    } else {
        Err(RebacTupleValidationError::InvalidToken {
            field,
            value: value.to_owned(),
        })
    }
}

pub(super) fn validate_tenant_scope(
    field: &'static str,
    value: &str,
) -> Result<(), RebacTupleValidationError> {
    validate_non_empty(field, value)?;
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        Ok(())
    } else {
        Err(RebacTupleValidationError::InvalidToken {
            field,
            value: value.to_owned(),
        })
    }
}

pub(super) fn validate_relation(
    field: &'static str,
    value: &str,
) -> Result<(), RebacTupleValidationError> {
    validate_non_empty(field, value)?;
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        Ok(())
    } else {
        Err(RebacTupleValidationError::InvalidToken {
            field,
            value: value.to_owned(),
        })
    }
}

pub(super) fn validate_tuple_segment(
    field: &'static str,
    value: &str,
) -> Result<(), RebacTupleValidationError> {
    validate_non_empty(field, value)?;
    if value
        .chars()
        .all(|c| !c.is_whitespace() && c != '#' && c != '@')
    {
        Ok(())
    } else {
        Err(RebacTupleValidationError::InvalidToken {
            field,
            value: value.to_owned(),
        })
    }
}

pub(super) fn validate_opaque_token(
    field: &'static str,
    value: &str,
) -> Result<(), RebacTupleValidationError> {
    validate_non_empty(field, value)?;
    if value.chars().all(|c| !c.is_whitespace()) {
        Ok(())
    } else {
        Err(RebacTupleValidationError::InvalidToken {
            field,
            value: value.to_owned(),
        })
    }
}

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), RebacTupleValidationError> {
    if value.is_empty() {
        Err(RebacTupleValidationError::EmptyField { field })
    } else {
        Ok(())
    }
}
