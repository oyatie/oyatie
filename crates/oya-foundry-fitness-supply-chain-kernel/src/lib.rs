//! Supply-chain fitness kernel — Cosign + Rekor + SBOM attestation
//! checks. Per M-CC-P08-IP-001: every shipped artifact must carry a
//! valid Cosign signature and a Rekor transparency-log entry.
//!
//! I/O-free. Runners verify signatures / fetch Rekor entries / parse
//! SBOMs outside the kernel, then feed typed [`ArtifactAttestation`]
//! records into [`check_signed`].
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// A built artifact under supply-chain review.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactAttestation {
    pub artifact_id: String,                       // data_class: INTERNAL_ONLY
    pub digest_sha256: String,                     // data_class: INTERNAL_ONLY
    pub cosign_signature: Option<CosignSignature>, // data_class: INTERNAL_ONLY
    pub rekor_entry: Option<RekorEntry>,           // data_class: INTERNAL_ONLY
    pub sbom_present: bool,                        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CosignSignature {
    pub key_id: String,         // data_class: INTERNAL_ONLY
    pub verified: bool,         // data_class: INTERNAL_ONLY
    pub signed_at_unix_ms: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RekorEntry {
    pub log_index: u64,             // data_class: INTERNAL_ONLY
    pub log_id: String,             // data_class: INTERNAL_ONLY
    pub integrated_at_unix_ms: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SupplyChainViolationKind {
    MissingCosignSignature,
    CosignVerificationFailed,
    MissingRekorEntry,
    MissingSbom,
    EmptyDigest,
    DigestNotSha256Hex,
}

impl SupplyChainViolationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingCosignSignature => "missing cosign signature",
            Self::CosignVerificationFailed => "cosign signature failed verification",
            Self::MissingRekorEntry => "missing rekor transparency-log entry",
            Self::MissingSbom => "missing SBOM",
            Self::EmptyDigest => "artifact digest is empty",
            Self::DigestNotSha256Hex => "digest is not 64-char lowercase sha256 hex",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplyChainViolation {
    pub artifact_id: String,            // data_class: INTERNAL_ONLY
    pub kind: SupplyChainViolationKind, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplyChainReport {
    pub artifacts_checked: usize,              // data_class: INTERNAL_ONLY
    pub violations: Vec<SupplyChainViolation>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupplyChainError {
    EmptyArtifactId,
    DuplicateArtifact { artifact_id: String },
}

impl SupplyChainError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyArtifactId => "artifact_id is empty".to_owned(),
            Self::DuplicateArtifact { artifact_id } => {
                format!("duplicate artifact: {artifact_id}")
            }
        }
    }
}

fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

/// Verify each artifact carries Cosign + Rekor + SBOM proof. Returns
/// per-artifact violations or an error for malformed input.
pub fn check_signed(
    artifacts: &[ArtifactAttestation],
) -> Result<SupplyChainReport, SupplyChainError> {
    let mut seen = std::collections::BTreeSet::new();
    let mut violations = Vec::new();

    for a in artifacts {
        if a.artifact_id.is_empty() {
            return Err(SupplyChainError::EmptyArtifactId);
        }
        if !seen.insert(a.artifact_id.as_str()) {
            return Err(SupplyChainError::DuplicateArtifact {
                artifact_id: a.artifact_id.clone(),
            });
        }

        if a.digest_sha256.is_empty() {
            violations.push(SupplyChainViolation {
                artifact_id: a.artifact_id.clone(),
                kind: SupplyChainViolationKind::EmptyDigest,
            });
        } else if !is_sha256_hex(&a.digest_sha256) {
            violations.push(SupplyChainViolation {
                artifact_id: a.artifact_id.clone(),
                kind: SupplyChainViolationKind::DigestNotSha256Hex,
            });
        }

        match &a.cosign_signature {
            None => violations.push(SupplyChainViolation {
                artifact_id: a.artifact_id.clone(),
                kind: SupplyChainViolationKind::MissingCosignSignature,
            }),
            Some(sig) if !sig.verified => violations.push(SupplyChainViolation {
                artifact_id: a.artifact_id.clone(),
                kind: SupplyChainViolationKind::CosignVerificationFailed,
            }),
            _ => {}
        }

        if a.rekor_entry.is_none() {
            violations.push(SupplyChainViolation {
                artifact_id: a.artifact_id.clone(),
                kind: SupplyChainViolationKind::MissingRekorEntry,
            });
        }

        if !a.sbom_present {
            violations.push(SupplyChainViolation {
                artifact_id: a.artifact_id.clone(),
                kind: SupplyChainViolationKind::MissingSbom,
            });
        }
    }

    Ok(SupplyChainReport {
        artifacts_checked: artifacts.len(),
        violations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(verified: bool) -> CosignSignature {
        CosignSignature {
            key_id: "kid-1".into(),
            verified,
            signed_at_unix_ms: 1000,
        }
    }
    fn rekor() -> RekorEntry {
        RekorEntry {
            log_index: 42,
            log_id: "log-1".into(),
            integrated_at_unix_ms: 1001,
        }
    }
    fn good_digest() -> String {
        "a".repeat(64)
    }
    fn good_artifact() -> ArtifactAttestation {
        ArtifactAttestation {
            artifact_id: "art-1".into(),
            digest_sha256: good_digest(),
            cosign_signature: Some(sig(true)),
            rekor_entry: Some(rekor()),
            sbom_present: true,
        }
    }

    #[test]
    fn fully_signed_artifact_passes() {
        let r = check_signed(&[good_artifact()]).unwrap();
        assert!(r.violations.is_empty(), "{:?}", r.violations);
    }

    #[test]
    fn missing_cosign_flagged() {
        let mut a = good_artifact();
        a.cosign_signature = None;
        let r = check_signed(&[a]).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == SupplyChainViolationKind::MissingCosignSignature)
        );
    }

    #[test]
    fn unverified_cosign_flagged() {
        let mut a = good_artifact();
        a.cosign_signature = Some(sig(false));
        let r = check_signed(&[a]).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == SupplyChainViolationKind::CosignVerificationFailed)
        );
    }

    #[test]
    fn missing_rekor_flagged() {
        let mut a = good_artifact();
        a.rekor_entry = None;
        let r = check_signed(&[a]).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == SupplyChainViolationKind::MissingRekorEntry)
        );
    }

    #[test]
    fn missing_sbom_flagged() {
        let mut a = good_artifact();
        a.sbom_present = false;
        let r = check_signed(&[a]).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == SupplyChainViolationKind::MissingSbom)
        );
    }

    #[test]
    fn empty_digest_flagged() {
        let mut a = good_artifact();
        a.digest_sha256 = String::new();
        let r = check_signed(&[a]).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == SupplyChainViolationKind::EmptyDigest)
        );
    }

    #[test]
    fn short_digest_flagged() {
        let mut a = good_artifact();
        a.digest_sha256 = "deadbeef".into();
        let r = check_signed(&[a]).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == SupplyChainViolationKind::DigestNotSha256Hex)
        );
    }

    #[test]
    fn uppercase_digest_flagged() {
        let mut a = good_artifact();
        a.digest_sha256 = "A".repeat(64);
        let r = check_signed(&[a]).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == SupplyChainViolationKind::DigestNotSha256Hex)
        );
    }

    #[test]
    fn nonhex_digest_flagged() {
        let mut a = good_artifact();
        a.digest_sha256 = "z".repeat(64);
        let r = check_signed(&[a]).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == SupplyChainViolationKind::DigestNotSha256Hex)
        );
    }

    #[test]
    fn empty_artifact_id_errors() {
        let mut a = good_artifact();
        a.artifact_id = String::new();
        let err = check_signed(&[a]).unwrap_err();
        assert!(matches!(err, SupplyChainError::EmptyArtifactId));
    }

    #[test]
    fn duplicate_artifact_errors() {
        let err = check_signed(&[good_artifact(), good_artifact()]).unwrap_err();
        assert!(matches!(err, SupplyChainError::DuplicateArtifact { .. }));
    }

    #[test]
    fn multiple_artifacts_aggregate() {
        let mut a1 = good_artifact();
        a1.artifact_id = "a1".into();
        let mut a2 = good_artifact();
        a2.artifact_id = "a2".into();
        a2.cosign_signature = None;
        let mut a3 = good_artifact();
        a3.artifact_id = "a3".into();
        a3.rekor_entry = None;
        a3.sbom_present = false;
        let r = check_signed(&[a1, a2, a3]).unwrap();
        assert_eq!(r.artifacts_checked, 3);
        assert_eq!(r.violations.len(), 3);
    }

    #[test]
    fn missing_three_signals_yields_three_violations() {
        let mut a = good_artifact();
        a.cosign_signature = None;
        a.rekor_entry = None;
        a.sbom_present = false;
        let r = check_signed(&[a]).unwrap();
        assert_eq!(r.violations.len(), 3);
    }

    #[test]
    fn violation_kind_as_str_distinct() {
        let kinds = [
            SupplyChainViolationKind::MissingCosignSignature,
            SupplyChainViolationKind::CosignVerificationFailed,
            SupplyChainViolationKind::MissingRekorEntry,
            SupplyChainViolationKind::MissingSbom,
            SupplyChainViolationKind::EmptyDigest,
            SupplyChainViolationKind::DigestNotSha256Hex,
        ];
        let names: std::collections::HashSet<_> = kinds.iter().map(|k| k.as_str()).collect();
        assert_eq!(names.len(), kinds.len());
    }
}
