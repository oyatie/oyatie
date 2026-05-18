//! Ontology-projection coverage check (ADR-0145 Invariant 3).
//!
//! # Why this crate exists
//!
//! ADR-0145 Invariant 3 requires µservices that own canonical entities
//! (Person, Task, Document, Recording, etc.) to project them into
//! Ontology. The per-µservice `manifest.json` declares the
//! projections via the `ontology_projections: [{entity_name,
//! projection_target_table}]` block (see
//! `specs/microservices/manifest-schema.json`).
//!
//! # Skeleton scope
//!
//! Advisory mode only. Strict mode is `unimplemented!()` pending the
//! follow-up `adr-0145-ontology-projection-validator`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

/// One supplied microservice manifest (`microservices/<ms>/manifest.json`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestDocument {
    pub path: String,
    pub microservice: String,
    pub contents: String,
}

/// Canonical-entity-owning µservices that ADR-0145 Invariant 3
/// explicitly requires to project. The list is hand-maintained in the
/// kernel (closed set) until the strict-mode parser cross-checks
/// against `registry/ontology/entities.json`.
pub const CANONICAL_ENTITY_OWNERS: &[&str] = &[
    "ontology",
    "tenancy",
    "audit-chain",
    "foundry",
    "governance",
    "tasks",
    "calendar",
    "drive",
    "mail",
    "meet",
    "recordings",
    "network",
    "messenger",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryReport {
    pub manifests_checked: usize,
    pub manifests_with_projections: usize,
    pub manifests_owning_entities: usize,
    pub advisory_findings: Vec<OntologyProjectionFinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OntologyProjectionFinding {
    pub manifest_path: String,
    pub microservice: String,
    pub summary: String,
}

impl fmt::Display for OntologyProjectionFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}): {}",
            self.microservice, self.manifest_path, self.summary
        )
    }
}

/// Advisory entrypoint — returns the report without erroring.
pub fn validate_advisory<I>(manifests: I) -> AdvisoryReport
where
    I: IntoIterator<Item = ManifestDocument>,
{
    let manifests: Vec<ManifestDocument> = manifests.into_iter().collect();
    let mut findings = Vec::new();
    let mut with_projections = 0usize;
    let mut owning = 0usize;

    for manifest in &manifests {
        let owns = CANONICAL_ENTITY_OWNERS
            .iter()
            .any(|ms| *ms == manifest.microservice);
        if owns {
            owning += 1;
        }
        let has_field = manifest_declares_projections(&manifest.contents);
        if has_field {
            with_projections += 1;
            // Owners with empty projections list are advisory findings:
            // the field is present but no concrete entity is declared.
            if owns && manifest_has_empty_projections(&manifest.contents) {
                findings.push(OntologyProjectionFinding {
                    manifest_path: manifest.path.clone(),
                    microservice: manifest.microservice.clone(),
                    summary: "owns canonical entities but ontology_projections is empty (ADR-0145 Invariant 3)".into(),
                });
            }
        } else if owns {
            findings.push(OntologyProjectionFinding {
                manifest_path: manifest.path.clone(),
                microservice: manifest.microservice.clone(),
                summary: "owns canonical entities but manifest does not declare ontology_projections (ADR-0145 Invariant 3)".into(),
            });
        }
    }

    AdvisoryReport {
        manifests_checked: manifests.len(),
        manifests_with_projections: with_projections,
        manifests_owning_entities: owning,
        advisory_findings: findings,
    }
}

/// Strict mode. `unimplemented!()` until the follow-up authors a real
/// JSON parser path.
pub fn validate_strict<I>(_manifests: I) -> !
where
    I: IntoIterator<Item = ManifestDocument>,
{
    unimplemented!(
        "oya-check-ontology-projection-coverage: strict-mode validator not yet implemented; \
         tracked under registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-ontology-projection-validator"
    )
}

fn manifest_declares_projections(contents: &str) -> bool {
    contents.contains("\"ontology_projections\"")
}

fn manifest_has_empty_projections(contents: &str) -> bool {
    // Tolerant scan — checks for the literal empty-array form. The
    // strict-mode parser will use a real JSON tokenizer.
    let normalized: String = contents.chars().filter(|c| !c.is_whitespace()).collect();
    normalized.contains("\"ontology_projections\":[]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advisory_passes_when_owner_declares_non_empty_projections() {
        let manifest = ManifestDocument {
            path: "microservices/ontology/manifest.json".into(),
            microservice: "ontology".into(),
            contents: r#"{
  "ontology_projections": [
    {"entity_name": "Person", "projection_target_table": "ontology_persons"}
  ]
}"#
            .into(),
        };
        let report = validate_advisory(vec![manifest]);
        assert_eq!(report.manifests_checked, 1);
        assert_eq!(report.manifests_with_projections, 1);
        assert!(report.advisory_findings.is_empty());
    }

    #[test]
    fn advisory_finds_owner_with_empty_projections() {
        let manifest = ManifestDocument {
            path: "microservices/tasks/manifest.json".into(),
            microservice: "tasks".into(),
            contents: r#"{ "ontology_projections": [] }"#.into(),
        };
        let report = validate_advisory(vec![manifest]);
        assert_eq!(report.advisory_findings.len(), 1);
    }

    #[test]
    fn advisory_finds_owner_missing_field_entirely() {
        let manifest = ManifestDocument {
            path: "microservices/network/manifest.json".into(),
            microservice: "network".into(),
            contents: r#"{ "microservice": "network" }"#.into(),
        };
        let report = validate_advisory(vec![manifest]);
        assert_eq!(report.advisory_findings.len(), 1);
    }

    #[test]
    fn advisory_skips_non_owner_microservice() {
        let manifest = ManifestDocument {
            path: "microservices/community/manifest.json".into(),
            microservice: "community".into(),
            contents: r#"{ "microservice": "community" }"#.into(),
        };
        let report = validate_advisory(vec![manifest]);
        assert!(report.advisory_findings.is_empty());
    }

    #[test]
    #[should_panic(expected = "strict-mode validator not yet implemented")]
    fn strict_mode_panics_until_authored() {
        validate_strict(Vec::<ManifestDocument>::new());
    }

    #[test]
    fn canonical_entity_owners_non_empty() {
        assert!(!CANONICAL_ENTITY_OWNERS.is_empty());
    }
}
