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
    InvalidSunset {
        artifact_path: String,
        metadata_path: String,
        sunset: String,
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
        if !is_valid_sunset(&metadata.sunset) {
            return Err(ApiSemverError::InvalidSunset {
                artifact_path: contract.artifact_path.clone(),
                metadata_path: metadata.metadata_path.clone(),
                sunset: metadata.sunset.clone(),
            });
        }
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

/// Accept "none" or a calendar date YYYY-MM-DD with valid month (01-12) and day (01-N)
/// where N is the maximum day count for that month (leap-year-aware for February).
fn is_valid_sunset(s: &str) -> bool {
    if s == "none" {
        return true;
    }
    // Must be exactly YYYY-MM-DD: 10 chars, dashes at positions 4 and 7
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let year_bytes = &b[0..4];
    let month_bytes = &b[5..7];
    let day_bytes = &b[8..10];
    if !year_bytes.iter().all(|c| c.is_ascii_digit())
        || !month_bytes.iter().all(|c| c.is_ascii_digit())
        || !day_bytes.iter().all(|c| c.is_ascii_digit())
    {
        return false;
    }
    let year: u32 = parse_digits_u32(year_bytes);
    let month: u32 = parse_digits_u32(month_bytes);
    let day: u32 = parse_digits_u32(day_bytes);
    if month < 1 || month > 12 {
        return false;
    }
    let max_day = days_in_month(year, month);
    day >= 1 && day <= max_day
}

fn parse_digits_u32(digits: &[u8]) -> u32 {
    digits
        .iter()
        .fold(0u32, |acc, &d| acc * 10 + (d - b'0') as u32)
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
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

    #[test]
    fn accepts_sunset_none() {
        let mut m = metadata("contracts/openapi/workspace/mail-v1.meta.yaml");
        m.sunset = "none".into();
        assert_eq!(
            validate_api_semver([contract(
                "contracts/openapi/workspace/mail-v1.yaml",
                Some(m)
            )]),
            Ok(ApiSemverReport {
                contracts_checked: 1,
                metadata_checked: 1,
            })
        );
    }

    #[test]
    fn accepts_well_formed_sunset_date() {
        let mut m = metadata("contracts/openapi/workspace/mail-v1.meta.yaml");
        m.sunset = "2026-01-15".into();
        assert_eq!(
            validate_api_semver([contract(
                "contracts/openapi/workspace/mail-v1.yaml",
                Some(m)
            )]),
            Ok(ApiSemverReport {
                contracts_checked: 1,
                metadata_checked: 1,
            })
        );
    }

    #[test]
    fn rejects_sunset_freeform_string() {
        let mut m = metadata("contracts/openapi/workspace/mail-v1.meta.yaml");
        m.sunset = "soon".into();
        assert_eq!(
            validate_api_semver([contract(
                "contracts/openapi/workspace/mail-v1.yaml",
                Some(m)
            )]),
            Err(ApiSemverError::InvalidSunset {
                artifact_path: "contracts/openapi/workspace/mail-v1.yaml".into(),
                metadata_path: "contracts/openapi/workspace/mail-v1.meta.yaml".into(),
                sunset: "soon".into(),
            })
        );
    }

    #[test]
    fn rejects_sunset_invalid_month() {
        let mut m = metadata("contracts/openapi/workspace/mail-v1.meta.yaml");
        m.sunset = "2026-13-01".into();
        assert_eq!(
            validate_api_semver([contract(
                "contracts/openapi/workspace/mail-v1.yaml",
                Some(m)
            )]),
            Err(ApiSemverError::InvalidSunset {
                artifact_path: "contracts/openapi/workspace/mail-v1.yaml".into(),
                metadata_path: "contracts/openapi/workspace/mail-v1.meta.yaml".into(),
                sunset: "2026-13-01".into(),
            })
        );
    }

    #[test]
    fn rejects_sunset_wrong_separator() {
        let mut m = metadata("contracts/openapi/workspace/mail-v1.meta.yaml");
        m.sunset = "2026/01/01".into();
        assert_eq!(
            validate_api_semver([contract(
                "contracts/openapi/workspace/mail-v1.yaml",
                Some(m)
            )]),
            Err(ApiSemverError::InvalidSunset {
                artifact_path: "contracts/openapi/workspace/mail-v1.yaml".into(),
                metadata_path: "contracts/openapi/workspace/mail-v1.meta.yaml".into(),
                sunset: "2026/01/01".into(),
            })
        );
    }

    #[test]
    fn rejects_sunset_day_out_of_range() {
        let mut m = metadata("contracts/openapi/workspace/mail-v1.meta.yaml");
        m.sunset = "2026-01-32".into();
        assert_eq!(
            validate_api_semver([contract(
                "contracts/openapi/workspace/mail-v1.yaml",
                Some(m)
            )]),
            Err(ApiSemverError::InvalidSunset {
                artifact_path: "contracts/openapi/workspace/mail-v1.yaml".into(),
                metadata_path: "contracts/openapi/workspace/mail-v1.meta.yaml".into(),
                sunset: "2026-01-32".into(),
            })
        );
    }

    #[test]
    fn rejects_sunset_zero_month() {
        let mut m = metadata("contracts/openapi/workspace/mail-v1.meta.yaml");
        m.sunset = "2026-00-01".into();
        assert_eq!(
            validate_api_semver([contract(
                "contracts/openapi/workspace/mail-v1.yaml",
                Some(m)
            )]),
            Err(ApiSemverError::InvalidSunset {
                artifact_path: "contracts/openapi/workspace/mail-v1.yaml".into(),
                metadata_path: "contracts/openapi/workspace/mail-v1.meta.yaml".into(),
                sunset: "2026-00-01".into(),
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
