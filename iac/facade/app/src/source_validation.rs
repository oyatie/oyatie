use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ArchiveSourceProvider {
    S3,
    Gcs,
}

pub(super) fn validate_archive_source_location(
    module: &CloudIacReleaseIndexModuleSeed,
) -> Result<(), CloudIacReleaseIndexError> {
    let Some(location) = &module.archive_source_location else {
        return Ok(());
    };
    validate_non_empty("archive_source_location", location)?;
    if contains_secret_like_marker(location) {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_location",
            reason: "object source contains a credential-like marker".to_string(),
        });
    }
    if location
        .chars()
        .any(|ch| ch.is_ascii_whitespace() || ch.is_control())
    {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_location",
            reason: "object source must not contain whitespace or control characters".to_string(),
        });
    }
    if archive_source_provider(location).is_none() {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_location",
            reason: "object source must use s3::https:// or gcs::https://".to_string(),
        });
    }
    if location.contains('@') || location.contains('?') || location.contains('#') {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_location",
            reason: "object source must not embed userinfo, query strings, or fragments"
                .to_string(),
        });
    }
    let archive_name = archive_file_name(&module.archive_file)?;
    if !location.ends_with(&archive_name) {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_location",
            reason: "object source must end with the configured archive filename".to_string(),
        });
    }
    Ok(())
}

pub(super) fn validate_archive_source_pin(
    module: &CloudIacReleaseIndexModuleSeed,
) -> Result<(), CloudIacReleaseIndexError> {
    let Some(location) = &module.archive_source_location else {
        return validate_no_orphan_archive_source_pin(module);
    };
    let provider = archive_source_provider(location).ok_or_else(|| {
        CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_location",
            reason: "object source must use s3::https:// or gcs::https://".to_string(),
        }
    })?;

    let Some(source_integrity) = module.archive_source_integrity_sha256.as_deref() else {
        return Err(CloudIacReleaseIndexError::MissingField {
            field: "archive_source_integrity_sha256",
        });
    };
    validate_archive_source_integrity_sha256(source_integrity, &module.archive_sha256)?;

    match provider {
        ArchiveSourceProvider::S3 => {
            let Some(version_id) = module.archive_source_version_id.as_deref() else {
                return Err(CloudIacReleaseIndexError::MissingField {
                    field: "archive_source_version_id",
                });
            };
            validate_archive_source_version_id(version_id)?;
            if module.archive_source_generation.is_some() {
                return Err(CloudIacReleaseIndexError::InvalidField {
                    field: "archive_source_generation",
                    reason: "GCS generation metadata must not be set for S3 object sources"
                        .to_string(),
                });
            }
        }
        ArchiveSourceProvider::Gcs => {
            let Some(generation) = module.archive_source_generation.as_deref() else {
                return Err(CloudIacReleaseIndexError::MissingField {
                    field: "archive_source_generation",
                });
            };
            validate_archive_source_generation(generation)?;
            if module.archive_source_version_id.is_some() {
                return Err(CloudIacReleaseIndexError::InvalidField {
                    field: "archive_source_version_id",
                    reason: "S3 version-id metadata must not be set for GCS object sources"
                        .to_string(),
                });
            }
        }
    }
    Ok(())
}

pub(super) fn validate_no_orphan_archive_source_pin(
    module: &CloudIacReleaseIndexModuleSeed,
) -> Result<(), CloudIacReleaseIndexError> {
    if module.archive_source_integrity_sha256.is_some() {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_integrity_sha256",
            reason: "object-source integrity metadata requires archive_source_location".to_string(),
        });
    }
    if module.archive_source_version_id.is_some() {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_version_id",
            reason: "S3 version-id metadata requires archive_source_location".to_string(),
        });
    }
    if module.archive_source_generation.is_some() {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_generation",
            reason: "GCS generation metadata requires archive_source_location".to_string(),
        });
    }
    Ok(())
}

pub(super) fn validate_archive_source_integrity_sha256(
    source_integrity: &str,
    archive_sha256: &str,
) -> Result<(), CloudIacReleaseIndexError> {
    validate_non_empty("archive_source_integrity_sha256", source_integrity)?;
    if !is_lowercase_sha256(source_integrity) {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_integrity_sha256",
            reason: "object-source integrity must be exactly 64 lowercase hexadecimal characters"
                .to_string(),
        });
    }
    if source_integrity != archive_sha256 {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_integrity_sha256",
            reason: "object-source integrity must match archive_sha256".to_string(),
        });
    }
    Ok(())
}

pub(super) fn validate_archive_source_version_id(
    value: &str,
) -> Result<(), CloudIacReleaseIndexError> {
    validate_pin_token("archive_source_version_id", value, "S3 version-id metadata")
}

pub(super) fn validate_archive_source_generation(
    value: &str,
) -> Result<(), CloudIacReleaseIndexError> {
    validate_non_empty("archive_source_generation", value)?;
    if !value.chars().all(|ch| ch.is_ascii_digit()) || value.chars().all(|ch| ch == '0') {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "archive_source_generation",
            reason: "GCS generation metadata must be a non-zero ASCII decimal string".to_string(),
        });
    }
    Ok(())
}

pub(super) fn validate_pin_token(
    field: &'static str,
    value: &str,
    label: &'static str,
) -> Result<(), CloudIacReleaseIndexError> {
    validate_non_empty(field, value)?;
    if contains_secret_like_marker(value) {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field,
            reason: format!("{label} contains a credential-like marker"),
        });
    }
    if value
        .chars()
        .any(|ch| ch.is_ascii_whitespace() || ch.is_control())
    {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field,
            reason: format!("{label} must not contain whitespace or control characters"),
        });
    }
    if value.contains('\\')
        || value.contains('"')
        || value.contains('@')
        || value.contains('?')
        || value.contains('#')
    {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field,
            reason: format!("{label} must not contain URL/userinfo/control delimiters"),
        });
    }
    Ok(())
}

pub(super) fn archive_source_provider(location: &str) -> Option<ArchiveSourceProvider> {
    if location.starts_with("s3::https://") {
        Some(ArchiveSourceProvider::S3)
    } else if location.starts_with("gcs::https://") {
        Some(ArchiveSourceProvider::Gcs)
    } else {
        None
    }
}

pub(super) fn is_lowercase_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|ch| ch.is_ascii_digit() || matches!(ch, 'a'..='f'))
}

pub(super) fn validate_evidence_ref_seed(value: &str) -> Result<(), CloudIacReleaseIndexError> {
    validate_non_empty("evidence_ref", value)?;
    if !value.starts_with("evidence://") {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "evidence_ref",
            reason: "evidence ref must use evidence:// scheme".to_string(),
        });
    }
    if contains_secret_like_marker(value) {
        return Err(CloudIacReleaseIndexError::InvalidField {
            field: "evidence_ref",
            reason: "evidence ref contains a credential-like marker".to_string(),
        });
    }
    Ok(())
}
