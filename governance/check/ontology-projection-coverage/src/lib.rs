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
//! # Gate scope
//!
//! Advisory mode preserves the original report-only rollout surface.
//! Strict mode parses each manifest as JSON and fails closed for
//! canonical-entity owners that omit concrete projection declarations.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fmt;

use serde::Deserialize;

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
pub struct StrictReport {
    pub manifests_checked: usize,
    pub manifests_with_projections: usize,
    pub manifests_owning_entities: usize,
    pub projections_checked: usize,
    pub strict_findings: Vec<OntologyProjectionFinding>,
}

impl StrictReport {
    pub fn is_success(&self) -> bool {
        self.strict_findings.is_empty()
    }
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

/// Strict entrypoint. Canonical-entity owners must declare at least one
/// concrete projection, and each projection must name a non-empty
/// entity plus target table. This intentionally stops short of the
/// future registry/ontology/entities.json authority-source cross-check;
/// that broader registry-GA validator remains tracked separately.
pub fn validate_strict<I>(manifests: I) -> StrictReport
where
    I: IntoIterator<Item = ManifestDocument>,
{
    let manifests: Vec<ManifestDocument> = manifests.into_iter().collect();
    let mut findings = Vec::new();
    let mut with_projections = 0usize;
    let mut owning = 0usize;
    let mut projections_checked = 0usize;

    for manifest in &manifests {
        let owns = owns_canonical_entities(&manifest.microservice);
        if owns {
            owning += 1;
        }

        let parsed = match serde_json::from_str::<ManifestJson>(&manifest.contents) {
            Ok(parsed) => parsed,
            Err(err) => {
                findings.push(OntologyProjectionFinding {
                    manifest_path: manifest.path.clone(),
                    microservice: manifest.microservice.clone(),
                    summary: format!("manifest JSON parse failed: {err}"),
                });
                continue;
            }
        };

        let Some(projections) = parsed.ontology_projections else {
            if owns {
                findings.push(OntologyProjectionFinding {
                    manifest_path: manifest.path.clone(),
                    microservice: manifest.microservice.clone(),
                    summary: "owns canonical entities but manifest does not declare ontology_projections (ADR-0145 Invariant 3)".into(),
                });
            }
            continue;
        };

        if !projections.is_empty() {
            with_projections += 1;
        }

        if owns && projections.is_empty() {
            findings.push(OntologyProjectionFinding {
                manifest_path: manifest.path.clone(),
                microservice: manifest.microservice.clone(),
                summary: "owns canonical entities but ontology_projections is empty (ADR-0145 Invariant 3)".into(),
            });
        }

        let mut seen_entities = BTreeSet::<String>::new();
        for projection in projections {
            projections_checked += 1;
            let entity = projection.entity_name.trim();
            let target = projection.projection_target_table.trim();
            if entity.is_empty() {
                findings.push(OntologyProjectionFinding {
                    manifest_path: manifest.path.clone(),
                    microservice: manifest.microservice.clone(),
                    summary: "ontology_projections entry has empty entity_name".into(),
                });
            }
            if target.is_empty() {
                findings.push(OntologyProjectionFinding {
                    manifest_path: manifest.path.clone(),
                    microservice: manifest.microservice.clone(),
                    summary: format!(
                        "ontology_projections entry for {entity:?} has empty projection_target_table"
                    ),
                });
            }
            if !entity.is_empty() && !seen_entities.insert(entity.to_string()) {
                findings.push(OntologyProjectionFinding {
                    manifest_path: manifest.path.clone(),
                    microservice: manifest.microservice.clone(),
                    summary: format!("duplicate ontology projection entity_name {entity:?}"),
                });
            }
        }
    }

    StrictReport {
        manifests_checked: manifests.len(),
        manifests_with_projections: with_projections,
        manifests_owning_entities: owning,
        projections_checked,
        strict_findings: findings,
    }
}

#[derive(Debug, Deserialize)]
struct ManifestJson {
    ontology_projections: Option<Vec<OntologyProjectionJson>>,
}

#[derive(Debug, Deserialize)]
struct OntologyProjectionJson {
    entity_name: String,
    projection_target_table: String,
}

fn owns_canonical_entities(microservice: &str) -> bool {
    CANONICAL_ENTITY_OWNERS.contains(&microservice)
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
    fn canonical_entity_owners_non_empty() {
        assert!(!CANONICAL_ENTITY_OWNERS.is_empty());
    }

    #[test]
    fn strict_passes_when_owner_declares_non_empty_projection() {
        let manifest = ManifestDocument {
            path: "microservices/tasks/manifest.json".into(),
            microservice: "tasks".into(),
            contents: r#"{
  "ontology_projections": [
    {"entity_name": "Task", "projection_target_table": "ontology_tasks"}
  ]
}"#
            .into(),
        };

        let report = validate_strict(vec![manifest]);

        assert!(report.is_success());
        assert_eq!(report.projections_checked, 1);
        assert_eq!(report.manifests_owning_entities, 1);
    }

    #[test]
    fn strict_fails_owner_with_empty_projection_list() {
        let manifest = ManifestDocument {
            path: "microservices/tasks/manifest.json".into(),
            microservice: "tasks".into(),
            contents: r#"{ "ontology_projections": [] }"#.into(),
        };

        let report = validate_strict(vec![manifest]);

        assert!(!report.is_success());
        assert_eq!(report.strict_findings.len(), 1);
    }

    #[test]
    fn strict_fails_projection_missing_target_table() {
        let manifest = ManifestDocument {
            path: "microservices/tasks/manifest.json".into(),
            microservice: "tasks".into(),
            contents: r#"{
  "ontology_projections": [
    {"entity_name": "Task", "projection_target_table": ""}
  ]
}"#
            .into(),
        };

        let report = validate_strict(vec![manifest]);

        assert!(!report.is_success());
        assert_eq!(report.strict_findings.len(), 1);
    }

    #[test]
    fn strict_fails_duplicate_entity_names() {
        let manifest = ManifestDocument {
            path: "microservices/tasks/manifest.json".into(),
            microservice: "tasks".into(),
            contents: r#"{
  "ontology_projections": [
    {"entity_name": "Task", "projection_target_table": "ontology_tasks"},
    {"entity_name": "Task", "projection_target_table": "ontology_tasks_shadow"}
  ]
}"#
            .into(),
        };

        let report = validate_strict(vec![manifest]);

        assert!(!report.is_success());
        assert_eq!(report.strict_findings.len(), 1);
    }

    #[test]
    fn strict_skips_non_owner_with_empty_projection_list() {
        let manifest = ManifestDocument {
            path: "microservices/community/manifest.json".into(),
            microservice: "community".into(),
            contents: r#"{ "ontology_projections": [] }"#.into(),
        };

        let report = validate_strict(vec![manifest]);

        assert!(report.is_success());
    }
}
