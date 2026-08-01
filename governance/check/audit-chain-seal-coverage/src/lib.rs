//! Audit-chain seal coverage check (ADR-0145 Invariant 1).
//!
//! # Why this crate exists
//!
//! ADR-0145 Invariant 1 requires every state-changing capability to
//! emit a seal at the CALLING µservice. The per-µservice
//! `manifest.json#audit_chain.seal_events` field carries the
//! declarations; this kernel grounds the claim by inspecting the
//! manifests and surfacing capabilities without a matching seal event.
//!
//! # Skeleton scope
//!
//! Advisory mode only. Strict mode is `unimplemented!()` pending the
//! follow-up `adr-0145-audit-chain-seal-validator`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestDocument {
    pub path: String,
    pub microservice: String,
    pub contents: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryReport {
    pub manifests_checked: usize,
    pub manifests_with_audit_enabled: usize,
    pub manifests_with_seal_events: usize,
    pub advisory_findings: Vec<SealCoverageFinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SealCoverageFinding {
    pub manifest_path: String,
    pub microservice: String,
    pub summary: String,
}

impl fmt::Display for SealCoverageFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}): {}",
            self.microservice, self.manifest_path, self.summary
        )
    }
}

pub fn validate_advisory<I>(manifests: I) -> AdvisoryReport
where
    I: IntoIterator<Item = ManifestDocument>,
{
    let manifests: Vec<ManifestDocument> = manifests.into_iter().collect();
    let mut findings = Vec::new();
    let mut audit_enabled = 0usize;
    let mut with_seals = 0usize;

    for manifest in &manifests {
        let enabled = manifest_audit_enabled(&manifest.contents);
        let has_seal = manifest_has_seal_events(&manifest.contents);
        if enabled {
            audit_enabled += 1;
        }
        if has_seal {
            with_seals += 1;
        }
        if enabled && !has_seal {
            findings.push(SealCoverageFinding {
                manifest_path: manifest.path.clone(),
                microservice: manifest.microservice.clone(),
                summary: "audit_chain.enabled=true but seal_events is empty (ADR-0145 Invariant 1)"
                    .into(),
            });
        }
    }

    AdvisoryReport {
        manifests_checked: manifests.len(),
        manifests_with_audit_enabled: audit_enabled,
        manifests_with_seal_events: with_seals,
        advisory_findings: findings,
    }
}

pub fn validate_strict<I>(_manifests: I) -> !
where
    I: IntoIterator<Item = ManifestDocument>,
{
    unimplemented!(
        "check-audit-chain-seal-coverage: strict-mode validator not yet implemented; \
         tracked under registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-audit-chain-seal-validator"
    )
}

fn manifest_audit_enabled(contents: &str) -> bool {
    let normalized: String = contents.chars().filter(|c| !c.is_whitespace()).collect();
    normalized.contains("\"audit_chain\":{\"enabled\":true")
        || normalized.contains("\"enabled\":true,\"seal_events\"")
}

fn manifest_has_seal_events(contents: &str) -> bool {
    let normalized: String = contents.chars().filter(|c| !c.is_whitespace()).collect();
    if !normalized.contains("\"seal_events\"") {
        return false;
    }
    // Look for non-empty array
    !normalized.contains("\"seal_events\":[]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advisory_passes_when_seal_events_populated() {
        let manifest = ManifestDocument {
            path: "microservices/audit-chain/manifest.json".into(),
            microservice: "audit-chain".into(),
            contents: r#"{
  "audit_chain": { "enabled": true, "seal_events": ["sealed.v1"] }
}"#
            .into(),
        };
        let report = validate_advisory(vec![manifest]);
        assert_eq!(report.manifests_with_seal_events, 1);
        assert!(report.advisory_findings.is_empty());
    }

    #[test]
    fn advisory_finds_enabled_but_empty_seals() {
        let manifest = ManifestDocument {
            path: "microservices/tasks/manifest.json".into(),
            microservice: "tasks".into(),
            contents: r#"{
  "audit_chain": { "enabled": true, "seal_events": [] }
}"#
            .into(),
        };
        let report = validate_advisory(vec![manifest]);
        assert_eq!(report.advisory_findings.len(), 1);
    }

    #[test]
    fn advisory_skips_disabled_audit() {
        let manifest = ManifestDocument {
            path: "microservices/docs/manifest.json".into(),
            microservice: "docs".into(),
            contents: r#"{
  "audit_chain": { "enabled": false, "seal_events": [] }
}"#
            .into(),
        };
        let report = validate_advisory(vec![manifest]);
        assert!(report.advisory_findings.is_empty());
    }

    #[test]
    #[should_panic(expected = "strict-mode validator not yet implemented")]
    fn strict_mode_panics_until_authored() {
        validate_strict(Vec::<ManifestDocument>::new());
    }
}
