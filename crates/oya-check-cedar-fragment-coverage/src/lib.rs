//! Cedar fragment coverage validator — enforces invariants C01..C04 from
//! `registry/cedar-fragments.json`. Pure std-only kernel: takes
//! parsed inputs as data, returns ValidationReport with violations.
//!
//! Closes the drift loop between OpenAPI contracts (which reference Cedar
//! fragments via `cedar_fragments[]`) and the actual `.cedar` files on disk.
//! Without this, contracts can name fragments that nobody ever writes —
//! the registry sits paper-only.
//!
//! Invariants (see `registry/cedar-fragments.json::drift_invariants`):
//!   C01: every `cedar_fragments[]` reference in `contracts/*.openapi.yaml`
//!        must resolve to a `fragment_id` in the registry.
//!   C02: every `cedar_fragments_planned[]` reference in `bounded-contexts.json`
//!        must resolve to a `fragment_id` in the registry.
//!   C03: every `.cedar` file under `.omc/cedar/` must appear as a `fragment_id`
//!        with status=operational.
//!   C04: `status=operational` ↔ `fragment_path_planned` exists on disk
//!        (consistency between declared status and physical state).

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FragmentStatus {
    Operational,
    Planned,
    BlockedByFoundationPrerequisite,
}

impl FragmentStatus {
    pub fn name(self) -> &'static str {
        match self {
            FragmentStatus::Operational => "operational",
            FragmentStatus::Planned => "planned",
            FragmentStatus::BlockedByFoundationPrerequisite => "blocked-by-foundation-prerequisite",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FragmentRow {
    pub fragment_id: String,
    pub fragment_path_planned: String,
    pub status: FragmentStatus,
}

/// Inputs to the validator. Caller is responsible for parsing JSON/YAML and
/// running `git ls-files` (or equivalent) to populate these fields.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CoverageInputs {
    pub registry_rows: Vec<FragmentRow>,
    /// Fragment IDs cited by contracts/*.openapi.yaml `cedar_fragments[]` arrays.
    pub openapi_references: BTreeSet<String>,
    /// Fragment IDs cited by `registry/bounded-contexts.json`
    /// `cedar_fragments_planned[]` arrays (after `(M02-P20 …)` parens are stripped).
    pub bc_references: BTreeSet<String>,
    /// Paths of `.cedar` files under `.omc/cedar/` that are HEAD-tracked.
    pub cedar_files_on_disk: BTreeSet<String>,
    /// Paths that are HEAD-tracked (used for C04 path-exists check).
    pub head_tracked_paths: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Violation {
    /// C01: contract names a fragment_id that has no registry row.
    C01UnknownOpenapiReference { fragment_id: String },
    /// C02: BC registry names a fragment_id that has no registry row.
    C02UnknownBcReference { fragment_id: String },
    /// C03: `.cedar` file on disk has no registry row (orphan).
    C03OrphanCedarFile { path: String },
    /// C03: `.cedar` file on disk has a registry row, but its status is not operational.
    C03CedarFileStatusMismatch {
        fragment_id: String,
        actual_status: FragmentStatus,
    },
    /// C04: status=operational but path missing on disk.
    C04OperationalPathMissing { fragment_id: String, path: String },
    /// C04: status≠operational but path exists on disk.
    C04NonOperationalPathExists {
        fragment_id: String,
        path: String,
        status: FragmentStatus,
    },
    /// Duplicate fragment_id within the registry itself.
    DuplicateFragmentId { fragment_id: String },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ValidationReport {
    pub violations: Vec<Violation>,
    pub rows_seen: usize,
    pub openapi_references_seen: usize,
    pub bc_references_seen: usize,
    pub cedar_files_seen: usize,
}

impl ValidationReport {
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

pub fn validate(inputs: &CoverageInputs) -> ValidationReport {
    let mut violations = Vec::new();
    let mut by_id: BTreeMap<&str, &FragmentRow> = BTreeMap::new();

    for row in &inputs.registry_rows {
        if by_id.insert(row.fragment_id.as_str(), row).is_some() {
            violations.push(Violation::DuplicateFragmentId {
                fragment_id: row.fragment_id.clone(),
            });
        }
    }

    // C01
    for cited in &inputs.openapi_references {
        if !by_id.contains_key(cited.as_str()) {
            violations.push(Violation::C01UnknownOpenapiReference {
                fragment_id: cited.clone(),
            });
        }
    }

    // C02
    for cited in &inputs.bc_references {
        if !by_id.contains_key(cited.as_str()) {
            violations.push(Violation::C02UnknownBcReference {
                fragment_id: cited.clone(),
            });
        }
    }

    // C03: every .cedar file on disk must have a row with status=operational
    let registry_paths: BTreeMap<&str, &FragmentRow> = inputs
        .registry_rows
        .iter()
        .map(|r| (r.fragment_path_planned.as_str(), r))
        .collect();
    for path in &inputs.cedar_files_on_disk {
        match registry_paths.get(path.as_str()) {
            None => violations.push(Violation::C03OrphanCedarFile { path: path.clone() }),
            Some(row) if !matches!(row.status, FragmentStatus::Operational) => {
                violations.push(Violation::C03CedarFileStatusMismatch {
                    fragment_id: row.fragment_id.clone(),
                    actual_status: row.status,
                });
            }
            Some(_) => {}
        }
    }

    // C04
    for row in &inputs.registry_rows {
        let path_exists = inputs
            .head_tracked_paths
            .contains(&row.fragment_path_planned);
        match (row.status, path_exists) {
            (FragmentStatus::Operational, false) => {
                violations.push(Violation::C04OperationalPathMissing {
                    fragment_id: row.fragment_id.clone(),
                    path: row.fragment_path_planned.clone(),
                });
            }
            (FragmentStatus::Planned, true)
            | (FragmentStatus::BlockedByFoundationPrerequisite, true) => {
                violations.push(Violation::C04NonOperationalPathExists {
                    fragment_id: row.fragment_id.clone(),
                    path: row.fragment_path_planned.clone(),
                    status: row.status,
                });
            }
            _ => {}
        }
    }

    ValidationReport {
        violations,
        rows_seen: inputs.registry_rows.len(),
        openapi_references_seen: inputs.openapi_references.len(),
        bc_references_seen: inputs.bc_references.len(),
        cedar_files_seen: inputs.cedar_files_on_disk.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str, path: &str, status: FragmentStatus) -> FragmentRow {
        FragmentRow {
            fragment_id: id.into(),
            fragment_path_planned: path.into(),
            status,
        }
    }

    fn baseline_inputs() -> CoverageInputs {
        CoverageInputs {
            registry_rows: vec![
                row(
                    "ops-internal-public",
                    ".omc/cedar/ops-internal-public.cedar",
                    FragmentStatus::BlockedByFoundationPrerequisite,
                ),
                row(
                    "ops-tenant-private",
                    ".omc/cedar/ops-tenant-private.cedar",
                    FragmentStatus::Planned,
                ),
            ],
            openapi_references: BTreeSet::new(),
            bc_references: BTreeSet::new(),
            cedar_files_on_disk: BTreeSet::new(),
            head_tracked_paths: BTreeSet::new(),
        }
    }

    #[test]
    fn baseline_is_clean() {
        let report = validate(&baseline_inputs());
        assert!(report.is_clean(), "violations: {:?}", report.violations);
        assert_eq!(report.rows_seen, 2);
    }

    #[test]
    fn c01_unknown_openapi_reference() {
        let mut inputs = baseline_inputs();
        inputs
            .openapi_references
            .insert("ops-nonexistent".to_string());
        let report = validate(&inputs);
        assert!(matches!(
            report.violations.as_slice(),
            [Violation::C01UnknownOpenapiReference { fragment_id }] if fragment_id == "ops-nonexistent"
        ));
    }

    #[test]
    fn c01_known_openapi_reference_clean() {
        let mut inputs = baseline_inputs();
        inputs
            .openapi_references
            .insert("ops-internal-public".to_string());
        let report = validate(&inputs);
        assert!(report.is_clean());
    }

    #[test]
    fn c02_unknown_bc_reference() {
        let mut inputs = baseline_inputs();
        inputs
            .bc_references
            .insert("ops-missing-fragment".to_string());
        let report = validate(&inputs);
        assert!(matches!(
            report.violations.as_slice(),
            [Violation::C02UnknownBcReference { fragment_id }] if fragment_id == "ops-missing-fragment"
        ));
    }

    #[test]
    fn c03_orphan_cedar_file() {
        let mut inputs = baseline_inputs();
        inputs
            .cedar_files_on_disk
            .insert(".omc/cedar/ops-rogue.cedar".to_string());
        let report = validate(&inputs);
        assert!(matches!(
            report.violations.as_slice(),
            [Violation::C03OrphanCedarFile { path }] if path == ".omc/cedar/ops-rogue.cedar"
        ));
    }

    #[test]
    fn c03_cedar_file_status_mismatch() {
        // A .cedar file exists on disk but its row says status=Planned.
        // Per C03, every file on disk must have status=Operational.
        let mut inputs = baseline_inputs();
        inputs
            .cedar_files_on_disk
            .insert(".omc/cedar/ops-tenant-private.cedar".to_string());
        inputs
            .head_tracked_paths
            .insert(".omc/cedar/ops-tenant-private.cedar".to_string());
        let report = validate(&inputs);
        // C03 mismatch + C04 NonOperationalPathExists both fire — that's correct;
        // both invariants are independently violated by the same fact.
        assert!(report.violations.iter().any(|v| matches!(
            v,
            Violation::C03CedarFileStatusMismatch {
                fragment_id, actual_status: FragmentStatus::Planned
            } if fragment_id == "ops-tenant-private"
        )));
    }

    #[test]
    fn c04_operational_path_missing() {
        let mut inputs = CoverageInputs::default();
        inputs.registry_rows.push(row(
            "ops-internal-public",
            ".omc/cedar/ops-internal-public.cedar",
            FragmentStatus::Operational,
        ));
        // head_tracked_paths is empty → path is missing.
        let report = validate(&inputs);
        assert!(matches!(
            report.violations.as_slice(),
            [Violation::C04OperationalPathMissing { fragment_id, .. }] if fragment_id == "ops-internal-public"
        ));
    }

    #[test]
    fn c04_non_operational_path_exists() {
        let mut inputs = baseline_inputs();
        inputs
            .head_tracked_paths
            .insert(".omc/cedar/ops-tenant-private.cedar".to_string());
        let report = validate(&inputs);
        assert!(report.violations.iter().any(|v| matches!(
            v,
            Violation::C04NonOperationalPathExists {
                fragment_id, status: FragmentStatus::Planned, ..
            } if fragment_id == "ops-tenant-private"
        )));
    }

    #[test]
    fn duplicate_fragment_id() {
        let mut inputs = CoverageInputs::default();
        inputs
            .registry_rows
            .push(row("dup", ".omc/cedar/dup.cedar", FragmentStatus::Planned));
        inputs.registry_rows.push(row(
            "dup",
            ".omc/cedar/dup-other.cedar",
            FragmentStatus::Planned,
        ));
        let report = validate(&inputs);
        assert!(matches!(
            report.violations.as_slice(),
            [Violation::DuplicateFragmentId { fragment_id }] if fragment_id == "dup"
        ));
    }

    #[test]
    fn fragment_status_name_round_trip() {
        let names: Vec<&str> = [
            FragmentStatus::Operational,
            FragmentStatus::Planned,
            FragmentStatus::BlockedByFoundationPrerequisite,
        ]
        .iter()
        .map(|s| s.name())
        .collect();
        assert_eq!(
            names,
            vec![
                "operational",
                "planned",
                "blocked-by-foundation-prerequisite"
            ]
        );
    }

    #[test]
    fn report_counters_populated() {
        let mut inputs = baseline_inputs();
        inputs
            .openapi_references
            .insert("ops-internal-public".to_string());
        inputs
            .bc_references
            .insert("ops-tenant-private".to_string());
        inputs
            .cedar_files_on_disk
            .insert(".omc/cedar/ops-internal-public.cedar".to_string());
        // The file-on-disk reference should match the existing row's path; mark
        // it operational so C03 passes, and add path to HEAD so C04 passes.
        inputs.registry_rows[0].status = FragmentStatus::Operational;
        inputs
            .head_tracked_paths
            .insert(".omc/cedar/ops-internal-public.cedar".to_string());
        let report = validate(&inputs);
        assert!(report.is_clean(), "violations: {:?}", report.violations);
        assert_eq!(report.rows_seen, 2);
        assert_eq!(report.openapi_references_seen, 1);
        assert_eq!(report.bc_references_seen, 1);
        assert_eq!(report.cedar_files_seen, 1);
    }
}
