//! Shared validation and data-classification helpers.

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

use crate::error::CloudSurfaceError;

pub(crate) fn validate_nonempty(
    value: &str,
    error: CloudSurfaceError,
) -> Result<(), CloudSurfaceError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

pub(crate) fn prefixed_token(
    value: String,
    prefix: &str,
    error: CloudSurfaceError,
) -> Result<String, CloudSurfaceError> {
    if value.starts_with(prefix)
        && value.len() > prefix.len()
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Ok(value)
    } else {
        Err(error)
    }
}

pub(crate) fn public_class(
    data_class: DataClass,
) -> Result<Classified<PrivacyDataClass>, CloudSurfaceError> {
    let class =
        PrivacyDataClass::new(data_class).map_err(|_| CloudSurfaceError::InvalidDataClass)?;
    if class.data_class() == DataClass::Public {
        Ok(public(class))
    } else {
        Err(CloudSurfaceError::InvalidDataClass)
    }
}

pub(crate) fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

pub(crate) fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}
