//! Foundry supply-chain fitness kernel.
//!
//! ADR-0039's final posture is Trivy + Cosign + dual-format SBOM + signed
//! provenance. Bootstrap is not allowed to claim that posture early. This pure
//! kernel enforces the source-only phase: Rust dependency scanning must be wired,
//! higher catalog attestations require corresponding evidence, and release
//! manifests cannot appear before release-artifact scanning/signing evidence or
//! an explicit pre-release empty-scope declaration.

use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplyChainRecord {
    pub subject: String,     // data_class: INTERNAL_ONLY
    pub attestation: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SupplyChainEvidence {
    pub deny_config_present: bool,             // data_class: INTERNAL_ONLY
    pub cargo_deny_check_wired: bool,          // data_class: INTERNAL_ONLY
    pub cargo_audit_check_wired: bool,         // data_class: INTERNAL_ONLY
    pub third_party_actions_pinned: bool,      // data_class: INTERNAL_ONLY
    pub require_adr0039_evidence: bool,        // data_class: INTERNAL_ONLY
    pub release_manifest_present: bool,        // data_class: INTERNAL_ONLY
    pub release_images_declared: bool,         // data_class: INTERNAL_ONLY
    pub release_empty_scope_declared: bool,    // data_class: INTERNAL_ONLY
    pub trivy_release_scan_wired: bool,        // data_class: INTERNAL_ONLY
    pub trivy_filesystem_scan_wired: bool,     // data_class: INTERNAL_ONLY
    pub trivy_container_scan_wired: bool,      // data_class: INTERNAL_ONLY
    pub trivy_iac_scan_wired: bool,            // data_class: INTERNAL_ONLY
    pub trivy_dependency_scan_wired: bool,     // data_class: INTERNAL_ONLY
    pub cosign_release_signing_wired: bool,    // data_class: INTERNAL_ONLY
    pub cosign_rekor_verification_wired: bool, // data_class: INTERNAL_ONLY
    pub sbom_dual_format_wired: bool,          // data_class: INTERNAL_ONLY
    pub sbom_spdx_wired: bool,                 // data_class: INTERNAL_ONLY
    pub sbom_cyclonedx_wired: bool,            // data_class: INTERNAL_ONLY
    pub provenance_attestation_wired: bool,    // data_class: INTERNAL_ONLY
    pub signed_commit_policy_wired: bool,      // data_class: INTERNAL_ONLY
    pub admission_policy_wired: bool,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SupplyChainReport {
    pub records_checked: usize,     // data_class: INTERNAL_ONLY
    pub source_only_records: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseArtifact {
    pub artifact_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseSupplyChainEvidence {
    pub artifact_ref: String,               // data_class: INTERNAL_ONLY
    pub artifact_digest: String,            // data_class: INTERNAL_ONLY
    pub release_version: String,            // data_class: INTERNAL_ONLY
    pub source_revision: String,            // data_class: INTERNAL_ONLY
    pub sbom_spdx_ref: String,              // data_class: INTERNAL_ONLY
    pub sbom_cyclonedx_ref: String,         // data_class: INTERNAL_ONLY
    pub cosign_signature_ref: String,       // data_class: INTERNAL_ONLY
    pub cosign_certificate_ref: String,     // data_class: INTERNAL_ONLY
    pub rekor_log_index: u64,               // data_class: INTERNAL_ONLY
    pub trivy_filesystem_scan_ref: String,  // data_class: INTERNAL_ONLY
    pub trivy_container_scan_ref: String,   // data_class: INTERNAL_ONLY
    pub trivy_iac_scan_ref: String,         // data_class: INTERNAL_ONLY
    pub trivy_dependency_scan_ref: String,  // data_class: INTERNAL_ONLY
    pub provenance_attestation_ref: String, // data_class: INTERNAL_ONLY
    pub audit_event_type: String,           // data_class: INTERNAL_ONLY
    pub attestor: String,                   // data_class: INTERNAL_ONLY
    pub high_critical_findings_open: u64,   // data_class: INTERNAL_ONLY
    pub signed: bool,                       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReleaseSupplyChainReport {
    pub artifacts_checked: usize,        // data_class: INTERNAL_ONLY
    pub evidence_records_checked: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupplyChainError {
    NoCatalogRecords,
    MissingDenyConfig,
    MissingCargoDenyCheck,
    MissingCargoAuditCheck,
    UnpinnedThirdPartyAction,
    UnknownAttestation {
        subject: String,
        attestation: String,
    },
    UnsupportedAttestationClaim {
        subject: String,
        attestation: String,
        missing_evidence: &'static str,
    },
    MissingAdr0039Evidence {
        missing_evidence: &'static str,
    },
    ReleaseManifestWithoutAdr0039Evidence {
        missing_evidence: &'static str,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReleaseSupplyChainError {
    NoReleaseArtifacts,
    NoEvidenceRecords,
    MissingPreReleaseEmptyScopeRationale,
    InvalidArtifactRef {
        artifact_ref: String,
    },
    DuplicateArtifact {
        artifact_ref: String,
    },
    DuplicateEvidence {
        artifact_ref: String,
    },
    MissingEvidenceForArtifact {
        artifact_ref: String,
    },
    EvidenceForUnknownArtifact {
        artifact_ref: String,
    },
    MissingField {
        artifact_ref: String,
        field: &'static str,
    },
    InvalidDigest {
        artifact_ref: String,
    },
    DigestNotPinnedInArtifactRef {
        artifact_ref: String,
        artifact_digest: String,
    },
    InvalidReleaseVersion {
        artifact_ref: String,
        release_version: String,
    },
    InvalidSourceRevision {
        artifact_ref: String,
        source_revision: String,
    },
    MissingRekorInclusion {
        artifact_ref: String,
    },
    OpenHighCriticalFindings {
        artifact_ref: String,
        count: u64,
    },
    UnsignedEvidence {
        artifact_ref: String,
    },
    InvalidAuditEventType {
        artifact_ref: String,
        audit_event_type: String,
    },
    InvalidSbomRef {
        artifact_ref: String,
        field: &'static str,
    },
    InvalidTrivyRef {
        artifact_ref: String,
        field: &'static str,
    },
}

pub fn validate_supply_chain<R>(
    records: R,
    evidence: SupplyChainEvidence,
) -> Result<SupplyChainReport, SupplyChainError>
where
    R: IntoIterator<Item = SupplyChainRecord>,
{
    if !evidence.deny_config_present {
        return Err(SupplyChainError::MissingDenyConfig);
    }
    if !evidence.cargo_deny_check_wired {
        return Err(SupplyChainError::MissingCargoDenyCheck);
    }
    if !evidence.cargo_audit_check_wired {
        return Err(SupplyChainError::MissingCargoAuditCheck);
    }
    if !evidence.third_party_actions_pinned {
        return Err(SupplyChainError::UnpinnedThirdPartyAction);
    }
    if evidence.require_adr0039_evidence {
        require_adr0039_evidence(evidence)?;
    }
    if evidence.release_manifest_present {
        if !evidence.trivy_release_scan_wired {
            return Err(SupplyChainError::ReleaseManifestWithoutAdr0039Evidence {
                missing_evidence: "trivy_release_scan_wired",
            });
        }
        if !evidence.cosign_release_signing_wired {
            return Err(SupplyChainError::ReleaseManifestWithoutAdr0039Evidence {
                missing_evidence: "cosign_release_signing_wired",
            });
        }
        if !evidence.sbom_dual_format_wired {
            return Err(SupplyChainError::ReleaseManifestWithoutAdr0039Evidence {
                missing_evidence: "sbom_dual_format_wired",
            });
        }
    }

    let mut records_checked = 0usize;
    let mut source_only_records = 0usize;
    for record in records {
        records_checked += 1;
        match record.attestation.as_str() {
            "source-only" => source_only_records += 1,
            "license-checked" => {
                if !evidence.cargo_deny_check_wired || !evidence.cargo_audit_check_wired {
                    return Err(SupplyChainError::UnsupportedAttestationClaim {
                        subject: record.subject,
                        attestation: record.attestation,
                        missing_evidence: "rust_dependency_scan_wiring",
                    });
                }
            }
            "sbom" => {
                if !evidence.sbom_dual_format_wired {
                    return Err(SupplyChainError::UnsupportedAttestationClaim {
                        subject: record.subject,
                        attestation: record.attestation,
                        missing_evidence: "sbom_dual_format_wired",
                    });
                }
            }
            "signed-provenance" => {
                if !evidence.sbom_dual_format_wired {
                    return Err(SupplyChainError::UnsupportedAttestationClaim {
                        subject: record.subject,
                        attestation: record.attestation,
                        missing_evidence: "sbom_dual_format_wired",
                    });
                }
                if !evidence.cosign_release_signing_wired {
                    return Err(SupplyChainError::UnsupportedAttestationClaim {
                        subject: record.subject,
                        attestation: record.attestation,
                        missing_evidence: "cosign_release_signing_wired",
                    });
                }
            }
            _ => {
                return Err(SupplyChainError::UnknownAttestation {
                    subject: record.subject,
                    attestation: record.attestation,
                });
            }
        }
    }

    if records_checked == 0 {
        Err(SupplyChainError::NoCatalogRecords)
    } else {
        Ok(SupplyChainReport {
            records_checked,
            source_only_records,
        })
    }
}

pub fn validate_release_supply_chain<A, E>(
    artifacts: A,
    evidence_records: E,
) -> Result<ReleaseSupplyChainReport, ReleaseSupplyChainError>
where
    A: IntoIterator<Item = ReleaseArtifact>,
    E: IntoIterator<Item = ReleaseSupplyChainEvidence>,
{
    let artifacts = release_artifact_map(artifacts)?;
    if artifacts.is_empty() {
        return Err(ReleaseSupplyChainError::NoReleaseArtifacts);
    }
    let evidence = release_evidence_map(evidence_records)?;
    if evidence.is_empty() {
        return Err(ReleaseSupplyChainError::NoEvidenceRecords);
    }

    validate_release_supply_chain_maps(artifacts, evidence)
}

fn validate_release_supply_chain_maps(
    artifacts: BTreeMap<String, ReleaseArtifact>,
    evidence: BTreeMap<String, ReleaseSupplyChainEvidence>,
) -> Result<ReleaseSupplyChainReport, ReleaseSupplyChainError> {
    for artifact_ref in artifacts.keys() {
        if !evidence.contains_key(artifact_ref) {
            return Err(ReleaseSupplyChainError::MissingEvidenceForArtifact {
                artifact_ref: artifact_ref.clone(),
            });
        }
    }
    for record in evidence.values() {
        if !artifacts.contains_key(&record.artifact_ref) {
            return Err(ReleaseSupplyChainError::EvidenceForUnknownArtifact {
                artifact_ref: record.artifact_ref.clone(),
            });
        }
        validate_release_evidence_record(record)?;
    }

    Ok(ReleaseSupplyChainReport {
        artifacts_checked: artifacts.len(),
        evidence_records_checked: evidence.len(),
    })
}

pub fn validate_pre_release_supply_chain<A, E>(
    artifacts: A,
    evidence_records: E,
    empty_scope_declared: bool,
) -> Result<ReleaseSupplyChainReport, ReleaseSupplyChainError>
where
    A: IntoIterator<Item = ReleaseArtifact>,
    E: IntoIterator<Item = ReleaseSupplyChainEvidence>,
{
    let artifacts = release_artifact_map(artifacts)?;
    let evidence = release_evidence_map(evidence_records)?;

    if artifacts.is_empty() {
        if evidence.is_empty() && empty_scope_declared {
            return Ok(ReleaseSupplyChainReport {
                artifacts_checked: 0,
                evidence_records_checked: 0,
            });
        }
        return Err(ReleaseSupplyChainError::MissingPreReleaseEmptyScopeRationale);
    }

    if evidence.is_empty() {
        return Ok(ReleaseSupplyChainReport {
            artifacts_checked: artifacts.len(),
            evidence_records_checked: 0,
        });
    }

    validate_release_supply_chain_maps(artifacts, evidence)
}

fn release_artifact_map<A>(
    artifacts: A,
) -> Result<BTreeMap<String, ReleaseArtifact>, ReleaseSupplyChainError>
where
    A: IntoIterator<Item = ReleaseArtifact>,
{
    let mut map = BTreeMap::new();
    for artifact in artifacts {
        let artifact_ref = artifact.artifact_ref.trim();
        if artifact_ref.is_empty() {
            return Err(ReleaseSupplyChainError::InvalidArtifactRef {
                artifact_ref: artifact.artifact_ref,
            });
        }
        if !artifact_ref.contains('@') || !artifact_ref.contains("sha256:") {
            return Err(ReleaseSupplyChainError::InvalidArtifactRef {
                artifact_ref: artifact.artifact_ref,
            });
        }
        if map
            .insert(artifact_ref.to_string(), artifact.clone())
            .is_some()
        {
            return Err(ReleaseSupplyChainError::DuplicateArtifact {
                artifact_ref: artifact_ref.to_string(),
            });
        }
    }
    Ok(map)
}

fn release_evidence_map<E>(
    evidence_records: E,
) -> Result<BTreeMap<String, ReleaseSupplyChainEvidence>, ReleaseSupplyChainError>
where
    E: IntoIterator<Item = ReleaseSupplyChainEvidence>,
{
    let mut map = BTreeMap::new();
    for evidence in evidence_records {
        let artifact_ref = evidence.artifact_ref.trim();
        if artifact_ref.is_empty() {
            return Err(ReleaseSupplyChainError::InvalidArtifactRef {
                artifact_ref: evidence.artifact_ref,
            });
        }
        if map
            .insert(artifact_ref.to_string(), evidence.clone())
            .is_some()
        {
            return Err(ReleaseSupplyChainError::DuplicateEvidence {
                artifact_ref: artifact_ref.to_string(),
            });
        }
    }
    Ok(map)
}

fn validate_release_evidence_record(
    record: &ReleaseSupplyChainEvidence,
) -> Result<(), ReleaseSupplyChainError> {
    for (field, value) in [
        ("artifact_digest", &record.artifact_digest),
        ("release_version", &record.release_version),
        ("source_revision", &record.source_revision),
        ("sbom_spdx_ref", &record.sbom_spdx_ref),
        ("sbom_cyclonedx_ref", &record.sbom_cyclonedx_ref),
        ("cosign_signature_ref", &record.cosign_signature_ref),
        ("cosign_certificate_ref", &record.cosign_certificate_ref),
        (
            "trivy_filesystem_scan_ref",
            &record.trivy_filesystem_scan_ref,
        ),
        ("trivy_container_scan_ref", &record.trivy_container_scan_ref),
        ("trivy_iac_scan_ref", &record.trivy_iac_scan_ref),
        (
            "trivy_dependency_scan_ref",
            &record.trivy_dependency_scan_ref,
        ),
        (
            "provenance_attestation_ref",
            &record.provenance_attestation_ref,
        ),
        ("audit_event_type", &record.audit_event_type),
        ("attestor", &record.attestor),
    ] {
        if value.trim().is_empty() {
            return Err(ReleaseSupplyChainError::MissingField {
                artifact_ref: record.artifact_ref.clone(),
                field,
            });
        }
    }

    if !is_sha256_digest(&record.artifact_digest) {
        return Err(ReleaseSupplyChainError::InvalidDigest {
            artifact_ref: record.artifact_ref.clone(),
        });
    }
    if !record.artifact_ref.contains(&record.artifact_digest) {
        return Err(ReleaseSupplyChainError::DigestNotPinnedInArtifactRef {
            artifact_ref: record.artifact_ref.clone(),
            artifact_digest: record.artifact_digest.clone(),
        });
    }
    if !is_semver(&record.release_version) {
        return Err(ReleaseSupplyChainError::InvalidReleaseVersion {
            artifact_ref: record.artifact_ref.clone(),
            release_version: record.release_version.clone(),
        });
    }
    if !is_git_sha(&record.source_revision) {
        return Err(ReleaseSupplyChainError::InvalidSourceRevision {
            artifact_ref: record.artifact_ref.clone(),
            source_revision: record.source_revision.clone(),
        });
    }
    if record.rekor_log_index == 0 {
        return Err(ReleaseSupplyChainError::MissingRekorInclusion {
            artifact_ref: record.artifact_ref.clone(),
        });
    }
    if record.high_critical_findings_open != 0 {
        return Err(ReleaseSupplyChainError::OpenHighCriticalFindings {
            artifact_ref: record.artifact_ref.clone(),
            count: record.high_critical_findings_open,
        });
    }
    if !record.signed {
        return Err(ReleaseSupplyChainError::UnsignedEvidence {
            artifact_ref: record.artifact_ref.clone(),
        });
    }
    if record.audit_event_type != "oya.audit.builder_supply_attest" {
        return Err(ReleaseSupplyChainError::InvalidAuditEventType {
            artifact_ref: record.artifact_ref.clone(),
            audit_event_type: record.audit_event_type.clone(),
        });
    }
    for (field, value, suffix) in [
        ("sbom_spdx_ref", &record.sbom_spdx_ref, ".spdx.json"),
        (
            "sbom_cyclonedx_ref",
            &record.sbom_cyclonedx_ref,
            ".cyclonedx.json",
        ),
    ] {
        if !value.ends_with(suffix) {
            return Err(ReleaseSupplyChainError::InvalidSbomRef {
                artifact_ref: record.artifact_ref.clone(),
                field,
            });
        }
    }
    for (field, value) in [
        (
            "trivy_filesystem_scan_ref",
            &record.trivy_filesystem_scan_ref,
        ),
        ("trivy_container_scan_ref", &record.trivy_container_scan_ref),
        ("trivy_iac_scan_ref", &record.trivy_iac_scan_ref),
        (
            "trivy_dependency_scan_ref",
            &record.trivy_dependency_scan_ref,
        ),
    ] {
        if !value.to_ascii_lowercase().contains("trivy") {
            return Err(ReleaseSupplyChainError::InvalidTrivyRef {
                artifact_ref: record.artifact_ref.clone(),
                field,
            });
        }
    }
    Ok(())
}

fn is_sha256_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_git_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_semver(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts
            .into_iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

fn require_adr0039_evidence(evidence: SupplyChainEvidence) -> Result<(), SupplyChainError> {
    let release_scope_declared =
        evidence.release_images_declared || evidence.release_empty_scope_declared;
    for (present, missing_evidence) in [
        (
            evidence.release_manifest_present,
            "release_manifest_present",
        ),
        (
            release_scope_declared,
            "release_images_declared_or_empty_scope_rationale",
        ),
        (
            evidence.trivy_filesystem_scan_wired,
            "trivy_filesystem_scan_wired",
        ),
        (
            evidence.trivy_container_scan_wired,
            "trivy_container_scan_wired",
        ),
        (evidence.trivy_iac_scan_wired, "trivy_iac_scan_wired"),
        (
            evidence.trivy_dependency_scan_wired,
            "trivy_dependency_scan_wired",
        ),
        (
            evidence.cosign_release_signing_wired,
            "cosign_release_signing_wired",
        ),
        (
            evidence.cosign_rekor_verification_wired,
            "cosign_rekor_verification_wired",
        ),
        (evidence.sbom_spdx_wired, "sbom_spdx_wired"),
        (evidence.sbom_cyclonedx_wired, "sbom_cyclonedx_wired"),
        (
            evidence.provenance_attestation_wired,
            "provenance_attestation_wired",
        ),
        (
            evidence.signed_commit_policy_wired,
            "signed_commit_policy_wired",
        ),
        (evidence.admission_policy_wired, "admission_policy_wired"),
    ] {
        if !present {
            return Err(SupplyChainError::MissingAdr0039Evidence { missing_evidence });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_source_only_bootstrap_with_dependency_scans_wired() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                evidence()
            ),
            Ok(SupplyChainReport {
                records_checked: 1,
                source_only_records: 1,
            })
        );
    }

    #[test]
    fn rejects_missing_dependency_scan_wiring() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                SupplyChainEvidence {
                    cargo_audit_check_wired: false,
                    ..evidence()
                }
            ),
            Err(SupplyChainError::MissingCargoAuditCheck)
        );
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                SupplyChainEvidence {
                    cargo_deny_check_wired: false,
                    ..evidence()
                }
            ),
            Err(SupplyChainError::MissingCargoDenyCheck)
        );
    }

    #[test]
    fn rejects_unpinned_third_party_actions() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                SupplyChainEvidence {
                    third_party_actions_pinned: false,
                    ..evidence()
                }
            ),
            Err(SupplyChainError::UnpinnedThirdPartyAction)
        );
    }

    #[test]
    fn rejects_release_manifest_without_trivy_cosign_and_sbom_evidence() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                SupplyChainEvidence {
                    release_manifest_present: true,
                    ..evidence()
                }
            ),
            Err(SupplyChainError::ReleaseManifestWithoutAdr0039Evidence {
                missing_evidence: "trivy_release_scan_wired",
            })
        );
    }

    #[test]
    fn rejects_full_adr0039_lane_without_required_evidence() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                SupplyChainEvidence {
                    require_adr0039_evidence: true,
                    ..evidence()
                }
            ),
            Err(SupplyChainError::MissingAdr0039Evidence {
                missing_evidence: "release_manifest_present",
            })
        );
    }

    #[test]
    fn accepts_full_adr0039_lane_when_static_evidence_is_wired() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                full_adr0039_evidence()
            ),
            Ok(SupplyChainReport {
                records_checked: 1,
                source_only_records: 1,
            })
        );
    }

    #[test]
    fn accepts_full_adr0039_lane_with_explicit_pre_release_empty_scope() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                SupplyChainEvidence {
                    release_images_declared: false,
                    release_empty_scope_declared: true,
                    ..full_adr0039_evidence()
                }
            ),
            Ok(SupplyChainReport {
                records_checked: 1,
                source_only_records: 1,
            })
        );
    }

    #[test]
    fn rejects_full_adr0039_lane_without_release_artifacts_or_empty_scope_rationale() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                SupplyChainEvidence {
                    release_images_declared: false,
                    release_empty_scope_declared: false,
                    ..full_adr0039_evidence()
                }
            ),
            Err(SupplyChainError::MissingAdr0039Evidence {
                missing_evidence: "release_images_declared_or_empty_scope_rationale",
            })
        );
    }

    #[test]
    fn rejects_sbom_or_signed_claim_without_evidence() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "sbom")],
                evidence()
            ),
            Err(SupplyChainError::UnsupportedAttestationClaim {
                subject: "oya-intelligence-capability-kernel".into(),
                attestation: "sbom".into(),
                missing_evidence: "sbom_dual_format_wired",
            })
        );
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "signed-provenance")],
                SupplyChainEvidence {
                    sbom_dual_format_wired: true,
                    ..evidence()
                }
            ),
            Err(SupplyChainError::UnsupportedAttestationClaim {
                subject: "oya-intelligence-capability-kernel".into(),
                attestation: "signed-provenance".into(),
                missing_evidence: "cosign_release_signing_wired",
            })
        );
    }

    #[test]
    fn rejects_empty_catalog_records() {
        assert_eq!(
            validate_supply_chain([], evidence()),
            Err(SupplyChainError::NoCatalogRecords)
        );
    }

    #[test]
    fn accepts_release_supply_chain_evidence_for_every_artifact() {
        assert_eq!(
            validate_release_supply_chain([release_artifact()], [release_evidence()]),
            Ok(ReleaseSupplyChainReport {
                artifacts_checked: 1,
                evidence_records_checked: 1,
            })
        );
    }

    #[test]
    fn accepts_pre_release_supply_chain_empty_scope_only_when_declared() {
        assert_eq!(
            validate_pre_release_supply_chain(
                Vec::<ReleaseArtifact>::new(),
                Vec::<ReleaseSupplyChainEvidence>::new(),
                true
            ),
            Ok(ReleaseSupplyChainReport {
                artifacts_checked: 0,
                evidence_records_checked: 0,
            })
        );
        assert_eq!(
            validate_pre_release_supply_chain(
                Vec::<ReleaseArtifact>::new(),
                Vec::<ReleaseSupplyChainEvidence>::new(),
                false
            ),
            Err(ReleaseSupplyChainError::MissingPreReleaseEmptyScopeRationale)
        );
    }

    #[test]
    fn accepts_pre_release_artifacts_before_attestation_records_exist() {
        assert_eq!(
            validate_pre_release_supply_chain(
                [release_artifact()],
                Vec::<ReleaseSupplyChainEvidence>::new(),
                false
            ),
            Ok(ReleaseSupplyChainReport {
                artifacts_checked: 1,
                evidence_records_checked: 0,
            })
        );
    }

    #[test]
    fn pre_release_evidence_still_must_cover_every_artifact() {
        assert_eq!(
            validate_pre_release_supply_chain(
                [release_artifact()],
                [ReleaseSupplyChainEvidence {
                    artifact_ref: release_artifact_ref("other"),
                    ..release_evidence()
                }],
                false
            ),
            Err(ReleaseSupplyChainError::MissingEvidenceForArtifact {
                artifact_ref: release_artifact().artifact_ref,
            })
        );
    }

    #[test]
    fn rejects_release_artifact_without_evidence() {
        assert_eq!(
            validate_release_supply_chain([release_artifact()], []),
            Err(ReleaseSupplyChainError::NoEvidenceRecords)
        );
        assert_eq!(
            validate_release_supply_chain(
                [release_artifact()],
                [ReleaseSupplyChainEvidence {
                    artifact_ref: release_artifact_ref("other"),
                    ..release_evidence()
                }]
            ),
            Err(ReleaseSupplyChainError::MissingEvidenceForArtifact {
                artifact_ref: release_artifact().artifact_ref,
            })
        );
    }

    #[test]
    fn rejects_release_evidence_with_open_high_critical_findings() {
        assert_eq!(
            validate_release_supply_chain(
                [release_artifact()],
                [ReleaseSupplyChainEvidence {
                    high_critical_findings_open: 1,
                    ..release_evidence()
                }]
            ),
            Err(ReleaseSupplyChainError::OpenHighCriticalFindings {
                artifact_ref: release_artifact().artifact_ref,
                count: 1,
            })
        );
    }

    #[test]
    fn rejects_release_evidence_without_rekor_or_signature() {
        assert_eq!(
            validate_release_supply_chain(
                [release_artifact()],
                [ReleaseSupplyChainEvidence {
                    rekor_log_index: 0,
                    ..release_evidence()
                }]
            ),
            Err(ReleaseSupplyChainError::MissingRekorInclusion {
                artifact_ref: release_artifact().artifact_ref,
            })
        );
        assert_eq!(
            validate_release_supply_chain(
                [release_artifact()],
                [ReleaseSupplyChainEvidence {
                    signed: false,
                    ..release_evidence()
                }]
            ),
            Err(ReleaseSupplyChainError::UnsignedEvidence {
                artifact_ref: release_artifact().artifact_ref,
            })
        );
    }

    #[test]
    fn rejects_release_evidence_not_pinned_to_artifact_digest() {
        assert_eq!(
            validate_release_supply_chain(
                [release_artifact()],
                [ReleaseSupplyChainEvidence {
                    artifact_digest:
                        "sha256:fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"
                            .into(),
                    ..release_evidence()
                }]
            ),
            Err(ReleaseSupplyChainError::DigestNotPinnedInArtifactRef {
                artifact_ref: release_artifact().artifact_ref,
                artifact_digest:
                    "sha256:fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210".into(),
            })
        );
    }

    fn record(subject: &str, attestation: &str) -> SupplyChainRecord {
        SupplyChainRecord {
            subject: subject.into(),
            attestation: attestation.into(),
        }
    }

    fn release_artifact() -> ReleaseArtifact {
        ReleaseArtifact {
            artifact_ref: release_artifact_ref("tooling"),
        }
    }

    fn release_artifact_ref(name: &str) -> String {
        format!(
            "ghcr.io/oyatie/{name}@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        )
    }

    fn release_evidence() -> ReleaseSupplyChainEvidence {
        ReleaseSupplyChainEvidence {
            artifact_ref: release_artifact().artifact_ref,
            artifact_digest:
                "sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".into(),
            release_version: "0.1.0".into(),
            source_revision: "0123456789abcdef0123456789abcdef01234567".into(),
            sbom_spdx_ref: "artifact://release/0.1.0/tooling.spdx.json".into(),
            sbom_cyclonedx_ref: "artifact://release/0.1.0/tooling.cyclonedx.json".into(),
            cosign_signature_ref: "rekor://log/123/signature".into(),
            cosign_certificate_ref: "rekor://log/123/certificate".into(),
            rekor_log_index: 123,
            trivy_filesystem_scan_ref: "artifact://release/0.1.0/trivy-fs.sarif".into(),
            trivy_container_scan_ref: "artifact://release/0.1.0/trivy-image.sarif".into(),
            trivy_iac_scan_ref: "artifact://release/0.1.0/trivy-iac.sarif".into(),
            trivy_dependency_scan_ref: "artifact://release/0.1.0/trivy-dep.sarif".into(),
            provenance_attestation_ref: "artifact://release/0.1.0/provenance.intoto.jsonl".into(),
            audit_event_type: "oya.audit.builder_supply_attest".into(),
            attestor: "axis-foundry".into(),
            high_critical_findings_open: 0,
            signed: true,
        }
    }

    fn evidence() -> SupplyChainEvidence {
        SupplyChainEvidence {
            deny_config_present: true,
            cargo_deny_check_wired: true,
            cargo_audit_check_wired: true,
            third_party_actions_pinned: true,
            require_adr0039_evidence: false,
            release_manifest_present: false,
            release_images_declared: false,
            release_empty_scope_declared: false,
            trivy_release_scan_wired: false,
            trivy_filesystem_scan_wired: false,
            trivy_container_scan_wired: false,
            trivy_iac_scan_wired: false,
            trivy_dependency_scan_wired: false,
            cosign_release_signing_wired: false,
            cosign_rekor_verification_wired: false,
            sbom_dual_format_wired: false,
            sbom_spdx_wired: false,
            sbom_cyclonedx_wired: false,
            provenance_attestation_wired: false,
            signed_commit_policy_wired: false,
            admission_policy_wired: false,
        }
    }

    fn full_adr0039_evidence() -> SupplyChainEvidence {
        SupplyChainEvidence {
            require_adr0039_evidence: true,
            release_manifest_present: true,
            release_images_declared: true,
            release_empty_scope_declared: false,
            trivy_release_scan_wired: true,
            trivy_filesystem_scan_wired: true,
            trivy_container_scan_wired: true,
            trivy_iac_scan_wired: true,
            trivy_dependency_scan_wired: true,
            cosign_release_signing_wired: true,
            cosign_rekor_verification_wired: true,
            sbom_dual_format_wired: true,
            sbom_spdx_wired: true,
            sbom_cyclonedx_wired: true,
            provenance_attestation_wired: true,
            signed_commit_policy_wired: true,
            admission_policy_wired: true,
            ..evidence()
        }
    }
}
