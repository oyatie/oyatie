//! Foundry public API semver fitness kernel.
//!
//! ADR-0037 requires public contract artifacts to declare a stability tier and
//! versioning metadata before they can become tenant/ISV-facing commitments.
//! The kernel is intentionally pure: adapters discover contract files and parse
//! metadata, while this module validates the policy shape.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiContractRecord {
    pub artifact_path: String,                 // data_class: INTERNAL_ONLY
    pub metadata: Option<ApiContractMetadata>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiContractMetadata {
    pub metadata_path: String,     // data_class: INTERNAL_ONLY
    pub tier: String,              // data_class: INTERNAL_ONLY
    pub owner_team: String,        // data_class: INTERNAL_ONLY
    pub version: String,           // data_class: INTERNAL_ONLY
    pub sunset: String,            // data_class: INTERNAL_ONLY
    pub related_adrs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApiSemverReport {
    pub contracts_checked: usize, // data_class: INTERNAL_ONLY
    pub metadata_checked: usize,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApiSemverError {
    MissingMetadata {
        artifact_path: String,
    },
    MissingMetadataField {
        artifact_path: String,
        metadata_path: String,
        field: &'static str,
    },
    InvalidTier {
        artifact_path: String,
        metadata_path: String,
        tier: String,
    },
    InvalidVersion {
        artifact_path: String,
        metadata_path: String,
        version: String,
    },
    MissingVersionSuffix {
        artifact_path: String,
    },
    VersionSuffixMismatch {
        artifact_path: String,
        metadata_path: String,
        path_major: u64,
        metadata_major: u64,
    },
    MissingRelatedAdr {
        artifact_path: String,
        metadata_path: String,
    },
}

pub fn validate_api_semver<C>(contracts: C) -> Result<ApiSemverReport, ApiSemverError>
where
    C: IntoIterator<Item = ApiContractRecord>,
{
    let mut contracts_checked = 0usize;
    let mut metadata_checked = 0usize;

    for contract in contracts {
        contracts_checked += 1;
        let path_major = version_suffix_major(&contract.artifact_path).ok_or_else(|| {
            ApiSemverError::MissingVersionSuffix {
                artifact_path: contract.artifact_path.clone(),
            }
        })?;
        let metadata =
            contract
                .metadata
                .as_ref()
                .ok_or_else(|| ApiSemverError::MissingMetadata {
                    artifact_path: contract.artifact_path.clone(),
                })?;
        metadata_checked += 1;
        validate_required_field(&contract, metadata, "tier", &metadata.tier)?;
        validate_required_field(&contract, metadata, "owner_team", &metadata.owner_team)?;
        validate_required_field(&contract, metadata, "version", &metadata.version)?;
        validate_required_field(&contract, metadata, "sunset", &metadata.sunset)?;
        if !matches!(metadata.tier.as_str(), "preview" | "stable" | "GA") {
            return Err(ApiSemverError::InvalidTier {
                artifact_path: contract.artifact_path,
                metadata_path: metadata.metadata_path.clone(),
                tier: metadata.tier.clone(),
            });
        }
        let metadata_major =
            semver_major(&metadata.version).ok_or_else(|| ApiSemverError::InvalidVersion {
                artifact_path: contract.artifact_path.clone(),
                metadata_path: metadata.metadata_path.clone(),
                version: metadata.version.clone(),
            })?;
        if metadata_major != path_major {
            return Err(ApiSemverError::VersionSuffixMismatch {
                artifact_path: contract.artifact_path,
                metadata_path: metadata.metadata_path.clone(),
                path_major,
                metadata_major,
            });
        }
        if !metadata
            .related_adrs
            .iter()
            .any(|adr| adr_id(adr).is_some())
        {
            return Err(ApiSemverError::MissingRelatedAdr {
                artifact_path: contract.artifact_path,
                metadata_path: metadata.metadata_path.clone(),
            });
        }
    }

    Ok(ApiSemverReport {
        contracts_checked,
        metadata_checked,
    })
}

fn validate_required_field(
    contract: &ApiContractRecord,
    metadata: &ApiContractMetadata,
    field: &'static str,
    value: &str,
) -> Result<(), ApiSemverError> {
    if value.trim().is_empty() {
        Err(ApiSemverError::MissingMetadataField {
            artifact_path: contract.artifact_path.clone(),
            metadata_path: metadata.metadata_path.clone(),
            field,
        })
    } else {
        Ok(())
    }
}

fn version_suffix_major(path: &str) -> Option<u64> {
    let file_name = path.rsplit('/').next()?;
    let stem = file_name.split('.').next()?;
    let (_, suffix) = stem.rsplit_once("-v")?;
    suffix.parse::<u64>().ok()
}

fn semver_major(version: &str) -> Option<u64> {
    let mut parts = version.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    let _ = (minor, patch);
    Some(major)
}

fn adr_id(value: &str) -> Option<&str> {
    let index = value.find("ADR-")?;
    let candidate = value.get(index..index + 8)?;
    if candidate[4..].bytes().all(|byte| byte.is_ascii_digit()) {
        Some(candidate)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_empty_bootstrap_contract_set() {
        assert_eq!(
            validate_api_semver([]),
            Ok(ApiSemverReport {
                contracts_checked: 0,
                metadata_checked: 0,
            })
        );
    }

    #[test]
    fn accepts_contracts_with_tier_owner_semver_sunset_and_adr() {
        assert_eq!(
            validate_api_semver([contract(
                "contracts/openapi/workspace/mail-v1.yaml",
                Some(metadata("contracts/openapi/workspace/mail-v1.meta.yaml"))
            )]),
            Ok(ApiSemverReport {
                contracts_checked: 1,
                metadata_checked: 1,
            })
        );
    }

    #[test]
    fn rejects_missing_metadata() {
        assert_eq!(
            validate_api_semver([contract("contracts/openapi/workspace/mail-v1.yaml", None)]),
            Err(ApiSemverError::MissingMetadata {
                artifact_path: "contracts/openapi/workspace/mail-v1.yaml".into(),
            })
        );
    }

    #[test]
    fn rejects_invalid_tier_and_semver() {
        let mut invalid_tier = metadata("contracts/openapi/workspace/mail-v1.meta.yaml");
        invalid_tier.tier = "beta".into();
        assert_eq!(
            validate_api_semver([contract(
                "contracts/openapi/workspace/mail-v1.yaml",
                Some(invalid_tier)
            )]),
            Err(ApiSemverError::InvalidTier {
                artifact_path: "contracts/openapi/workspace/mail-v1.yaml".into(),
                metadata_path: "contracts/openapi/workspace/mail-v1.meta.yaml".into(),
                tier: "beta".into(),
            })
        );

        let mut invalid_version = metadata("contracts/openapi/workspace/mail-v1.meta.yaml");
        invalid_version.version = "1".into();
        assert_eq!(
            validate_api_semver([contract(
                "contracts/openapi/workspace/mail-v1.yaml",
                Some(invalid_version)
            )]),
            Err(ApiSemverError::InvalidVersion {
                artifact_path: "contracts/openapi/workspace/mail-v1.yaml".into(),
                metadata_path: "contracts/openapi/workspace/mail-v1.meta.yaml".into(),
                version: "1".into(),
            })
        );
    }

    #[test]
    fn rejects_missing_or_mismatched_version_suffix() {
        assert_eq!(
            validate_api_semver([contract(
                "contracts/openapi/workspace/mail.yaml",
                Some(metadata("contracts/openapi/workspace/mail.meta.yaml"))
            )]),
            Err(ApiSemverError::MissingVersionSuffix {
                artifact_path: "contracts/openapi/workspace/mail.yaml".into(),
            })
        );

        let mut metadata = metadata("contracts/openapi/workspace/mail-v2.meta.yaml");
        metadata.version = "1.0.0".into();
        assert_eq!(
            validate_api_semver([contract(
                "contracts/openapi/workspace/mail-v2.yaml",
                Some(metadata)
            )]),
            Err(ApiSemverError::VersionSuffixMismatch {
                artifact_path: "contracts/openapi/workspace/mail-v2.yaml".into(),
                metadata_path: "contracts/openapi/workspace/mail-v2.meta.yaml".into(),
                path_major: 2,
                metadata_major: 1,
            })
        );
    }

    #[test]
    fn rejects_metadata_without_related_adr() {
        let mut metadata = metadata("contracts/openapi/workspace/mail-v1.meta.yaml");
        metadata.related_adrs = vec!["not-an-adr".into()];
        assert_eq!(
            validate_api_semver([contract(
                "contracts/openapi/workspace/mail-v1.yaml",
                Some(metadata)
            )]),
            Err(ApiSemverError::MissingRelatedAdr {
                artifact_path: "contracts/openapi/workspace/mail-v1.yaml".into(),
                metadata_path: "contracts/openapi/workspace/mail-v1.meta.yaml".into(),
            })
        );
    }

    fn contract(path: &str, metadata: Option<ApiContractMetadata>) -> ApiContractRecord {
        ApiContractRecord {
            artifact_path: path.into(),
            metadata,
        }
    }

    fn metadata(path: &str) -> ApiContractMetadata {
        ApiContractMetadata {
            metadata_path: path.into(),
            tier: "preview".into(),
            owner_team: "platform-api-sdk".into(),
            version: "1.0.0".into(),
            sunset: "none".into(),
            related_adrs: vec!["ADR-0037".into()],
        }
    }
}
