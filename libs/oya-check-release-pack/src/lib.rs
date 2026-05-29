//! Release evidence-pack fitness kernel.
//!
//! COMPLIANCE-MATRIX requires per-regulator evidence pack regeneration within
//! four hours. The bootstrap state must be explicit, and any published release
//! evidence pack must be signed, mirrored, notified, and audit-chain anchored.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseEvidencePackManifest {
    pub release_version: String,       // data_class: INTERNAL_ONLY
    pub empty_scope_rationale: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComplianceRegulatorRef {
    pub regulator: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseEvidencePackRecord {
    pub regulator: String,                     // data_class: INTERNAL_ONLY
    pub region: String,                        // data_class: INTERNAL_ONLY
    pub pack_id: String,                       // data_class: INTERNAL_ONLY
    pub release_version: String,               // data_class: INTERNAL_ONLY
    pub audit_cycle: String,                   // data_class: INTERNAL_ONLY
    pub coverage_window_start: String,         // data_class: INTERNAL_ONLY
    pub coverage_window_end: String,           // data_class: INTERNAL_ONLY
    pub owner_team: String,                    // data_class: INTERNAL_ONLY
    pub evidence_pack_ref: String,             // data_class: INTERNAL_ONLY
    pub cosign_attestation_ref: String,        // data_class: INTERNAL_ONLY
    pub audit_event_id: String,                // data_class: INTERNAL_ONLY
    pub requested_at_epoch_minutes: u64,       // data_class: INTERNAL_ONLY
    pub regenerated_at_epoch_minutes: u64,     // data_class: INTERNAL_ONLY
    pub controls_mapped: u32,                  // data_class: INTERNAL_ONLY
    pub evidence_links: u32,                   // data_class: INTERNAL_ONLY
    pub trust_portal_mirror_regenerated: bool, // data_class: INTERNAL_ONLY
    pub regulator_notification_sent: bool,     // data_class: INTERNAL_ONLY
    pub status: String,                        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReleaseEvidencePackPolicy {
    pub max_regeneration_minutes: u64, // data_class: INTERNAL_ONLY
    pub require_records: bool,         // data_class: INTERNAL_ONLY
}

impl ReleaseEvidencePackPolicy {
    pub fn compliance_matrix_sla() -> Self {
        Self {
            max_regeneration_minutes: 240,
            require_records: false,
        }
    }

    pub fn release_blocking_sla() -> Self {
        Self {
            max_regeneration_minutes: 240,
            require_records: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseEvidencePackReport {
    pub known_regulators_checked: usize,  // data_class: INTERNAL_ONLY
    pub records_checked: usize,           // data_class: INTERNAL_ONLY
    pub published_records_checked: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReleaseEvidencePackError {
    InvalidPolicy,
    MissingReleaseVersion,
    NoKnownRegulators,
    InvalidKnownRegulator,
    MissingEmptyScopeRationale,
    RecordsRequired,
    DuplicateRecord {
        key: String,
    },
    MissingField {
        key: String,
        field: &'static str,
    },
    UnknownRegulator {
        key: String,
        regulator: String,
    },
    ReleaseVersionMismatch {
        key: String,
        expected: String,
        actual: String,
    },
    UnknownAuditCycle {
        key: String,
        audit_cycle: String,
    },
    UnknownStatus {
        key: String,
        status: String,
    },
    RegeneratedBeforeRequested {
        key: String,
    },
    RegenerationSlaExceeded {
        key: String,
        actual_minutes: u64,
        maximum_minutes: u64,
    },
    EmptyControlMapping {
        key: String,
    },
    EmptyEvidenceLinks {
        key: String,
    },
    EvidencePackRefInvalid {
        key: String,
        evidence_pack_ref: String,
    },
    CosignAttestationRefInvalid {
        key: String,
        cosign_attestation_ref: String,
    },
    AuditEventInvalid {
        key: String,
        audit_event_id: String,
    },
    TrustPortalMirrorMissing {
        key: String,
    },
    RegulatorNotificationMissing {
        key: String,
    },
}

pub fn validate_release_evidence_packs<R, K>(
    manifest: ReleaseEvidencePackManifest,
    records: R,
    known_regulators: K,
    policy: ReleaseEvidencePackPolicy,
) -> Result<ReleaseEvidencePackReport, ReleaseEvidencePackError>
where
    R: IntoIterator<Item = ReleaseEvidencePackRecord>,
    K: IntoIterator<Item = ComplianceRegulatorRef>,
{
    if policy.max_regeneration_minutes == 0 {
        return Err(ReleaseEvidencePackError::InvalidPolicy);
    }
    if !usable_ref(&manifest.release_version) {
        return Err(ReleaseEvidencePackError::MissingReleaseVersion);
    }

    let known_regulators = known_regulator_set(known_regulators)?;
    let records = records.into_iter().collect::<Vec<_>>();
    if records.is_empty() {
        if policy.require_records {
            return Err(ReleaseEvidencePackError::RecordsRequired);
        }
        if !usable_ref(&manifest.empty_scope_rationale) {
            return Err(ReleaseEvidencePackError::MissingEmptyScopeRationale);
        }
        return Ok(ReleaseEvidencePackReport {
            known_regulators_checked: known_regulators.len(),
            records_checked: 0,
            published_records_checked: 0,
        });
    }

    let mut seen = BTreeSet::new();
    let mut published_records_checked = 0usize;
    for record in &records {
        let key = record_key(record);
        validate_required_fields(record, &key)?;
        if !seen.insert(key.clone()) {
            return Err(ReleaseEvidencePackError::DuplicateRecord { key });
        }
        if !known_regulators.contains(record.regulator.trim()) {
            return Err(ReleaseEvidencePackError::UnknownRegulator {
                key,
                regulator: record.regulator.clone(),
            });
        }
        if record.release_version != manifest.release_version {
            return Err(ReleaseEvidencePackError::ReleaseVersionMismatch {
                key,
                expected: manifest.release_version.clone(),
                actual: record.release_version.clone(),
            });
        }
        if !valid_audit_cycle(&record.audit_cycle) {
            return Err(ReleaseEvidencePackError::UnknownAuditCycle {
                key,
                audit_cycle: record.audit_cycle.clone(),
            });
        }
        if record.status != "published" {
            return Err(ReleaseEvidencePackError::UnknownStatus {
                key,
                status: record.status.clone(),
            });
        }
        validate_timing(record, &key, policy)?;
        validate_evidence_shape(record, &key)?;
        published_records_checked += 1;
    }

    Ok(ReleaseEvidencePackReport {
        known_regulators_checked: known_regulators.len(),
        records_checked: records.len(),
        published_records_checked,
    })
}

fn known_regulator_set<K>(known_regulators: K) -> Result<BTreeSet<String>, ReleaseEvidencePackError>
where
    K: IntoIterator<Item = ComplianceRegulatorRef>,
{
    let mut regulators = BTreeSet::new();
    for regulator in known_regulators {
        let value = regulator.regulator.trim();
        if !usable_ref(value) {
            return Err(ReleaseEvidencePackError::InvalidKnownRegulator);
        }
        regulators.insert(value.to_string());
    }
    if regulators.is_empty() {
        Err(ReleaseEvidencePackError::NoKnownRegulators)
    } else {
        Ok(regulators)
    }
}

fn validate_required_fields(
    record: &ReleaseEvidencePackRecord,
    key: &str,
) -> Result<(), ReleaseEvidencePackError> {
    for (field, value) in [
        ("regulator", &record.regulator),
        ("region", &record.region),
        ("pack_id", &record.pack_id),
        ("release_version", &record.release_version),
        ("audit_cycle", &record.audit_cycle),
        ("coverage_window_start", &record.coverage_window_start),
        ("coverage_window_end", &record.coverage_window_end),
        ("owner_team", &record.owner_team),
        ("evidence_pack_ref", &record.evidence_pack_ref),
        ("cosign_attestation_ref", &record.cosign_attestation_ref),
        ("audit_event_id", &record.audit_event_id),
        ("status", &record.status),
    ] {
        if !usable_ref(value) {
            return Err(ReleaseEvidencePackError::MissingField {
                key: key.into(),
                field,
            });
        }
    }
    Ok(())
}

fn validate_timing(
    record: &ReleaseEvidencePackRecord,
    key: &str,
    policy: ReleaseEvidencePackPolicy,
) -> Result<(), ReleaseEvidencePackError> {
    if record.regenerated_at_epoch_minutes < record.requested_at_epoch_minutes {
        return Err(ReleaseEvidencePackError::RegeneratedBeforeRequested { key: key.into() });
    }
    let actual_minutes = record.regenerated_at_epoch_minutes - record.requested_at_epoch_minutes;
    if actual_minutes > policy.max_regeneration_minutes {
        return Err(ReleaseEvidencePackError::RegenerationSlaExceeded {
            key: key.into(),
            actual_minutes,
            maximum_minutes: policy.max_regeneration_minutes,
        });
    }
    Ok(())
}

fn validate_evidence_shape(
    record: &ReleaseEvidencePackRecord,
    key: &str,
) -> Result<(), ReleaseEvidencePackError> {
    if record.controls_mapped == 0 {
        return Err(ReleaseEvidencePackError::EmptyControlMapping { key: key.into() });
    }
    if record.evidence_links == 0 {
        return Err(ReleaseEvidencePackError::EmptyEvidenceLinks { key: key.into() });
    }
    if !valid_evidence_pack_ref(&record.evidence_pack_ref) {
        return Err(ReleaseEvidencePackError::EvidencePackRefInvalid {
            key: key.into(),
            evidence_pack_ref: record.evidence_pack_ref.clone(),
        });
    }
    if !valid_cosign_attestation_ref(&record.cosign_attestation_ref) {
        return Err(ReleaseEvidencePackError::CosignAttestationRefInvalid {
            key: key.into(),
            cosign_attestation_ref: record.cosign_attestation_ref.clone(),
        });
    }
    if !record
        .audit_event_id
        .starts_with("EVT-EVIDENCE-PACK-PUBLISHED")
    {
        return Err(ReleaseEvidencePackError::AuditEventInvalid {
            key: key.into(),
            audit_event_id: record.audit_event_id.clone(),
        });
    }
    if !record.trust_portal_mirror_regenerated {
        return Err(ReleaseEvidencePackError::TrustPortalMirrorMissing { key: key.into() });
    }
    if !record.regulator_notification_sent {
        return Err(ReleaseEvidencePackError::RegulatorNotificationMissing { key: key.into() });
    }
    Ok(())
}

fn record_key(record: &ReleaseEvidencePackRecord) -> String {
    format!(
        "{}:{}:{}",
        record.release_version.trim(),
        record.region.trim(),
        record.regulator.trim()
    )
}

fn valid_audit_cycle(value: &str) -> bool {
    matches!(
        value,
        "annual" | "per-audit" | "on-demand" | "per-incident" | "per-release"
    )
}

fn valid_evidence_pack_ref(value: &str) -> bool {
    value.starts_with("artifact://")
        || value.starts_with("trust://")
        || value.starts_with("audits/")
        || value.starts_with("https://trust.oyatie.com/")
}

fn valid_cosign_attestation_ref(value: &str) -> bool {
    value.starts_with("rekor://")
        || value.starts_with("cosign://")
        || value.starts_with("artifact://")
}

fn usable_ref(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && !matches!(value, "n/a" | "N/A" | "none" | "None" | "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_explicit_pre_release_empty_scope() {
        let report = validate_release_evidence_packs(
            manifest(),
            [],
            known_regulators(),
            ReleaseEvidencePackPolicy::compliance_matrix_sla(),
        )
        .expect("pre-release empty scope accepted");

        assert_eq!(report.known_regulators_checked, 2);
        assert_eq!(report.records_checked, 0);
    }

    #[test]
    fn rejects_empty_scope_when_release_requires_records() {
        assert_eq!(
            validate_release_evidence_packs(
                manifest(),
                [],
                known_regulators(),
                ReleaseEvidencePackPolicy::release_blocking_sla(),
            ),
            Err(ReleaseEvidencePackError::RecordsRequired)
        );
    }

    #[test]
    fn rejects_missing_known_regulators() {
        assert_eq!(
            validate_release_evidence_packs(
                manifest(),
                [],
                [],
                ReleaseEvidencePackPolicy::compliance_matrix_sla(),
            ),
            Err(ReleaseEvidencePackError::NoKnownRegulators)
        );
    }

    #[test]
    fn accepts_published_evidence_pack_within_four_hour_sla() {
        let report = validate_release_evidence_packs(
            release_manifest(),
            [published_record()],
            known_regulators(),
            ReleaseEvidencePackPolicy::compliance_matrix_sla(),
        )
        .expect("published pack accepted");

        assert_eq!(report.records_checked, 1);
        assert_eq!(report.published_records_checked, 1);
    }

    #[test]
    fn rejects_unknown_regulator() {
        let mut record = published_record();
        record.regulator = "UnknownRegulator".into();

        assert!(matches!(
            validate_release_evidence_packs(
                release_manifest(),
                [record],
                known_regulators(),
                ReleaseEvidencePackPolicy::compliance_matrix_sla(),
            ),
            Err(ReleaseEvidencePackError::UnknownRegulator { .. })
        ));
    }

    #[test]
    fn rejects_regeneration_over_four_hours() {
        let mut record = published_record();
        record.regenerated_at_epoch_minutes = record.requested_at_epoch_minutes + 241;

        assert!(matches!(
            validate_release_evidence_packs(
                release_manifest(),
                [record],
                known_regulators(),
                ReleaseEvidencePackPolicy::compliance_matrix_sla(),
            ),
            Err(ReleaseEvidencePackError::RegenerationSlaExceeded { .. })
        ));
    }

    #[test]
    fn rejects_unmirrored_or_unnotified_pack() {
        let mut record = published_record();
        record.trust_portal_mirror_regenerated = false;

        assert!(matches!(
            validate_release_evidence_packs(
                release_manifest(),
                [record],
                known_regulators(),
                ReleaseEvidencePackPolicy::compliance_matrix_sla(),
            ),
            Err(ReleaseEvidencePackError::TrustPortalMirrorMissing { .. })
        ));
    }

    #[test]
    fn rejects_empty_control_or_evidence_coverage() {
        let mut record = published_record();
        record.controls_mapped = 0;

        assert!(matches!(
            validate_release_evidence_packs(
                release_manifest(),
                [record],
                known_regulators(),
                ReleaseEvidencePackPolicy::compliance_matrix_sla(),
            ),
            Err(ReleaseEvidencePackError::EmptyControlMapping { .. })
        ));
    }

    fn manifest() -> ReleaseEvidencePackManifest {
        ReleaseEvidencePackManifest {
            release_version: "pre-release".into(),
            empty_scope_rationale:
                "No regulator-facing release evidence packs exist before a release candidate."
                    .into(),
        }
    }

    fn release_manifest() -> ReleaseEvidencePackManifest {
        ReleaseEvidencePackManifest {
            release_version: "0.1.0".into(),
            empty_scope_rationale: "n/a".into(),
        }
    }

    fn known_regulators() -> Vec<ComplianceRegulatorRef> {
        vec![
            ComplianceRegulatorRef {
                regulator: "KR PIPA".into(),
            },
            ComplianceRegulatorRef {
                regulator: "GDPR".into(),
            },
        ]
    }

    fn published_record() -> ReleaseEvidencePackRecord {
        ReleaseEvidencePackRecord {
            regulator: "GDPR".into(),
            region: "eu".into(),
            pack_id: "oya-pack-eu".into(),
            release_version: "0.1.0".into(),
            audit_cycle: "per-release".into(),
            coverage_window_start: "2026-05-01".into(),
            coverage_window_end: "2026-05-10".into(),
            owner_team: "ops-compliance".into(),
            evidence_pack_ref: "artifact://release/0.1.0/evidence/gdpr.md".into(),
            cosign_attestation_ref: "rekor://log/123/evidence-pack".into(),
            audit_event_id: "EVT-EVIDENCE-PACK-PUBLISHED-0001".into(),
            requested_at_epoch_minutes: 1000,
            regenerated_at_epoch_minutes: 1240,
            controls_mapped: 8,
            evidence_links: 8,
            trust_portal_mirror_regenerated: true,
            regulator_notification_sent: true,
            status: "published".into(),
        }
    }
}
