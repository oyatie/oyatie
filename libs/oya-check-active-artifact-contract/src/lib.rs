//! Active machine-readable artifact contract validator kernel.
//!
//! Pure std-only kernel implementing the v3.0.0 artifact-capabilities-registry
//! validation rules per ADR-0069 + `specs/active-machine-readable-artifact-contract.json`.
//!
//! The kernel is I/O-free: it takes pre-parsed `ArtifactRow` values and a set of
//! HEAD-tracked paths (resolved by the runtime via `git ls-files`), and returns
//! a `ValidationReport` of violations. The runtime (e.g., `oya-dev-cli`) is
//! responsible for JSON parsing, file I/O, and `git ls-files` invocation; per
//! ADR-0015 the kernel layer has no outbound I/O.

use std::collections::{BTreeMap, BTreeSet};

/// Closed enum of artifact profiles per `specs/artifact-profile-defaults.json`.
/// Each profile bundles default capability declarations; registry rows declare
/// `artifact_profile` + sparse `capability_overrides` rather than full 9-cap boilerplate.
/// Closes architect r17 finding #8.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ArtifactProfile {
    Schema,
    Spec,
    Registry,
    Template,
    PlanAttestation,
    Ledger,
    ClaimMatrix,
    EvidenceBundle,
    KernelCrate,
    CiLane,
    Verifier,
}

impl ArtifactProfile {
    pub fn all() -> [ArtifactProfile; 11] {
        [
            ArtifactProfile::Schema,
            ArtifactProfile::Spec,
            ArtifactProfile::Registry,
            ArtifactProfile::Template,
            ArtifactProfile::PlanAttestation,
            ArtifactProfile::Ledger,
            ArtifactProfile::ClaimMatrix,
            ArtifactProfile::EvidenceBundle,
            ArtifactProfile::KernelCrate,
            ArtifactProfile::CiLane,
            ArtifactProfile::Verifier,
        ]
    }

    pub fn parse(s: &str) -> Option<ArtifactProfile> {
        match s {
            "schema" => Some(ArtifactProfile::Schema),
            "spec" => Some(ArtifactProfile::Spec),
            "registry" => Some(ArtifactProfile::Registry),
            "template" => Some(ArtifactProfile::Template),
            "plan-attestation" => Some(ArtifactProfile::PlanAttestation),
            "ledger" => Some(ArtifactProfile::Ledger),
            "claim-matrix" => Some(ArtifactProfile::ClaimMatrix),
            "evidence-bundle" => Some(ArtifactProfile::EvidenceBundle),
            "kernel-crate" => Some(ArtifactProfile::KernelCrate),
            "ci-lane" => Some(ArtifactProfile::CiLane),
            "verifier" => Some(ArtifactProfile::Verifier),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ArtifactProfile::Schema => "schema",
            ArtifactProfile::Spec => "spec",
            ArtifactProfile::Registry => "registry",
            ArtifactProfile::Template => "template",
            ArtifactProfile::PlanAttestation => "plan-attestation",
            ArtifactProfile::Ledger => "ledger",
            ArtifactProfile::ClaimMatrix => "claim-matrix",
            ArtifactProfile::EvidenceBundle => "evidence-bundle",
            ArtifactProfile::KernelCrate => "kernel-crate",
            ArtifactProfile::CiLane => "ci-lane",
            ArtifactProfile::Verifier => "verifier",
        }
    }

    /// Profile defaults baked into the kernel for testability. The canonical
    /// authority is `specs/artifact-profile-defaults.json`; runtimes
    /// that load the JSON should use that file and MAY use this baseline only
    /// as a sanity-check fallback. The default for every capability in every
    /// profile is `Planned`; runtime is responsible for richer defaults.
    pub fn default_capabilities(self) -> BTreeMap<CapabilityKind, CapabilityStatus> {
        let _ = self; // every profile defaults all 9 capabilities to Planned
        let mut map = BTreeMap::new();
        for kind in CapabilityKind::ALL {
            map.insert(kind, CapabilityStatus::Planned);
        }
        map
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CapabilityStatus {
    Operational,
    Planned,
    BlockedByFoundation,
    NotApplicable,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CapabilityKind {
    Enforcement,
    Verification,
    Validation,
    Autogen,
    Selfheal,
    Selfupdate,
    Selfmaintain,
    Telemetry,
    Provenance,
}

impl CapabilityKind {
    /// All 9 capabilities the v3.0.0 contract requires per registry row.
    pub const ALL: [CapabilityKind; 9] = [
        CapabilityKind::Enforcement,
        CapabilityKind::Verification,
        CapabilityKind::Validation,
        CapabilityKind::Autogen,
        CapabilityKind::Selfheal,
        CapabilityKind::Selfupdate,
        CapabilityKind::Selfmaintain,
        CapabilityKind::Telemetry,
        CapabilityKind::Provenance,
    ];

    pub fn name(self) -> &'static str {
        match self {
            CapabilityKind::Enforcement => "enforcement",
            CapabilityKind::Verification => "verification",
            CapabilityKind::Validation => "validation",
            CapabilityKind::Autogen => "autogen",
            CapabilityKind::Selfheal => "selfheal",
            CapabilityKind::Selfupdate => "selfupdate",
            CapabilityKind::Selfmaintain => "selfmaintain",
            CapabilityKind::Telemetry => "telemetry",
            CapabilityKind::Provenance => "provenance",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityDeclaration {
    pub status: CapabilityStatus, // data_class: INTERNAL_ONLY
    /// Resolvable anchor proving evidence (crate path / lane id / generator path).
    /// Required when status == Operational; informational when Planned.
    pub evidence_ref: Option<String>, // data_class: INTERNAL_ONLY
    /// What must land for status to flip to Operational. Required when Planned
    /// or BlockedByFoundation; empty otherwise.
    pub prerequisite_for_operational: Vec<String>, // data_class: INTERNAL_ONLY
    /// Required when status == NotApplicable.
    pub not_applicable_rationale: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactRow {
    pub artifact_id: String,      // data_class: INTERNAL_ONLY
    pub artifact_path: String,    // data_class: INTERNAL_ONLY
    pub artifact_format: String,  // data_class: INTERNAL_ONLY
    pub contract_version: String, // data_class: INTERNAL_ONLY
    pub capabilities: BTreeMap<CapabilityKind, CapabilityDeclaration>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Severity {
    Error,
    Warn,
    Info,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Violation {
    pub artifact_id: String,   // data_class: INTERNAL_ONLY
    pub rule_id: &'static str, // data_class: INTERNAL_ONLY
    pub severity: Severity,    // data_class: INTERNAL_ONLY
    pub message: String,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ValidationReport {
    pub violations: Vec<Violation>,          // data_class: INTERNAL_ONLY
    pub artifact_ids_seen: BTreeSet<String>, // data_class: INTERNAL_ONLY
    pub head_tracked_count: usize,           // data_class: INTERNAL_ONLY
    pub untracked_count: usize,              // data_class: INTERNAL_ONLY
    pub operational_caps: usize,             // data_class: INTERNAL_ONLY
    pub planned_caps: usize,                 // data_class: INTERNAL_ONLY
    pub blocked_caps: usize,                 // data_class: INTERNAL_ONLY
    pub not_applicable_caps: usize,          // data_class: INTERNAL_ONLY
}

impl ValidationReport {
    pub fn error_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == Severity::Error)
            .count()
    }

    pub fn has_errors(&self) -> bool {
        self.error_count() > 0
    }
}

/// Validate the registry against the v3.0.0 contract rules.
///
/// `rows` — registry rows pre-parsed by the runtime.
/// `head_tracked_paths` — set of paths returned by `git ls-files` (from runtime).
///
/// Rules enforced (subset; architect r17 recommended-next-action minimum):
/// - R01: every `artifact_path` is HEAD-tracked OR row carries `status=planned` for a foundation gap
/// - R02: no duplicate `artifact_id` across rows
/// - R03: every row declares all 9 capabilities (none missing)
/// - R04: every `status=Operational` capability has a non-empty `evidence_ref`
/// - R05: every `status=Planned` capability has populated `prerequisite_for_operational`
/// - R06: every `status=BlockedByFoundation` capability has populated `prerequisite_for_operational`
/// - R07: every `status=NotApplicable` capability has populated `not_applicable_rationale`
pub fn validate(rows: &[ArtifactRow], head_tracked_paths: &BTreeSet<String>) -> ValidationReport {
    let mut report = ValidationReport::default();
    let mut id_first_seen_index: BTreeMap<String, usize> = BTreeMap::new();
    report.head_tracked_count = head_tracked_paths.len();

    for (index, row) in rows.iter().enumerate() {
        // R02: duplicate artifact_id
        if let Some(prior) = id_first_seen_index.get(&row.artifact_id) {
            report.violations.push(Violation {
                artifact_id: row.artifact_id.clone(),
                rule_id: "R02-duplicate-artifact-id",
                severity: Severity::Error,
                message: format!(
                    "artifact_id `{}` first seen at row index {prior}; duplicate at row {index}",
                    row.artifact_id
                ),
            });
        } else {
            id_first_seen_index.insert(row.artifact_id.clone(), index);
            report.artifact_ids_seen.insert(row.artifact_id.clone());
        }

        // R01: artifact_path HEAD-tracked
        let path_is_tracked = head_tracked_paths.contains(&row.artifact_path);
        if !path_is_tracked {
            report.untracked_count += 1;
            report.violations.push(Violation {
                artifact_id: row.artifact_id.clone(),
                rule_id: "R01-artifact-path-not-in-head",
                severity: Severity::Error,
                message: format!(
                    "artifact_path `{}` is not in HEAD (verified via `git ls-files`)",
                    row.artifact_path
                ),
            });
        }

        // R03: all 9 capabilities present
        for kind in CapabilityKind::ALL {
            if !row.capabilities.contains_key(&kind) {
                report.violations.push(Violation {
                    artifact_id: row.artifact_id.clone(),
                    rule_id: "R03-missing-capability",
                    severity: Severity::Error,
                    message: format!("capability `{}` is missing", kind.name()),
                });
            }
        }

        // R04 – R07: status-specific invariants
        for (kind, decl) in &row.capabilities {
            match decl.status {
                CapabilityStatus::Operational => {
                    report.operational_caps += 1;
                    if decl
                        .evidence_ref
                        .as_ref()
                        .map(|s| s.trim().is_empty())
                        .unwrap_or(true)
                    {
                        report.violations.push(Violation {
                            artifact_id: row.artifact_id.clone(),
                            rule_id: "R04-operational-without-evidence",
                            severity: Severity::Error,
                            message: format!(
                                "capability `{}` is Operational but evidence_ref is empty",
                                kind.name()
                            ),
                        });
                    }
                }
                CapabilityStatus::Planned => {
                    report.planned_caps += 1;
                    if decl.prerequisite_for_operational.is_empty() {
                        report.violations.push(Violation {
                            artifact_id: row.artifact_id.clone(),
                            rule_id: "R05-planned-without-prerequisite",
                            severity: Severity::Error,
                            message: format!(
                                "capability `{}` is Planned but prerequisite_for_operational is empty",
                                kind.name()
                            ),
                        });
                    }
                }
                CapabilityStatus::BlockedByFoundation => {
                    report.blocked_caps += 1;
                    if decl.prerequisite_for_operational.is_empty() {
                        report.violations.push(Violation {
                            artifact_id: row.artifact_id.clone(),
                            rule_id: "R06-blocked-without-foundation-prerequisite",
                            severity: Severity::Error,
                            message: format!(
                                "capability `{}` is BlockedByFoundation but prerequisite_for_operational is empty",
                                kind.name()
                            ),
                        });
                    }
                }
                CapabilityStatus::NotApplicable => {
                    report.not_applicable_caps += 1;
                    if decl
                        .not_applicable_rationale
                        .as_ref()
                        .map(|s| s.trim().is_empty())
                        .unwrap_or(true)
                    {
                        report.violations.push(Violation {
                            artifact_id: row.artifact_id.clone(),
                            rule_id: "R07-not-applicable-without-rationale",
                            severity: Severity::Warn,
                            message: format!(
                                "capability `{}` is NotApplicable but rationale is empty",
                                kind.name()
                            ),
                        });
                    }
                }
            }
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cap(
        status: CapabilityStatus,
        evidence: Option<&str>,
        prereq: &[&str],
        rationale: Option<&str>,
    ) -> CapabilityDeclaration {
        CapabilityDeclaration {
            status,
            evidence_ref: evidence.map(str::to_string),
            prerequisite_for_operational: prereq.iter().map(|s| s.to_string()).collect(),
            not_applicable_rationale: rationale.map(str::to_string),
        }
    }

    fn full_caps(
        status: CapabilityStatus,
        evidence: Option<&str>,
        prereq: &[&str],
        rationale: Option<&str>,
    ) -> BTreeMap<CapabilityKind, CapabilityDeclaration> {
        let mut map = BTreeMap::new();
        for kind in CapabilityKind::ALL {
            map.insert(kind, cap(status, evidence, prereq, rationale));
        }
        map
    }

    fn row(
        id: &str,
        path: &str,
        caps: BTreeMap<CapabilityKind, CapabilityDeclaration>,
    ) -> ArtifactRow {
        ArtifactRow {
            artifact_id: id.into(),
            artifact_path: path.into(),
            artifact_format: "json".into(),
            contract_version: "v3.0.0".into(),
            capabilities: caps,
        }
    }

    #[test]
    fn empty_input_is_clean() {
        let head: BTreeSet<String> = BTreeSet::new();
        let report = validate(&[], &head);
        assert!(report.violations.is_empty());
        assert_eq!(report.head_tracked_count, 0);
    }

    #[test]
    fn r01_flags_path_not_in_head() {
        let mut head = BTreeSet::new();
        head.insert("docs/CONSTITUTION.md".into());
        let rows = vec![row(
            "ops-portal-ledger",
            "evidence/ledger/ops-portal-ledger.json",
            full_caps(
                CapabilityStatus::Planned,
                None,
                &["crates/oya-check-active-artifact-contract"],
                None,
            ),
        )];
        let report = validate(&rows, &head);
        let r01_count = report
            .violations
            .iter()
            .filter(|v| v.rule_id == "R01-artifact-path-not-in-head")
            .count();
        assert_eq!(r01_count, 1);
        assert_eq!(report.untracked_count, 1);
    }

    #[test]
    fn r02_flags_duplicate_artifact_id() {
        let mut head = BTreeSet::new();
        head.insert("specs/x.json".into());
        let caps = full_caps(CapabilityStatus::Planned, None, &["validator-crate"], None);
        let rows = vec![
            row("dup", "specs/x.json", caps.clone()),
            row("dup", "specs/x.json", caps),
        ];
        let report = validate(&rows, &head);
        assert_eq!(
            report
                .violations
                .iter()
                .filter(|v| v.rule_id == "R02-duplicate-artifact-id")
                .count(),
            1
        );
    }

    #[test]
    fn r03_flags_missing_capabilities() {
        let mut head = BTreeSet::new();
        head.insert("specs/x.json".into());
        let mut caps = BTreeMap::new();
        caps.insert(
            CapabilityKind::Enforcement,
            cap(CapabilityStatus::Planned, None, &["x"], None),
        );
        let rows = vec![row("partial", "specs/x.json", caps)];
        let report = validate(&rows, &head);
        let r03_count = report
            .violations
            .iter()
            .filter(|v| v.rule_id == "R03-missing-capability")
            .count();
        assert_eq!(r03_count, 8);
    }

    #[test]
    fn r04_flags_operational_without_evidence() {
        let mut head = BTreeSet::new();
        head.insert("specs/x.json".into());
        let caps = full_caps(CapabilityStatus::Operational, None, &[], None);
        let rows = vec![row("op", "specs/x.json", caps)];
        let report = validate(&rows, &head);
        let r04 = report
            .violations
            .iter()
            .filter(|v| v.rule_id == "R04-operational-without-evidence")
            .count();
        assert_eq!(r04, 9);
    }

    #[test]
    fn r05_flags_planned_without_prerequisite() {
        let mut head = BTreeSet::new();
        head.insert("specs/x.json".into());
        let caps = full_caps(CapabilityStatus::Planned, None, &[], None);
        let rows = vec![row("p", "specs/x.json", caps)];
        let report = validate(&rows, &head);
        let r05 = report
            .violations
            .iter()
            .filter(|v| v.rule_id == "R05-planned-without-prerequisite")
            .count();
        assert_eq!(r05, 9);
    }

    #[test]
    fn r06_flags_blocked_without_prerequisite() {
        let mut head = BTreeSet::new();
        head.insert("specs/x.json".into());
        let caps = full_caps(CapabilityStatus::BlockedByFoundation, None, &[], None);
        let rows = vec![row("b", "specs/x.json", caps)];
        let report = validate(&rows, &head);
        let r06 = report
            .violations
            .iter()
            .filter(|v| v.rule_id == "R06-blocked-without-foundation-prerequisite")
            .count();
        assert_eq!(r06, 9);
    }

    #[test]
    fn r07_warns_na_without_rationale() {
        let mut head = BTreeSet::new();
        head.insert("specs/x.json".into());
        let caps = full_caps(CapabilityStatus::NotApplicable, None, &[], None);
        let rows = vec![row("n", "specs/x.json", caps)];
        let report = validate(&rows, &head);
        let r07 = report
            .violations
            .iter()
            .filter(|v| v.rule_id == "R07-not-applicable-without-rationale")
            .count();
        assert_eq!(r07, 9);
        assert_eq!(report.error_count(), 0); // R07 is Warn, not Error
    }

    #[test]
    fn artifact_profile_parses_all_known_names() {
        let names = [
            "schema",
            "spec",
            "registry",
            "template",
            "plan-attestation",
            "ledger",
            "claim-matrix",
            "evidence-bundle",
            "kernel-crate",
            "ci-lane",
            "verifier",
        ];
        for n in names {
            assert!(ArtifactProfile::parse(n).is_some(), "should parse: {n}");
        }
        assert!(ArtifactProfile::parse("not-a-profile").is_none());
    }

    #[test]
    fn artifact_profile_round_trips() {
        for p in ArtifactProfile::all() {
            assert_eq!(ArtifactProfile::parse(p.name()), Some(p));
        }
    }

    #[test]
    fn artifact_profile_default_capabilities_covers_all_9() {
        for p in ArtifactProfile::all() {
            let defaults = p.default_capabilities();
            assert_eq!(
                defaults.len(),
                9,
                "profile {} missing capabilities",
                p.name()
            );
            for kind in CapabilityKind::ALL {
                assert!(
                    defaults.contains_key(&kind),
                    "profile {} missing {}",
                    p.name(),
                    kind.name()
                );
            }
        }
    }

    #[test]
    fn happy_path_no_violations() {
        let mut head = BTreeSet::new();
        head.insert("specs/x.json".into());
        let caps = full_caps(
            CapabilityStatus::Planned,
            None,
            &["crates/oya-check-active-artifact-contract"],
            None,
        );
        let rows = vec![row("ok", "specs/x.json", caps)];
        let report = validate(&rows, &head);
        assert_eq!(report.violations.len(), 0);
        assert!(!report.has_errors());
        assert_eq!(report.planned_caps, 9);
    }
}
