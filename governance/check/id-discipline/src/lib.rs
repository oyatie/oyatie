//! Id-discipline validator — ADR-0709 D-1 enforcement.
//!
//! Every canonical id surface — event ids, audit-chain row ids, changeset ids,
//! tenant ids, cell ids, principal ids, resource ids, request ids — MUST be
//! UUIDv7. A declaration that pins no version admits UUID v4, which carries no
//! time ordering and is precisely what D-1 exists to forbid, so a bare
//! `format: uuid` is a finding rather than a pass.
//!
//! This crate previously enforced the OPPOSITE rule: it accepted `format: ulid`
//! and flagged `uuid` as the violation, citing ADR-0156 — which is the PII
//! registry decision and says nothing about identifiers. ADR-0350 (superseded by
//! ADR-0709) already carried "update id-discipline from ULID canonical to
//! UUIDv7" as an unfinished follow-up. This is that follow-up.
//!
//! Kernel-tier (ADR-0083); no I/O. The caller supplies documents and policy; the
//! live corpus walk lives in `tests/live_corpus.rs`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

/// A canonical id field whose declared format does not pin UUIDv7.
pub const CODE_UNDERSPECIFIED_FORMAT: &str = "ID-UNDERSPECIFIED-FORMAT";
/// A frozen entry that matches no live declaration — the baseline must shrink
/// in the same change that removes the violation.
pub const CODE_STALE_BASELINE: &str = "ID-STALE-BASELINE";
/// The walk collapsed. Never coverage; always a gate failure.
pub const CODE_EMPTY_SCAN: &str = "ID-EMPTY-SCAN";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaDocument {
    pub path: String,
    pub microservice: String,
    pub contents: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    pub documents_checked: usize,
    pub id_fields_inspected: usize,
    pub id_fields_with_declared_format: usize,
    pub microservices_audited: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Finding {
    pub code: String,
    pub path: String,
    pub field: String,
    pub line: usize,
    pub message: String,
}

/// A tolerated non-UUIDv7 declaration, keyed by `path` + `field`.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FrozenEntry {
    pub path: String,
    pub field: String,
    pub declared: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Policy {
    pub canonical_id_fields: BTreeSet<String>,
    pub uuidv7_pattern: String,
    pub frozen: BTreeSet<FrozenEntry>,
    pub min_expected_scanned_files: usize,
    pub min_expected_id_fields: usize,
}

/// A declared format/pattern that pins UUIDv7.
///
/// Accepts either the explicit version-bit regex (the shape
/// `specs/residency-attestation-schema.json` already uses) or an explicit
/// `uuidv7` / `uuid-v7` token. A bare `uuid` does NOT pin a version and so does
/// not satisfy D-1.
#[must_use]
pub fn declaration_pins_uuidv7(declared: &str, uuidv7_pattern: &str) -> bool {
    let normalized = declared.trim().trim_matches('"').to_ascii_lowercase();
    if normalized == uuidv7_pattern.to_ascii_lowercase() {
        return true;
    }
    normalized.replace(['-', '_', ' '], "") == "uuidv7"
}

/// True when a declaration *claims to be a UUID-family identifier* — either an
/// explicit scheme token or a regex carrying the UUID skeleton.
///
/// A free-form `pattern` is deliberately NOT in scope. The tree models several
/// ids as domain-specific slugs on purpose: `contracts/workflow_spec.v1.json`
/// pins `tenant_id` to `^ten_[a-zA-Z0-9_.-]+$` and `specs/tenant-model.json`
/// pins it to a hierarchical `oyatie.<env>.<name>` label. Those are designs, not
/// drift, and a gate that flagged them would be reporting its own rule rather
/// than a defect. Judging only UUID-shaped claims also side-steps the proximity
/// limitation below: the collector takes the nearest following `format`/`pattern`
/// line, which in JSON can belong to a sibling property, so a narrow predicate
/// is what keeps a mis-attributed neighbour from becoming a false finding.
#[must_use]
fn is_identifier_declaration(declared: &str) -> bool {
    let n = declared.trim().trim_matches('"').to_ascii_lowercase();
    if n.contains("uuid") || n.contains("ulid") {
        return true;
    }
    // A regex is only an identifier claim if it carries the UUID skeleton.
    n.starts_with('^') && n.contains("[0-9a-f]{8}")
}

/// Extract `(field, line, declared)` for every canonical id field in one
/// document that declares a `format` or `pattern` within the following lines.
fn declarations(doc: &SchemaDocument, policy: &Policy) -> Vec<(String, usize, Option<String>)> {
    let lines: Vec<&str> = doc.contents.lines().collect();
    let mut out = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        for id_field in &policy.canonical_id_fields {
            let yaml_key = format!("{id_field}:");
            let json_key = format!("\"{id_field}\"");
            if !trimmed.starts_with(&yaml_key) && !trimmed.starts_with(&json_key) {
                continue;
            }
            let mut declared: Option<String> = None;
            for next in lines.iter().take(idx + 9).skip(idx + 1) {
                let nt = next.trim_start().to_ascii_lowercase();
                let value = if let Some(rest) = nt.strip_prefix("format:") {
                    Some(rest)
                } else if let Some(rest) = nt.strip_prefix("\"format\":") {
                    Some(rest)
                } else if let Some(rest) = nt.strip_prefix("pattern:") {
                    Some(rest)
                } else if let Some(rest) = nt.strip_prefix("\"pattern\":") {
                    Some(rest)
                } else {
                    None
                };
                if let Some(v) = value {
                    declared = Some(v.trim().trim_end_matches(',').trim_matches('"').to_owned());
                    break;
                }
            }
            out.push((id_field.clone(), idx + 1, declared));
        }
    }
    out
}

/// Audit a batch of schema documents against the policy.
#[must_use]
pub fn audit_all(documents: &[SchemaDocument], policy: &Policy) -> (Report, Vec<Finding>) {
    let mut findings = Vec::new();
    let mut microservices = BTreeSet::new();
    let mut id_fields_inspected = 0usize;
    let mut with_format = 0usize;
    let mut observed_violations: BTreeMap<(String, String), String> = BTreeMap::new();

    for doc in documents {
        microservices.insert(doc.microservice.clone());
        for (field, line, declared) in declarations(doc, policy) {
            id_fields_inspected += 1;
            let Some(declared) = declared else { continue };
            if !is_identifier_declaration(&declared) {
                continue;
            }
            with_format += 1;
            if declaration_pins_uuidv7(&declared, &policy.uuidv7_pattern) {
                continue;
            }
            observed_violations.insert((doc.path.clone(), field.clone()), declared.clone());
            let frozen = policy
                .frozen
                .iter()
                .any(|f| f.path == doc.path && f.field == field);
            if frozen {
                continue;
            }
            findings.push(Finding {
                code: CODE_UNDERSPECIFIED_FORMAT.to_owned(),
                path: doc.path.clone(),
                field: field.clone(),
                line,
                message: format!(
                    "id field `{field}` declares `{declared}`, which does not pin UUIDv7 \
                     (ADR-0709 D-1). A bare `uuid` admits v4 and carries no time ordering. \
                     Declare the UUIDv7 pattern, or add an entry with a reason to \
                     frozen_underspecified_id_formats."
                ),
            });
        }
    }

    // Shrink-only: a frozen entry matching no live declaration is drift. Without
    // this the list would quietly outlive the violations it excuses and become a
    // permanent unaudited exemption.
    for entry in &policy.frozen {
        if !observed_violations.contains_key(&(entry.path.clone(), entry.field.clone())) {
            findings.push(Finding {
                code: CODE_STALE_BASELINE.to_owned(),
                path: entry.path.clone(),
                field: entry.field.clone(),
                line: 0,
                message: format!(
                    "frozen entry `{}` / `{}` matches no live underspecified declaration — the \
                     violation was fixed or the file moved. Remove the entry in the same change \
                     so the win is recorded (the baseline is shrink-only).",
                    entry.path, entry.field
                ),
            });
        }
    }

    if documents.len() < policy.min_expected_scanned_files {
        findings.push(Finding {
            code: CODE_EMPTY_SCAN.to_owned(),
            path: "<policy>".to_owned(),
            field: "min_expected_scanned_files".to_owned(),
            line: 0,
            message: format!(
                "scanned {} document(s), below the floor of {}; the scan roots or collection are \
                 broken — this is a gate failure, never coverage",
                documents.len(),
                policy.min_expected_scanned_files
            ),
        });
    }
    if id_fields_inspected < policy.min_expected_id_fields {
        findings.push(Finding {
            code: CODE_EMPTY_SCAN.to_owned(),
            path: "<policy>".to_owned(),
            field: "min_expected_id_fields".to_owned(),
            line: 0,
            message: format!(
                "inspected {id_fields_inspected} id field(s), below the floor of {}; the canonical \
                 field list or the walk stopped matching — this is a gate failure, never coverage",
                policy.min_expected_id_fields
            ),
        });
    }

    findings.sort();
    let report = Report {
        documents_checked: documents.len(),
        id_fields_inspected,
        id_fields_with_declared_format: with_format,
        microservices_audited: microservices.len(),
    };
    (report, findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    const V7: &str = "^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$";

    fn policy() -> Policy {
        Policy {
            canonical_id_fields: ["event_id", "tenant_id", "cell_id"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            uuidv7_pattern: V7.to_owned(),
            frozen: BTreeSet::new(),
            min_expected_scanned_files: 0,
            min_expected_id_fields: 0,
        }
    }

    fn doc(contents: &str) -> SchemaDocument {
        SchemaDocument {
            path: "t.yaml".into(),
            microservice: "messenger".into(),
            contents: contents.into(),
        }
    }

    #[test]
    fn bare_uuid_is_a_finding_because_it_admits_v4() {
        let (_r, findings) = audit_all(
            &[doc(
                "properties:\n  event_id:\n    type: string\n    format: uuid\n",
            )],
            &policy(),
        );
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].code, CODE_UNDERSPECIFIED_FORMAT);
    }

    #[test]
    fn the_uuidv7_version_bit_pattern_passes() {
        let (_r, findings) = audit_all(
            &[doc(&format!(
                "properties:\n  event_id:\n    type: string\n    pattern: \"{V7}\"\n"
            ))],
            &policy(),
        );
        assert!(findings.is_empty(), "{findings:?}");
    }

    /// The rule this crate previously had BACKWARDS: ULID was accepted and uuid
    /// rejected. It must now be the violation, or the inversion silently survives.
    #[test]
    fn ulid_is_now_a_finding_not_an_acceptance() {
        let (_r, findings) = audit_all(
            &[doc(
                "properties:\n  event_id:\n    type: string\n    format: ulid\n",
            )],
            &policy(),
        );
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(findings[0].message.contains("UUIDv7"));
    }

    #[test]
    fn a_frozen_entry_tolerates_exactly_its_own_declaration() {
        let mut p = policy();
        p.frozen.insert(FrozenEntry {
            path: "t.yaml".into(),
            field: "event_id".into(),
            declared: "uuid".into(),
        });
        let (_r, findings) = audit_all(
            &[doc(
                "properties:\n  event_id:\n    type: string\n    format: uuid\n",
            )],
            &p,
        );
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn a_frozen_entry_with_no_live_violation_is_stale() {
        let mut p = policy();
        p.frozen.insert(FrozenEntry {
            path: "gone.yaml".into(),
            field: "event_id".into(),
            declared: "uuid".into(),
        });
        let (_r, findings) = audit_all(&[doc("properties:\n")], &p);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].code, CODE_STALE_BASELINE);
    }

    /// A floor that cannot fail is not a floor.
    #[test]
    fn a_collapsed_walk_is_a_gate_failure_not_coverage() {
        let mut p = policy();
        p.min_expected_scanned_files = 10;
        p.min_expected_id_fields = 5;
        let (_r, findings) = audit_all(&[doc("properties:\n")], &p);
        assert_eq!(findings.len(), 2, "{findings:?}");
        assert!(findings.iter().all(|f| f.code == CODE_EMPTY_SCAN));
    }

    /// A hierarchical tenant slug is a design, not drift. The gate must not
    /// report its own rule as a defect.
    #[test]
    fn a_domain_specific_slug_pattern_is_out_of_scope() {
        let (_r, findings) = audit_all(
            &[doc(
                "properties:\n  tenant_id:\n    type: string\n    pattern: \"^oyatie\\.(dev|ci)\\.[a-z0-9-]+$\"\n",
            )],
            &policy(),
        );
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn unrelated_formats_are_not_reported() {
        let (_r, findings) = audit_all(
            &[doc(
                "properties:\n  event_id:\n    type: string\n    format: date-time\n",
            )],
            &policy(),
        );
        assert!(findings.is_empty(), "{findings:?}");
    }
}
