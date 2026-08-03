//! Honest-claims and ChangeSet graph validation kernel.
//!
//! This crate is intentionally I/O-free. Runners hand it corpus documents and
//! implementation-plan documents; the kernel returns exact file/line findings.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `panic!()`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::panic))]

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

const CHANGESET_CONTRACT: &str = "claimable-verifiable-bundleable-promotable";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HonestClaimsDocument {
    pub path: String,     // data_class: INTERNAL_ONLY
    pub contents: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HonestClaimsReport {
    pub documents_checked: usize, // data_class: INTERNAL_ONLY
    pub lines_checked: usize,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HonestClaimsViolation {
    pub path: String,    // data_class: INTERNAL_ONLY
    pub line: usize,     // data_class: INTERNAL_ONLY
    pub kind: ClaimKind, // data_class: INTERNAL_ONLY
    pub phrase: String,  // data_class: INTERNAL_ONLY
    pub summary: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimKind {
    DeferredRequiredClaim,
    DeferredActiveClaim,
    UnsupportedMaturityClaim,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImplementationPlanDocument {
    pub path: String,     // data_class: INTERNAL_ONLY
    pub contents: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeSetPlanReport {
    pub plans_checked: usize,             // data_class: INTERNAL_ONLY
    pub dependency_edges: usize,          // data_class: INTERNAL_ONLY
    pub serialization_edges: usize,       // data_class: INTERNAL_ONLY
    pub global_artifact_writes: usize,    // data_class: INTERNAL_ONLY
    pub legacy_missing_split_rule: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeSetPlanViolation {
    pub path: String,    // data_class: INTERNAL_ONLY
    pub line: usize,     // data_class: INTERNAL_ONLY
    pub kind: PlanKind,  // data_class: INTERNAL_ONLY
    pub summary: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanKind {
    MissingFrontmatter,
    InvalidFrontmatter,
    MissingRequiredField,
    InvalidRequiredField,
    DuplicateChangeSetId,
    UnknownDependency,
    SelfDependency,
    Cycle,
    UnknownSerializationPeer,
    AsymmetricSerialization,
    GlobalArtifactConflict,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParityTargetSourceTrackingReport {
    pub targets_checked: usize,     // data_class: INTERNAL_ONLY
    pub source_refs_checked: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParityTargetSourceTrackingViolation {
    pub path: String,                         // data_class: INTERNAL_ONLY
    pub target_id: String,                    // data_class: INTERNAL_ONLY
    pub kind: ParityTargetSourceTrackingKind, // data_class: INTERNAL_ONLY
    pub summary: String,                      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParityTargetSourceTrackingKind {
    MissingTargets,
    MalformedTarget,
    DuplicateTargetId,
    MissingPinnedSource,
    ExternalAuthorityLeak,
    PromotionAuthorityClaim,
    MissingValidationRefs,
    MissingStopConditions,
    OptimisticKernelAbiClaim,
    MissingCwardPessimism,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceFamilyDocument {
    pub path: String,     // data_class: INTERNAL_ONLY
    pub contents: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceFamilyParityReport {
    pub documents_checked: usize,          // data_class: INTERNAL_ONLY
    pub lines_checked: usize,              // data_class: INTERNAL_ONLY
    pub reference_families_checked: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceFamilyParityViolation {
    pub path: String,                    // data_class: INTERNAL_ONLY
    pub line: usize,                     // data_class: INTERNAL_ONLY
    pub kind: ReferenceFamilyParityKind, // data_class: INTERNAL_ONLY
    pub family: String,                  // data_class: INTERNAL_ONLY
    pub summary: String,                 // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceFamilyParityKind {
    MissingReferenceFamily,
    GithubOnlyAdapterOverfit,
    AdapterAsAuthority,
}

impl fmt::Display for HonestClaimsViolation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{} {:?}: {} ({})",
            self.path, self.line, self.kind, self.summary, self.phrase
        )
    }
}

impl fmt::Display for ChangeSetPlanViolation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{} {:?}: {}",
            self.path, self.line, self.kind, self.summary
        )
    }
}

impl fmt::Display for ParityTargetSourceTrackingViolation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{} {:?}: {}",
            self.path, self.target_id, self.kind, self.summary
        )
    }
}

impl fmt::Display for ReferenceFamilyParityViolation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{} {:?}: {} ({})",
            self.path, self.line, self.kind, self.summary, self.family
        )
    }
}

pub fn validate_honest_claims<D>(
    documents: D,
) -> Result<HonestClaimsReport, Vec<HonestClaimsViolation>>
where
    D: IntoIterator<Item = HonestClaimsDocument>,
{
    let mut documents_checked = 0usize;
    let mut lines_checked = 0usize;
    let mut violations = Vec::new();

    for document in documents {
        documents_checked += 1;
        let mut in_code_fence = false;
        for (line_index, line) in document.contents.lines().enumerate() {
            lines_checked += 1;
            let line_number = line_index + 1;
            let trimmed = line.trim_start();
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_fence = !in_code_fence;
                continue;
            }
            if in_code_fence || is_example_line(trimmed) {
                continue;
            }
            violations.extend(scan_claim_line(&document.path, line_number, line));
        }
    }

    if violations.is_empty() {
        Ok(HonestClaimsReport {
            documents_checked,
            lines_checked,
        })
    } else {
        Err(violations)
    }
}

pub fn validate_changeset_plan_graph<D>(
    documents: D,
) -> Result<ChangeSetPlanReport, Vec<ChangeSetPlanViolation>>
where
    D: IntoIterator<Item = ImplementationPlanDocument>,
{
    let mut records = Vec::new();
    let mut violations = Vec::new();

    for document in documents {
        match PlanRecord::parse(document) {
            Ok(record) => records.push(record),
            Err(mut record_violations) => violations.append(&mut record_violations),
        }
    }

    if records.is_empty() {
        violations.push(ChangeSetPlanViolation {
            path: ".omc/plans".to_string(),
            line: 1,
            kind: PlanKind::MissingFrontmatter,
            summary: "no implementation plan documents were supplied".to_string(),
        });
    }

    let mut by_id: BTreeMap<String, usize> = BTreeMap::new();
    for (index, record) in records.iter().enumerate() {
        if let Some(prior) = by_id.insert(record.id.clone(), index) {
            violations.push(ChangeSetPlanViolation {
                path: record.path.clone(),
                line: record.id_line,
                kind: PlanKind::DuplicateChangeSetId,
                summary: format!(
                    "ChangeSet id {} duplicates {}",
                    record.id, records[prior].path
                ),
            });
        }
    }

    let mut dependency_edges = 0usize;
    let mut serialization_edges = 0usize;
    let mut global_artifact_writes = 0usize;
    for record in &records {
        dependency_edges += record.depends_on.len();
        serialization_edges += record.serializes_with.len();
        global_artifact_writes += record.writes_global_artifacts.len();

        for dependency in &record.depends_on {
            if dependency == &record.id {
                violations.push(ChangeSetPlanViolation {
                    path: record.path.clone(),
                    line: record.depends_on_line.unwrap_or(record.id_line),
                    kind: PlanKind::SelfDependency,
                    summary: format!("ChangeSet {} depends on itself", record.id),
                });
            } else if !by_id.contains_key(dependency) {
                violations.push(ChangeSetPlanViolation {
                    path: record.path.clone(),
                    line: record.depends_on_line.unwrap_or(record.id_line),
                    kind: PlanKind::UnknownDependency,
                    summary: format!(
                        "ChangeSet {} depends on unknown ChangeSet {}",
                        record.id, dependency
                    ),
                });
            }
        }

        for peer in &record.serializes_with {
            if peer == &record.id {
                violations.push(ChangeSetPlanViolation {
                    path: record.path.clone(),
                    line: record.serializes_with_line.unwrap_or(record.id_line),
                    kind: PlanKind::SelfDependency,
                    summary: format!("ChangeSet {} serializes with itself", record.id),
                });
            } else if let Some(peer_index) = by_id.get(peer) {
                let peer_record = &records[*peer_index];
                if !peer_record.serializes_with.contains(&record.id) {
                    violations.push(ChangeSetPlanViolation {
                        path: record.path.clone(),
                        line: record.serializes_with_line.unwrap_or(record.id_line),
                        kind: PlanKind::AsymmetricSerialization,
                        summary: format!(
                            "{} serializes with {}, but the reverse edge is absent",
                            record.id, peer
                        ),
                    });
                }
            } else {
                violations.push(ChangeSetPlanViolation {
                    path: record.path.clone(),
                    line: record.serializes_with_line.unwrap_or(record.id_line),
                    kind: PlanKind::UnknownSerializationPeer,
                    summary: format!(
                        "ChangeSet {} serializes with unknown ChangeSet {}",
                        record.id, peer
                    ),
                });
            }
        }
    }

    let graph = records
        .iter()
        .map(|record| (record.id.clone(), record.depends_on.clone()))
        .collect::<BTreeMap<_, _>>();
    detect_cycles(&records, &graph, &mut violations);
    detect_global_artifact_conflicts(&records, &mut violations);

    if violations.is_empty() {
        Ok(ChangeSetPlanReport {
            plans_checked: records.len(),
            dependency_edges,
            serialization_edges,
            global_artifact_writes,
            legacy_missing_split_rule: records
                .iter()
                .filter(|record| !record.has_split_rule)
                .count(),
        })
    } else {
        Err(violations)
    }
}

pub fn validate_parity_target_source_tracking(
    path: impl Into<String>,
    input: &Value,
) -> Result<ParityTargetSourceTrackingReport, Vec<ParityTargetSourceTrackingViolation>> {
    let path = path.into();
    let mut violations = Vec::new();
    let Some(targets) = input.get("parity_targets").and_then(Value::as_array) else {
        violations.push(parity_violation(
            &path,
            "<document>",
            ParityTargetSourceTrackingKind::MissingTargets,
            "document must contain a parity_targets array",
        ));
        return Err(violations);
    };

    let mut seen_targets = BTreeSet::new();
    let mut source_refs_checked = 0usize;

    for target in targets {
        let Some(target_object) = target.as_object() else {
            violations.push(parity_violation(
                &path,
                "<malformed>",
                ParityTargetSourceTrackingKind::MalformedTarget,
                "parity target row must be an object",
            ));
            continue;
        };

        let target_id = string_field(target, "target_id").unwrap_or("<missing>");
        if target_id == "<missing>" {
            violations.push(parity_violation(
                &path,
                target_id,
                ParityTargetSourceTrackingKind::MalformedTarget,
                "parity target requires target_id",
            ));
        } else if !seen_targets.insert(target_id.to_string()) {
            violations.push(parity_violation(
                &path,
                target_id,
                ParityTargetSourceTrackingKind::DuplicateTargetId,
                "target_id must be unique within the source-tracking document",
            ));
        }

        for required in [
            "reference_kind",
            "source_tracking_state",
            "freshness_state",
            "parity_posture",
            "authority_boundary",
        ] {
            if string_field(target, required).is_none() {
                violations.push(parity_violation(
                    &path,
                    target_id,
                    ParityTargetSourceTrackingKind::MalformedTarget,
                    format!("parity target requires non-empty {required}"),
                ));
            }
        }

        match target_object.get("source_refs").and_then(Value::as_array) {
            Some(source_refs) if !source_refs.is_empty() => {
                source_refs_checked += source_refs.len();
                for source_ref in source_refs {
                    if string_field(source_ref, "url").is_none()
                        || string_field(source_ref, "retrieved_at").is_none()
                        || string_field(source_ref, "pinned_ref").is_none()
                    {
                        violations.push(parity_violation(
                            &path,
                            target_id,
                            ParityTargetSourceTrackingKind::MissingPinnedSource,
                            "each source_ref requires url, retrieved_at, and pinned_ref",
                        ));
                    }

                    if source_ref
                        .get("external_authority")
                        .and_then(Value::as_bool)
                        != Some(false)
                    {
                        violations.push(parity_violation(
                            &path,
                            target_id,
                            ParityTargetSourceTrackingKind::ExternalAuthorityLeak,
                            "source_ref.external_authority must be false",
                        ));
                    }
                }
            }
            _ => violations.push(parity_violation(
                &path,
                target_id,
                ParityTargetSourceTrackingKind::MissingPinnedSource,
                "parity target requires at least one pinned source_ref",
            )),
        }

        if !non_empty_array(target, "validation_refs") {
            violations.push(parity_violation(
                &path,
                target_id,
                ParityTargetSourceTrackingKind::MissingValidationRefs,
                "parity target requires validation_refs",
            ));
        }

        if !non_empty_array(target, "stop_conditions") {
            violations.push(parity_violation(
                &path,
                target_id,
                ParityTargetSourceTrackingKind::MissingStopConditions,
                "parity target requires stop_conditions",
            ));
        }

        let target_text = target.to_string().to_ascii_lowercase();
        if contains_authority_claim(&target_text) {
            violations.push(parity_violation(
                &path,
                target_id,
                ParityTargetSourceTrackingKind::PromotionAuthorityClaim,
                "parity target cannot claim merge, promotion, product-ready, or hyperscaler-ready authority",
            ));
        }

        if is_cward_or_linux_libc_target(target) {
            validate_cward_pessimism(&path, target_id, target, &target_text, &mut violations);
        }
    }

    if violations.is_empty() {
        Ok(ParityTargetSourceTrackingReport {
            targets_checked: targets.len(),
            source_refs_checked,
        })
    } else {
        Err(violations)
    }
}

fn validate_cward_pessimism(
    path: &str,
    target_id: &str,
    target: &Value,
    target_text: &str,
    violations: &mut Vec<ParityTargetSourceTrackingViolation>,
) {
    if string_field(target, "reference_kind") != Some("linux_libc_abi_pessimistic_input")
        || !string_field(target, "parity_posture")
            .is_some_and(|posture| posture.contains("pessimistic"))
        || target
            .get("userspace_abi_preserved")
            .and_then(Value::as_bool)
            != Some(true)
    {
        violations.push(parity_violation(
            path,
            target_id,
            ParityTargetSourceTrackingKind::MissingCwardPessimism,
            "c-ward/Linux libc ABI rows must be pessimistic userspace-ABI inputs only",
        ));
    }

    if target
        .get("internal_kernel_api_stable")
        .and_then(Value::as_bool)
        != Some(false)
        || target_text.contains("kernel api stable")
        || target_text.contains("kernel-api-stable")
        || target_text.contains("production ready")
    {
        violations.push(parity_violation(
            path,
            target_id,
            ParityTargetSourceTrackingKind::OptimisticKernelAbiClaim,
            "c-ward/Linux libc ABI rows must not claim stable internal kernel APIs or production readiness",
        ));
    }

    let gap_text = target
        .get("known_gaps")
        .and_then(Value::as_array)
        .map(|gaps| {
            gaps.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" ")
                .to_ascii_lowercase()
        })
        .unwrap_or_default();

    for required_gap in ["getent", "nightly", "incomplete", "linux-gnu-only"] {
        if !gap_text.contains(required_gap) {
            violations.push(parity_violation(
                path,
                target_id,
                ParityTargetSourceTrackingKind::MissingCwardPessimism,
                format!("c-ward known_gaps must record {required_gap} caveat"),
            ));
        }
    }
}

fn parity_violation(
    path: &str,
    target_id: &str,
    kind: ParityTargetSourceTrackingKind,
    summary: impl Into<String>,
) -> ParityTargetSourceTrackingViolation {
    ParityTargetSourceTrackingViolation {
        path: path.to_string(),
        target_id: target_id.to_string(),
        kind,
        summary: summary.into(),
    }
}

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str).and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn non_empty_array(value: &Value, key: &str) -> bool {
    value
        .get(key)
        .and_then(Value::as_array)
        .is_some_and(|values| !values.is_empty())
}

fn contains_authority_claim(target_text: &str) -> bool {
    [
        "approved_product_ready",
        "approved-product-ready",
        "approved product ready",
        "approved_hyperscaler_ready",
        "approved-hyperscaler-ready",
        "approved hyperscaler ready",
        "merge_authority",
        "merge-authority",
        "merge authority",
        "promotion_authority",
        "promotion-authority",
        "promotion authority",
        "permanent_product_authority",
        "permanent-product-authority",
        "permanent product authority",
        "product_ready",
        "product-ready",
        "product ready",
        "hyperscaler_ready",
        "hyperscaler-ready",
        "hyperscaler ready",
    ]
    .iter()
    .any(|marker| target_text.contains(marker))
}

fn is_cward_or_linux_libc_target(target: &Value) -> bool {
    let target_id = string_field(target, "target_id").unwrap_or_default();
    let reference_kind = string_field(target, "reference_kind").unwrap_or_default();
    let source_text = target
        .get("source_refs")
        .and_then(Value::as_array)
        .map(|refs| {
            refs.iter()
                .filter_map(|source_ref| string_field(source_ref, "url"))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();

    target_id.contains("c-ward")
        || reference_kind == "linux_libc_abi_pessimistic_input"
        || source_text.contains("github.com/sunfishcode/c-ward")
}

pub fn validate_reference_family_parity<D>(
    documents: D,
) -> Result<ReferenceFamilyParityReport, Vec<ReferenceFamilyParityViolation>>
where
    D: IntoIterator<Item = ReferenceFamilyDocument>,
{
    let mut documents_checked = 0usize;
    let mut lines_checked = 0usize;
    let mut reference_families = BTreeSet::new();
    let mut violations = Vec::new();

    for document in documents {
        documents_checked += 1;
        let scan = ReferenceFamilyScan::from_document(&document);
        lines_checked += scan.lines_checked;
        reference_families.extend(scan.present_families.iter().copied());

        if scan.family_lines.is_empty() {
            violations.push(ReferenceFamilyParityViolation {
                path: document.path.clone(),
                line: 1,
                kind: ReferenceFamilyParityKind::MissingReferenceFamily,
                family: "reference-family-set".to_string(),
                summary: "parity claim lacks a reference_families/reference family declaration"
                    .to_string(),
            });
        } else {
            let reference_line = scan.first_reference_line();
            for rule in REQUIRED_REFERENCE_FAMILIES {
                if !scan.present_families.contains(rule.id) {
                    violations.push(ReferenceFamilyParityViolation {
                        path: document.path.clone(),
                        line: reference_line,
                        kind: ReferenceFamilyParityKind::MissingReferenceFamily,
                        family: rule.display.to_string(),
                        summary: format!(
                            "reference family {} is required for directive parity coverage",
                            rule.display
                        ),
                    });
                }
            }
        }

        if let Some(line) = scan.github_family_line
            && !scan.gitlab_in_family_lines
        {
            violations.push(ReferenceFamilyParityViolation {
                path: document.path.clone(),
                line,
                kind: ReferenceFamilyParityKind::GithubOnlyAdapterOverfit,
                family: "GitHub/GitLab adapters".to_string(),
                summary:
                    "GitHub adapter coverage must be paired with GitLab to avoid hosted-SCM overfit"
                        .to_string(),
            });
        }

        for (line, family) in scan.adapter_authority_lines {
            violations.push(ReferenceFamilyParityViolation {
                path: document.path.clone(),
                line,
                kind: ReferenceFamilyParityKind::AdapterAsAuthority,
                family: family.to_string(),
                summary:
                    "reference adapters may be bridge/evidence families, not product authority"
                        .to_string(),
            });
        }
    }

    if violations.is_empty() {
        Ok(ReferenceFamilyParityReport {
            documents_checked,
            lines_checked,
            reference_families_checked: reference_families.len(),
        })
    } else {
        Err(violations)
    }
}

#[derive(Clone, Copy)]
struct ReferenceFamilyRule {
    id: &'static str,
    display: &'static str,
}

const REQUIRED_REFERENCE_FAMILIES: &[ReferenceFamilyRule] = &[
    ReferenceFamilyRule {
        id: "c-ward",
        display: "c-ward",
    },
    ReferenceFamilyRule {
        id: "kubernetes",
        display: "Kubernetes",
    },
    ReferenceFamilyRule {
        id: "argo",
        display: "Argo",
    },
    ReferenceFamilyRule {
        id: "github-gitlab-adapters",
        display: "GitHub/GitLab adapters",
    },
    ReferenceFamilyRule {
        id: "snyk-trivy-drift",
        display: "Snyk/Trivy/drift",
    },
    ReferenceFamilyRule {
        id: "linux-oci",
        display: "Linux/OCI",
    },
];

struct ReferenceFamilyScan {
    lines_checked: usize,
    family_lines: Vec<(usize, String)>,
    present_families: BTreeSet<&'static str>,
    github_family_line: Option<usize>,
    gitlab_in_family_lines: bool,
    adapter_authority_lines: Vec<(usize, &'static str)>,
}

impl ReferenceFamilyScan {
    fn from_document(document: &ReferenceFamilyDocument) -> Self {
        let mut lines_checked = 0usize;
        let mut family_lines = Vec::new();
        let mut adapter_authority_lines = Vec::new();
        let mut in_code_fence = false;
        let mut in_reference_family_list = false;

        for (line_index, raw) in document.contents.lines().enumerate() {
            lines_checked += 1;
            let line_number = line_index + 1;
            let trimmed = raw.trim_start();
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_fence = !in_code_fence;
                continue;
            }
            if in_code_fence || is_example_line(trimmed) {
                continue;
            }

            let lower = raw.to_ascii_lowercase();
            if let Some(family) = adapter_as_authority_family(&lower) {
                adapter_authority_lines.push((line_number, family));
            }

            if is_reference_family_header(&lower) {
                in_reference_family_list =
                    lower.trim_end().ends_with(':') || trimmed.trim_start().starts_with('#');
                family_lines.push((line_number, lower));
                continue;
            }

            if in_reference_family_list && trimmed.trim_start().starts_with("- ") {
                family_lines.push((line_number, lower));
                continue;
            }

            if !trimmed.trim().is_empty() && !trimmed.trim_start().starts_with('#') {
                in_reference_family_list = false;
            }
        }

        let mut present_families = BTreeSet::new();
        let family_blob = family_lines
            .iter()
            .map(|(_, line)| line.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        if has_c_ward_reference(&family_blob) {
            present_families.insert("c-ward");
        }
        if has_any(&family_blob, &["kubernetes", "k8s"]) {
            present_families.insert("kubernetes");
        }
        if has_any(
            &family_blob,
            &["argo", "argocd", "argo cd", "argo rollouts"],
        ) {
            present_families.insert("argo");
        }
        if family_blob.contains("github") && family_blob.contains("gitlab") {
            present_families.insert("github-gitlab-adapters");
        }
        if family_blob.contains("snyk")
            && family_blob.contains("trivy")
            && has_any(&family_blob, &["driftctl", "drift"])
        {
            present_families.insert("snyk-trivy-drift");
        }
        if family_blob.contains("linux") && family_blob.contains("oci") {
            present_families.insert("linux-oci");
        }

        let github_family_line = family_lines
            .iter()
            .find_map(|(line, text)| text.contains("github").then_some(*line));
        let gitlab_in_family_lines = family_blob.contains("gitlab");

        Self {
            lines_checked,
            family_lines,
            present_families,
            github_family_line,
            gitlab_in_family_lines,
            adapter_authority_lines,
        }
    }

    fn first_reference_line(&self) -> usize {
        self.family_lines
            .first()
            .map(|(line, _)| *line)
            .unwrap_or(1)
    }
}

fn is_reference_family_header(lower: &str) -> bool {
    lower.contains("reference_families")
        || lower.contains("reference_family")
        || lower.contains("reference-famil")
        || lower.contains("reference family")
        || lower.contains("reference families")
}

fn has_c_ward_reference(lower: &str) -> bool {
    has_any(lower, &["c-ward", "c ward", "c_ward", "cward"])
}

fn has_any(lower: &str, markers: &[&str]) -> bool {
    markers.iter().any(|marker| lower.contains(marker))
}

fn adapter_as_authority_family(lower: &str) -> Option<&'static str> {
    for (family, aliases) in ADAPTER_AUTHORITY_TERMS {
        for alias in *aliases {
            if adapter_alias_is_authority(lower, alias) {
                return Some(*family);
            }
        }
    }
    None
}

const ADAPTER_AUTHORITY_TERMS: &[(&str, &[&str])] = &[
    ("GitHub", &["github"]),
    ("GitLab", &["gitlab"]),
    ("Kubernetes", &["kubernetes", "k8s"]),
    ("Argo", &["argo", "argocd", "argo cd"]),
    ("Snyk", &["snyk"]),
    ("Trivy", &["trivy"]),
    ("driftctl", &["driftctl"]),
    ("c-ward", &["c-ward", "c ward", "c_ward", "cward"]),
    ("Linux", &["linux"]),
    ("OCI", &["oci"]),
];

fn adapter_alias_is_authority(lower: &str, alias: &str) -> bool {
    [
        format!("{alias} is the authority"),
        format!("{alias} is authority"),
        format!("{alias} owns authority"),
        format!("{alias} owns the authority"),
        format!("{alias} becomes authority"),
        format!("{alias} becomes the authority"),
        format!("{alias} is canonical"),
        format!("{alias} is the canonical"),
        format!("{alias} as authority"),
        format!("{alias} is source of truth"),
        format!("{alias} is the source of truth"),
        format!("authority: {alias}"),
        format!("canonical authority: {alias}"),
        format!("source of truth: {alias}"),
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn scan_claim_line(path: &str, line: usize, raw: &str) -> Vec<HonestClaimsViolation> {
    let lower = raw.to_ascii_lowercase();
    let mut violations = Vec::new();

    if mentions_hyperscaler_maturity(&lower) && !has_blocked_or_advisory_status(&lower) {
        violations.push(HonestClaimsViolation {
            path: path.to_string(),
            line,
            kind: ClaimKind::UnsupportedMaturityClaim,
            phrase: "hyperscaler mature".to_string(),
            summary: "hyperscaler-maturity claim lacks explicit blocked/advisory evidence status"
                .to_string(),
        });
    }

    let claim_kind = if has_required_context(&lower) {
        Some(ClaimKind::DeferredRequiredClaim)
    } else if has_active_context(&lower) {
        Some(ClaimKind::DeferredActiveClaim)
    } else {
        None
    };
    let Some(kind) = claim_kind else {
        return violations;
    };

    for phrase in deferral_phrases(&lower) {
        if has_honest_advisory_boundary(&lower) {
            continue;
        }
        violations.push(HonestClaimsViolation {
            path: path.to_string(),
            line,
            kind,
            phrase,
            summary: "line combines an active/required claim with deferred delivery language"
                .to_string(),
        });
    }

    if let Some(version) = version_deferral(&lower)
        && !has_honest_advisory_boundary(&lower)
    {
        violations.push(HonestClaimsViolation {
            path: path.to_string(),
            line,
            kind,
            phrase: version,
            summary: "line combines an active/required claim with a later-version deferral"
                .to_string(),
        });
    }

    violations
}

fn has_required_context(lower: &str) -> bool {
    [
        "required",
        "must",
        "shall",
        "blocks merge",
        "blocking",
        "branch protection",
        "required check",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn has_active_context(lower: &str) -> bool {
    [
        "active",
        "enforced",
        "mechanical",
        "wired",
        "complete",
        "ships",
        "production-ready",
        "ga quality",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn has_honest_advisory_boundary(lower: &str) -> bool {
    [
        "advisory",
        "planned",
        "proposed",
        "blocked_until",
        "blocked until",
        "not active",
        "not required",
        "not enforced",
        "not yet active",
        "per-product prd",
        "sunset",
        "retired",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn mentions_hyperscaler_maturity(lower: &str) -> bool {
    !lower.trim_start().starts_with("\"hyperscaler mature\"")
        && (lower.contains("hyperscaler mature")
            || lower.contains("hyperscaler-mature")
            || lower.contains("hyperscaler maturity achieved"))
}

fn has_blocked_or_advisory_status(lower: &str) -> bool {
    has_honest_advisory_boundary(lower)
        || lower.contains("blocked_until_required_evidence_is_green")
        || lower.contains("cannot claim")
        || lower.contains("forbidden")
        || lower.contains("allowed unless")
        || lower.contains("governs whether")
        || lower.contains("claim gate")
        || lower.contains("claim_boundary")
        || lower.contains("exact_claim")
        || lower.contains("\"we are hyperscaler mature\"")
        || lower.contains("'we are hyperscaler mature'")
        || lower.contains("evidence-gated")
        || lower.contains("unsupported")
}

fn deferral_phrases(lower: &str) -> Vec<String> {
    [
        "to be added",
        "to be implemented",
        "future work",
        "future scope",
        "later wave",
        "future wave",
        "follow-on iteration",
        "follow-up iteration",
        "out of scope for now",
        "not in scope for this",
        "deferred to",
        "punted to",
    ]
    .iter()
    .filter(|marker| lower.contains(**marker))
    .map(|marker| (*marker).to_string())
    .collect()
}

fn version_deferral(lower: &str) -> Option<String> {
    for prefix in [
        "lands in ",
        "ships in ",
        "will land in ",
        "will ship in ",
        "enabled in ",
        "enforced in ",
        "active in ",
        "available in ",
        "deferred to ",
    ] {
        if let Some(start) = lower.find(prefix) {
            let version_start = start + prefix.len();
            if let Some(version) = parse_version_token(&lower[version_start..])
                && version != "v1.0"
            {
                return Some(version);
            }
        }
    }
    None
}

fn parse_version_token(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    if bytes.first().copied() != Some(b'v') || bytes.get(1).is_none_or(|b| !b.is_ascii_digit()) {
        return None;
    }
    let mut index = 2usize;
    while index < bytes.len() && (bytes[index].is_ascii_digit() || bytes[index] == b'.') {
        index += 1;
    }
    Some(input[..index].trim_end_matches('.').to_string())
}

fn is_example_line(trimmed: &str) -> bool {
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("example:")
        || lower.starts_with("sample:")
        || lower.starts_with("# example")
        || lower.starts_with("// example")
}

#[derive(Clone, Debug)]
struct PlanRecord {
    path: String,
    id: String,
    id_line: usize,
    depends_on: BTreeSet<String>,
    depends_on_line: Option<usize>,
    serializes_with: BTreeSet<String>,
    serializes_with_line: Option<usize>,
    writes_global_artifacts: BTreeSet<String>,
    writes_global_artifacts_line: Option<usize>,
    has_split_rule: bool,
}

impl PlanRecord {
    fn parse(document: ImplementationPlanDocument) -> Result<Self, Vec<ChangeSetPlanViolation>> {
        let mut violations = Vec::new();
        let frontmatter = match parse_frontmatter(&document) {
            Ok(frontmatter) => frontmatter,
            Err(violation) => return Err(vec![violation]),
        };

        let Some((doc_class, doc_class_line)) = frontmatter.scalars.get("doc_class") else {
            violations.push(missing(&document.path, "doc_class"));
            return Err(violations);
        };
        if doc_class != "ImplementationPlan" {
            violations.push(ChangeSetPlanViolation {
                path: document.path.clone(),
                line: *doc_class_line,
                kind: PlanKind::InvalidRequiredField,
                summary: "doc_class must be ImplementationPlan".to_string(),
            });
        }

        let Some((id, id_line)) = frontmatter.scalars.get("id") else {
            violations.push(missing(&document.path, "id"));
            return Err(violations);
        };
        if !valid_changeset_id(id) {
            violations.push(ChangeSetPlanViolation {
                path: document.path.clone(),
                line: *id_line,
                kind: PlanKind::InvalidRequiredField,
                summary: format!("id {id} is not a valid numeric ChangeSet id"),
            });
        }

        require_scalar(
            &frontmatter,
            &document.path,
            "execution_unit",
            "ChangeSet",
            &mut violations,
        );
        require_scalar(
            &frontmatter,
            &document.path,
            "changeset_contract",
            CHANGESET_CONTRACT,
            &mut violations,
        );

        let has_split_rule = frontmatter.scalars.contains_key("changeset_split_rule");
        let depends_on = list_field(&frontmatter, "depends_on_changesets", &document.path);
        let serializes_with =
            list_field(&frontmatter, "serializes_with_changesets", &document.path);
        let writes_global_artifacts =
            list_field(&frontmatter, "writes_global_artifacts", &document.path);

        if violations.is_empty() {
            Ok(Self {
                path: document.path,
                id: id.clone(),
                id_line: *id_line,
                depends_on: depends_on.0,
                depends_on_line: depends_on.1,
                serializes_with: serializes_with.0,
                serializes_with_line: serializes_with.1,
                writes_global_artifacts: writes_global_artifacts.0,
                writes_global_artifacts_line: writes_global_artifacts.1,
                has_split_rule,
            })
        } else {
            Err(violations)
        }
    }
}

#[derive(Clone, Debug)]
struct Frontmatter {
    scalars: BTreeMap<String, (String, usize)>,
    lists: BTreeMap<String, (Vec<String>, usize)>,
}

fn parse_frontmatter(
    document: &ImplementationPlanDocument,
) -> Result<Frontmatter, ChangeSetPlanViolation> {
    let mut lines = document.contents.lines().enumerate();
    match lines.next() {
        Some((_, "---")) => {}
        _ => {
            return Err(ChangeSetPlanViolation {
                path: document.path.clone(),
                line: 1,
                kind: PlanKind::MissingFrontmatter,
                summary: "implementation plan must start with YAML frontmatter".to_string(),
            });
        }
    }

    let mut scalars = BTreeMap::new();
    let mut lists: BTreeMap<String, (Vec<String>, usize)> = BTreeMap::new();
    let mut active_list_key: Option<String> = None;
    for (index, raw) in lines {
        let line_no = index + 1;
        if raw == "---" {
            return Ok(Frontmatter { scalars, lists });
        }
        if raw.trim().is_empty() || raw.trim_start().starts_with('#') {
            continue;
        }
        let Some((key, value)) = raw.split_once(':') else {
            if raw.trim_start().starts_with("- ") {
                if let Some(key) = &active_list_key
                    && let Some((items, _)) = lists.get_mut(key)
                {
                    let item = unquote(raw.trim_start().trim_start_matches("- ").trim());
                    if !item.is_empty() {
                        items.push(item);
                    }
                }
                continue;
            }
            if raw.starts_with(' ') {
                continue;
            }
            return Err(ChangeSetPlanViolation {
                path: document.path.clone(),
                line: line_no,
                kind: PlanKind::InvalidFrontmatter,
                summary: "frontmatter line must be key: value".to_string(),
            });
        };
        let key = key.trim().to_string();
        let value = value.trim();
        if value.is_empty() {
            lists.entry(key.clone()).or_insert((Vec::new(), line_no));
            active_list_key = Some(key);
        } else {
            active_list_key = None;
            scalars.insert(key, (unquote(value), line_no));
        }
    }

    Err(ChangeSetPlanViolation {
        path: document.path.clone(),
        line: document.contents.lines().count().max(1),
        kind: PlanKind::MissingFrontmatter,
        summary: "frontmatter closing marker is missing".to_string(),
    })
}

fn missing(path: &str, field: &str) -> ChangeSetPlanViolation {
    ChangeSetPlanViolation {
        path: path.to_string(),
        line: 1,
        kind: PlanKind::MissingRequiredField,
        summary: format!("missing required frontmatter field {field}"),
    }
}

fn require_scalar(
    frontmatter: &Frontmatter,
    path: &str,
    field: &str,
    expected: &str,
    violations: &mut Vec<ChangeSetPlanViolation>,
) {
    match frontmatter.scalars.get(field) {
        Some((actual, _)) if actual == expected => {}
        Some((actual, line)) => violations.push(ChangeSetPlanViolation {
            path: path.to_string(),
            line: *line,
            kind: PlanKind::InvalidRequiredField,
            summary: format!("{field} must be {expected}, got {actual}"),
        }),
        None => violations.push(missing(path, field)),
    }
}

fn list_field(
    frontmatter: &Frontmatter,
    field: &str,
    path: &str,
) -> (BTreeSet<String>, Option<usize>) {
    let Some((value, line)) = frontmatter.scalars.get(field) else {
        if let Some((values, line)) = frontmatter.lists.get(field) {
            return (values.iter().cloned().collect(), Some(*line));
        }
        return (BTreeSet::new(), None);
    };
    let mut values = BTreeSet::new();
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "[]" {
        return (values, Some(*line));
    }
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        for item in inner.split(',') {
            let item = unquote(item.trim());
            if !item.is_empty() {
                values.insert(item);
            }
        }
    } else {
        values.insert(trimmed.to_string());
    }
    if values.iter().any(|value| value.contains('\t')) {
        values.insert(format!("INVALID-TAB-IN-{path}"));
    }
    (values, Some(*line))
}

fn unquote(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

fn valid_changeset_id(value: &str) -> bool {
    let Some(rest) = value.strip_prefix('M') else {
        return false;
    };
    let Some((milestone, rest)) = rest.split_once("-P") else {
        return false;
    };
    let Some((phase, ip)) = rest.split_once("-IP-") else {
        return false;
    };
    milestone.len() == 2
        && milestone.bytes().all(|byte| byte.is_ascii_digit())
        && phase.len() == 2
        && phase.bytes().all(|byte| byte.is_ascii_digit())
        && valid_ip_number(ip)
}

fn valid_ip_number(value: &str) -> bool {
    let (major, suffix) = value
        .split_once('.')
        .map_or((value, None), |(major, suffix)| (major, Some(suffix)));
    major.len() == 3
        && major.bytes().all(|byte| byte.is_ascii_digit())
        && suffix.is_none_or(|suffix| {
            !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
        })
}

fn detect_cycles(
    records: &[PlanRecord],
    graph: &BTreeMap<String, BTreeSet<String>>,
    violations: &mut Vec<ChangeSetPlanViolation>,
) {
    let mut permanent = BTreeSet::new();
    let mut temporary = BTreeSet::new();
    for record in records {
        visit_cycle(
            record,
            graph,
            records,
            &mut permanent,
            &mut temporary,
            violations,
        );
    }
}

fn visit_cycle(
    record: &PlanRecord,
    graph: &BTreeMap<String, BTreeSet<String>>,
    records: &[PlanRecord],
    permanent: &mut BTreeSet<String>,
    temporary: &mut BTreeSet<String>,
    violations: &mut Vec<ChangeSetPlanViolation>,
) {
    if permanent.contains(&record.id) {
        return;
    }
    if !temporary.insert(record.id.clone()) {
        violations.push(ChangeSetPlanViolation {
            path: record.path.clone(),
            line: record.depends_on_line.unwrap_or(record.id_line),
            kind: PlanKind::Cycle,
            summary: format!("ChangeSet dependency cycle includes {}", record.id),
        });
        return;
    }
    if let Some(deps) = graph.get(&record.id) {
        for dep in deps {
            if let Some(next) = records.iter().find(|candidate| &candidate.id == dep) {
                visit_cycle(next, graph, records, permanent, temporary, violations);
            }
        }
    }
    temporary.remove(&record.id);
    permanent.insert(record.id.clone());
}

fn detect_global_artifact_conflicts(
    records: &[PlanRecord],
    violations: &mut Vec<ChangeSetPlanViolation>,
) {
    let mut writers: BTreeMap<&str, Vec<&PlanRecord>> = BTreeMap::new();
    for record in records {
        for artifact in &record.writes_global_artifacts {
            writers.entry(artifact).or_default().push(record);
        }
    }
    for (artifact, records) in writers {
        for left_index in 0..records.len() {
            for right in records.iter().skip(left_index + 1) {
                let left = records[left_index];
                if !ordered_or_serialized(left, right) {
                    violations.push(ChangeSetPlanViolation {
                        path: left.path.clone(),
                        line: left.writes_global_artifacts_line.unwrap_or(left.id_line),
                        kind: PlanKind::GlobalArtifactConflict,
                        summary: format!(
                            "{} and {} both write global artifact {} without dependency or serialization edge",
                            left.id, right.id, artifact
                        ),
                    });
                }
            }
        }
    }
}

fn ordered_or_serialized(left: &PlanRecord, right: &PlanRecord) -> bool {
    left.depends_on.contains(&right.id)
        || right.depends_on.contains(&left.id)
        || (left.serializes_with.contains(&right.id) && right.serializes_with.contains(&left.id))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(path: &str, contents: &str) -> HonestClaimsDocument {
        HonestClaimsDocument {
            path: path.to_string(),
            contents: contents.to_string(),
        }
    }

    fn plan(path: &str, frontmatter: &str) -> ImplementationPlanDocument {
        ImplementationPlanDocument {
            path: path.to_string(),
            contents: format!("{frontmatter}\n# body\n"),
        }
    }

    fn base_plan(id: &str) -> String {
        format!(
            "---\ndoc_class: ImplementationPlan\nid: {id}\nexecution_unit: ChangeSet\nchangeset_contract: {CHANGESET_CONTRACT}\nchangeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable\n---"
        )
    }

    fn with_field(frontmatter: String, field: &str) -> String {
        frontmatter.replace("\n---", &format!("\n{field}\n---"))
    }

    #[test]
    fn honest_claims_accept_contextual_versions_and_examples() {
        let result = validate_honest_claims([
            doc(
                "docs/decisions/ADR-0129.md",
                "ADR cites agent-durable-goal v1.5.0 as a historical artifact.\nThe active LTS is v3.0.6 and the required tool is installed.\nConcrete tech is deferred to the per-product PRD; each PRD must name its matrix.\n```text\nrequired gate lands in v2\n```\nExample: required gate lands in v2.",
            ),
            doc(
                "specs/hyperscaler-gates.json",
                "claim_status: blocked_until_required_evidence_is_green",
            ),
        ]);
        assert!(result.is_ok());
    }

    #[test]
    fn honest_claims_reject_required_later_version_claim() {
        let violations = validate_honest_claims([doc(
            "docs/decisions/ADR-9999.md",
            "The required gate lands in v2.",
        )])
        .unwrap_err();
        assert_eq!(violations[0].kind, ClaimKind::DeferredRequiredClaim);
        assert_eq!(violations[0].phrase, "v2");
    }

    #[test]
    fn honest_claims_reject_required_future_work_claim() {
        let violations = validate_honest_claims([doc(
            "docs/decisions/ADR-9999.md",
            "The branch protection check is required; workflow wiring is future work.",
        )])
        .unwrap_err();
        assert_eq!(violations[0].kind, ClaimKind::DeferredRequiredClaim);
        assert_eq!(violations[0].phrase, "future work");
    }

    #[test]
    fn changeset_graph_accepts_valid_edges() {
        let a = with_field(
            base_plan("M01-P01-IP-001"),
            "depends_on_changesets: [\"M01-P01-IP-002\"]",
        );
        let report = validate_changeset_plan_graph([
            plan("IP-001.md", &a),
            plan("IP-002.md", &base_plan("M01-P01-IP-002")),
        ])
        .unwrap();
        assert_eq!(report.plans_checked, 2);
        assert_eq!(report.dependency_edges, 1);
    }

    #[test]
    fn changeset_graph_accepts_multiline_graph_lists() {
        let a = with_field(
            base_plan("M01-P01-IP-001"),
            "depends_on_changesets:\n  - M01-P01-IP-002",
        );
        let report = validate_changeset_plan_graph([
            plan("IP-001.md", &a),
            plan("IP-002.md", &base_plan("M01-P01-IP-002")),
        ])
        .unwrap();
        assert_eq!(report.plans_checked, 2);
        assert_eq!(report.dependency_edges, 1);
    }

    #[test]
    fn changeset_graph_rejects_duplicate_ids() {
        let violations = validate_changeset_plan_graph([
            plan("a.md", &base_plan("M01-P01-IP-001")),
            plan("b.md", &base_plan("M01-P01-IP-001")),
        ])
        .unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == PlanKind::DuplicateChangeSetId)
        );
    }

    #[test]
    fn changeset_graph_rejects_unknown_dependencies() {
        let a = with_field(
            base_plan("M01-P01-IP-001"),
            "depends_on_changesets: [\"M01-P01-IP-999\"]",
        );
        let violations = validate_changeset_plan_graph([plan("a.md", &a)]).unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == PlanKind::UnknownDependency)
        );
    }

    #[test]
    fn changeset_graph_rejects_cycles() {
        let a = with_field(
            base_plan("M01-P01-IP-001"),
            "depends_on_changesets: [\"M01-P01-IP-002\"]",
        );
        let b = with_field(
            base_plan("M01-P01-IP-002"),
            "depends_on_changesets: [\"M01-P01-IP-001\"]",
        );
        let violations =
            validate_changeset_plan_graph([plan("a.md", &a), plan("b.md", &b)]).unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == PlanKind::Cycle)
        );
    }

    #[test]
    fn changeset_graph_rejects_asymmetric_serialization() {
        let a = with_field(
            base_plan("M01-P01-IP-001"),
            "serializes_with_changesets: [\"M01-P01-IP-002\"]",
        );
        let violations = validate_changeset_plan_graph([
            plan("a.md", &a),
            plan("b.md", &base_plan("M01-P01-IP-002")),
        ])
        .unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == PlanKind::AsymmetricSerialization)
        );
    }

    #[test]
    fn changeset_graph_rejects_global_artifact_conflicts() {
        let a = with_field(
            base_plan("M01-P01-IP-001"),
            "writes_global_artifacts: [\"registry/fixuptasks.jsonl\"]",
        );
        let b = with_field(
            base_plan("M01-P01-IP-002"),
            "writes_global_artifacts: [\"registry/fixuptasks.jsonl\"]",
        );
        let violations =
            validate_changeset_plan_graph([plan("a.md", &a), plan("b.md", &b)]).unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == PlanKind::GlobalArtifactConflict)
        );
    }

    fn good_parity_target_fixture() -> Value {
        serde_json::json!({
            "parity_targets": [
                {
                    "target_id": "linux-libc-abi-c-ward",
                    "reference_kind": "linux_libc_abi_pessimistic_input",
                    "source_refs": [
                        {
                            "url": "https://github.com/sunfishcode/c-ward",
                            "pinned_ref": "main@2026-06-30-readme-inspection",
                            "retrieved_at": "2026-06-30T00:00:00Z",
                            "external_authority": false
                        }
                    ],
                    "source_tracking_state": "tracked_inventory",
                    "freshness_state": "fresh_at_collection",
                    "parity_posture": "pessimistic_reference_only",
                    "authority_boundary": "Benchmark and parity input only; first-party policy decides readiness.",
                    "validation_refs": ["libs/oya-check-honest-claims/src/lib.rs#validate_parity_target_source_tracking"],
                    "stop_conditions": ["source_changed_reopen_required", "missing_conformance_fixture"],
                    "userspace_abi_preserved": true,
                    "internal_kernel_api_stable": false,
                    "known_gaps": [
                        "getent runtime dependency",
                        "nightly dependency",
                        "incomplete implementation",
                        "linux-gnu-only ABI coverage"
                    ]
                }
            ]
        })
    }

    #[test]
    fn parity_target_source_tracking_accepts_good_fixture() {
        let report =
            validate_parity_target_source_tracking("good.json", &good_parity_target_fixture())
                .unwrap();
        assert_eq!(report.targets_checked, 1);
        assert_eq!(report.source_refs_checked, 1);
    }

    #[test]
    fn parity_target_source_tracking_rejects_external_authority() {
        let mut fixture = good_parity_target_fixture();
        fixture["parity_targets"][0]["source_refs"][0]["external_authority"] =
            serde_json::json!(true);

        let violations =
            validate_parity_target_source_tracking("bad-authority.json", &fixture).unwrap_err();

        assert!(violations.iter().any(
            |violation| violation.kind == ParityTargetSourceTrackingKind::ExternalAuthorityLeak
        ));
    }

    #[test]
    fn parity_target_source_tracking_rejects_duplicate_target_id() {
        let mut fixture = good_parity_target_fixture();
        let duplicate = fixture["parity_targets"][0].clone();
        fixture["parity_targets"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);

        let violations =
            validate_parity_target_source_tracking("duplicate.json", &fixture).unwrap_err();

        assert!(
            violations.iter().any(
                |violation| violation.kind == ParityTargetSourceTrackingKind::DuplicateTargetId
            )
        );
    }

    #[test]
    fn parity_target_source_tracking_requires_pinned_sources() {
        let mut fixture = good_parity_target_fixture();
        fixture["parity_targets"][0]["source_refs"][0]
            .as_object_mut()
            .unwrap()
            .remove("pinned_ref");

        let violations =
            validate_parity_target_source_tracking("missing-source.json", &fixture).unwrap_err();

        assert!(
            violations
                .iter()
                .any(|violation| violation.kind
                    == ParityTargetSourceTrackingKind::MissingPinnedSource)
        );
    }

    #[test]
    fn parity_target_source_tracking_rejects_embedded_authority_claim() {
        let mut fixture = good_parity_target_fixture();
        fixture["parity_targets"][0]["authority_boundary"] =
            serde_json::json!("external source is merge_authority for promotion decisions");

        let violations =
            validate_parity_target_source_tracking("embedded-authority.json", &fixture)
                .unwrap_err();

        assert!(
            violations.iter().any(|violation| violation.kind
                == ParityTargetSourceTrackingKind::PromotionAuthorityClaim)
        );
    }

    #[test]
    fn c_ward_linux_libc_abi_reference_is_pessimistic_only() {
        let mut fixture = good_parity_target_fixture();
        fixture["parity_targets"][0]["internal_kernel_api_stable"] = serde_json::json!(true);
        fixture["parity_targets"][0]["parity_posture"] = serde_json::json!("production ready");

        let violations =
            validate_parity_target_source_tracking("cward-optimistic.json", &fixture).unwrap_err();

        assert!(
            violations.iter().any(|violation| violation.kind
                == ParityTargetSourceTrackingKind::OptimisticKernelAbiClaim)
        );
    }
    fn reference_doc(contents: &str) -> ReferenceFamilyDocument {
        ReferenceFamilyDocument {
            path: "specs/reference-parity.md".to_string(),
            contents: contents.to_string(),
        }
    }

    fn good_reference_family_fixture() -> &'static str {
        "---\n\
reference_families:\n\
  - c-ward\n\
  - Kubernetes\n\
  - Argo\n\
  - GitHub/GitLab adapters\n\
  - Snyk/Trivy/drift\n\
  - Linux/OCI\n\
---\n\
Authority: Oyatie owns product authority; these are bridge/reference families only.\n\
GitHub and GitLab are adapter examples, not authority.\n"
    }

    #[test]
    fn reference_family_parity_accepts_full_directive_fixture() {
        let report =
            validate_reference_family_parity([reference_doc(good_reference_family_fixture())])
                .unwrap();
        assert_eq!(report.documents_checked, 1);
        assert_eq!(report.reference_families_checked, 6);
    }
    #[test]
    fn reference_family_parity_accepts_markdown_heading_fixture() {
        let report = validate_reference_family_parity([reference_doc(
            "## Reference families\n\
- c-ward\n\
- Kubernetes\n\
- Argo\n\
- GitHub/GitLab adapters\n\
- Snyk/Trivy/drift\n\
- Linux/OCI\n",
        )])
        .unwrap();
        assert_eq!(report.reference_families_checked, 6);
    }

    #[test]
    fn reference_family_parity_rejects_missing_reference_family() {
        let violations = validate_reference_family_parity([reference_doc(
            "GitHub adapter parity covers the target without declaring the required comparison set.",
        )])
        .unwrap_err();
        assert!(
            violations.iter().any(
                |violation| violation.kind == ReferenceFamilyParityKind::MissingReferenceFamily
            )
        );
    }

    #[test]
    fn reference_family_parity_rejects_github_only_adapter_overfit() {
        let fixture =
            good_reference_family_fixture().replace("GitHub/GitLab adapters", "GitHub adapter");
        let violations = validate_reference_family_parity([reference_doc(&fixture)]).unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind
                    == ReferenceFamilyParityKind::GithubOnlyAdapterOverfit)
        );
    }

    #[test]
    fn reference_family_parity_rejects_adapter_as_authority() {
        let fixture = format!(
            "{}\nGitHub is the authority for merge admission.\n",
            good_reference_family_fixture()
        );
        let violations = validate_reference_family_parity([reference_doc(&fixture)]).unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == ReferenceFamilyParityKind::AdapterAsAuthority)
        );
    }
    #[test]
    fn reference_family_parity_rejects_mixed_authority_and_negation_line() {
        let fixture = format!(
            "{}\nGitHub is the authority for merge admission; GitLab is not product authority.\n",
            good_reference_family_fixture()
        );
        let violations = validate_reference_family_parity([reference_doc(&fixture)]).unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.family == "GitHub"
                    && violation.kind == ReferenceFamilyParityKind::AdapterAsAuthority)
        );
    }

    #[test]
    fn reference_family_parity_rejects_source_of_truth_wording() {
        let fixture = format!(
            "{}\nKubernetes is the source of truth for control-plane parity.\n",
            good_reference_family_fixture()
        );
        let violations = validate_reference_family_parity([reference_doc(&fixture)]).unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.family == "Kubernetes"
                    && violation.kind == ReferenceFamilyParityKind::AdapterAsAuthority)
        );
    }

    #[test]
    fn reference_family_parity_rejects_same_alias_mixed_authority_and_negation() {
        let fixture = format!(
            "{}\nGitHub is the authority for merge admission; GitHub is not product authority.\n",
            good_reference_family_fixture()
        );
        let violations = validate_reference_family_parity([reference_doc(&fixture)]).unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.family == "GitHub"
                    && violation.kind == ReferenceFamilyParityKind::AdapterAsAuthority)
        );
    }
}
