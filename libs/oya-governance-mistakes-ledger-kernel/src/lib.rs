//! Foundry mistakes-ledger fitness kernel.
//!
//! # Naming justification
//!
//! - Crate `oya-governance-mistakes-ledger-kernel` —
//!   v4 BNF `oya-<product:foundry>-<facet:fitness>-<topic:mistakes-ledger>-<layer:kernel>`;
//!   13-layer-enum suffix `kernel` (innermost ring: I/O-free port + pure check
//!   functions per ADR-0056 "port-in-kernel"). The topic `mistakes-ledger`
//!   matches `docs/MISTAKES-LEDGER.md`, `docs/templates/mistakes-ledger-row-template.md`,
//!   and `registry/mistakes-ledger.json` — the canonical
//!   Control-2 storage path per `feedback_repeat_mistake_prevention` memory.
//! - Future dev-CLI `oya-governance-mistakes-ledger-app` —
//!   v4 BNF `oya-<product:foundry>-<facet:fitness>-<topic:mistakes-ledger>-<layer:app>`;
//!   binary tool surface (canonical `app` suffix per ADR-0107 amendment 2026-05-15),
//!   wraps the kernel for the mistakes-ledger Rust gate packet; legacy
//!   `oya gate validate mistakes-ledger` calls are local bridge/provenance only.
//!
//! # Intent
//!
//! Implements Control 3 of the 5-control mistakes-ledger stack from
//! `feedback_repeat_mistake_prevention.md` (Stage 10 of the
//! pipeline-maturity matrix, audit `evidence/audits/pipeline-maturity-audit-2026-05-15.md`).
//!
//! The check (audit blocker #3) detects two anti-patterns:
//!
//! 1. **Unindexed mistake.** A row in `registry/mistakes-ledger.json`
//!    whose `(primitive, failure_mode)` pair has no matching row in
//!    `docs/runbooks/sanctioned-primitives/preflight.md`. This means the
//!    repeat-class lesson never made it into a preflight probe and the
//!    same error can recur in a future session.
//! 2. **Malformed row.** A row missing any of the strict required fields
//!    enumerated in `docs/templates/mistakes-ledger-row-template.md`
//!    (id, primitive, failure_mode, first_seen, second_seen, occurrences,
//!    first_occurrence_evidence, second_occurrence_evidence,
//!    control_landed_at, preflight_hint, icm_keyword,
//!    citation_probe_lane, controls).
//!
//! # Algorithm (kernel — I/O-free)
//!
//! Runners parse `mistakes-ledger.json` into [`LedgerRow`] records, parse
//! `preflight.md` into a [`PreflightIndex`] (the set of indexed
//! `(primitive, failure_mode)` pairs), and pass both into [`check`]. The
//! kernel:
//!
//! 1. For each row, verifies all required fields are populated (non-empty
//!    where strings, ≥ 2 where `occurrences`).
//! 2. For each row, verifies the preflight index contains a matching
//!    `(primitive, failure_mode)` entry.
//! 3. Emits a [`Violation`] for each malformed or unindexed row.
//!
//! Filesystem walking and JSON/markdown parsing live in the dev-CLI
//! runner. The kernel only operates on typed value objects.

#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

/// One row from `registry/mistakes-ledger.json`.
///
/// Fields mirror the strict schema in
/// `docs/templates/mistakes-ledger-row-template.md`. The runner is
/// responsible for parsing JSON; this struct is the typed value object
/// the kernel consumes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerRow {
    pub id: String,                         // data_class: INTERNAL_ONLY
    pub primitive: String,                  // data_class: INTERNAL_ONLY
    pub failure_mode: String,               // data_class: INTERNAL_ONLY
    pub first_seen: String,                 // data_class: INTERNAL_ONLY
    pub second_seen: String,                // data_class: INTERNAL_ONLY
    pub occurrences: u32,                   // data_class: INTERNAL_ONLY
    pub first_occurrence_evidence: String,  // data_class: INTERNAL_ONLY
    pub second_occurrence_evidence: String, // data_class: INTERNAL_ONLY
    pub control_landed_at: String,          // data_class: INTERNAL_ONLY
    pub preflight_hint: String,             // data_class: INTERNAL_ONLY
    pub icm_keyword: String,                // data_class: INTERNAL_ONLY
    pub citation_probe_lane: String,        // data_class: INTERNAL_ONLY
    pub controls: Vec<String>,              // data_class: INTERNAL_ONLY
}

/// The set of `(primitive, failure_mode)` pairs that appear as canonical
/// rows in `docs/runbooks/sanctioned-primitives/preflight.md`. The runner
/// is responsible for parsing the markdown table; the kernel only checks
/// set membership.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PreflightIndex {
    pub indexed_pairs: BTreeSet<(String, String)>, // data_class: INTERNAL_ONLY
}

impl PreflightIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, primitive: impl Into<String>, failure_mode: impl Into<String>) {
        self.indexed_pairs
            .insert((primitive.into(), failure_mode.into()));
    }

    pub fn contains(&self, primitive: &str, failure_mode: &str) -> bool {
        self.indexed_pairs
            .contains(&(primitive.to_string(), failure_mode.to_string()))
    }

    pub fn len(&self) -> usize {
        self.indexed_pairs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.indexed_pairs.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViolationKind {
    /// Row is missing one of the strict required fields, or `occurrences` < 2.
    MalformedRow,
    /// Row is well-formed but its `(primitive, failure_mode)` pair has no
    /// matching row in the preflight runbook.
    UnindexedByPreflight,
    /// Row's `controls` array does not include all five canonical control
    /// IDs (preflight, ledger-row, fitness-lane, icm, citation).
    IncompleteControlSet,
}

impl ViolationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ViolationKind::MalformedRow => "malformed-row",
            ViolationKind::UnindexedByPreflight => "unindexed-by-preflight",
            ViolationKind::IncompleteControlSet => "incomplete-control-set",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Violation {
    pub row_id: String,      // data_class: INTERNAL_ONLY
    pub kind: ViolationKind, // data_class: INTERNAL_ONLY
    pub detail: String,      // data_class: INTERNAL_ONLY
    pub hint: String,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerReport {
    pub rows_checked: usize,           // data_class: INTERNAL_ONLY
    pub preflight_rows_indexed: usize, // data_class: INTERNAL_ONLY
    pub violations: Vec<Violation>,    // data_class: INTERNAL_ONLY
}

impl LedgerReport {
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Tier-1 error per ADR-0083. The kernel itself never returns this — it
/// only emits violations — but the runner needs a typed error surface
/// when the input invariants are violated upstream (e.g. duplicate row
/// ids in a single ledger file).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LedgerInputError {
    DuplicateRowId(String),
    EmptyRowId,
}

impl core::fmt::Display for LedgerInputError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LedgerInputError::DuplicateRowId(id) => {
                write!(
                    formatter,
                    "duplicate ledger row id `{id}` — every MFL-NNNN must be unique"
                )
            }
            LedgerInputError::EmptyRowId => {
                write!(
                    formatter,
                    "empty ledger row id — every row must declare an MFL-NNNN id"
                )
            }
        }
    }
}

impl std::error::Error for LedgerInputError {}

/// The five canonical control IDs that every well-formed ledger row must
/// list in its `controls` array. Order is irrelevant; presence is what
/// the kernel checks. Matches the 5-control stack in
/// `feedback_repeat_mistake_prevention.md`.
pub const REQUIRED_CONTROL_IDS: [&str; 5] =
    ["preflight", "ledger-row", "fitness-lane", "icm", "citation"];

/// Validate the typed ledger rows against the preflight index. The
/// runner-supplied rows MUST have unique ids; that invariant is asserted
/// here via [`LedgerInputError`] before the per-row checks run.
pub fn check(
    rows: &[LedgerRow],
    preflight: &PreflightIndex,
) -> Result<LedgerReport, LedgerInputError> {
    let mut seen_ids: BTreeSet<&str> = BTreeSet::new();
    for row in rows {
        if row.id.is_empty() {
            return Err(LedgerInputError::EmptyRowId);
        }
        if !seen_ids.insert(row.id.as_str()) {
            return Err(LedgerInputError::DuplicateRowId(row.id.clone()));
        }
    }

    let mut violations: Vec<Violation> = Vec::new();

    for row in rows {
        // 1. Strict-schema check (Control 2 template).
        if let Some(missing_field) = first_missing_field(row) {
            violations.push(Violation {
                row_id: row.id.clone(),
                kind: ViolationKind::MalformedRow,
                detail: format!("missing or invalid field `{missing_field}`"),
                hint: format!(
                    "fill `{missing_field}` per `docs/templates/mistakes-ledger-row-template.md` — every ledger row is a strict schema"
                ),
            });
            // A malformed row is still checked for preflight indexing
            // because the primitive+failure_mode pair may still be set.
        }

        // 2. Preflight-indexing check (Control 1 + Control 3 join).
        if !row.primitive.is_empty()
            && !row.failure_mode.is_empty()
            && !preflight.contains(&row.primitive, &row.failure_mode)
        {
            violations.push(Violation {
                row_id: row.id.clone(),
                kind: ViolationKind::UnindexedByPreflight,
                detail: format!(
                    "no row for ({primitive}, {failure_mode}) in preflight runbook",
                    primitive = row.primitive,
                    failure_mode = row.failure_mode,
                ),
                hint: format!(
                    "add a row to `docs/runbooks/sanctioned-primitives/preflight.md` with primitive=`{}` failure_mode=`{}` so the smoke probe runs before the next session — preflight is Control 1",
                    row.primitive, row.failure_mode
                ),
            });
        }

        // 3. Control-set completeness (every row claims all 5 controls landed).
        let row_controls: BTreeSet<&str> = row.controls.iter().map(String::as_str).collect();
        let missing_controls: Vec<&str> = REQUIRED_CONTROL_IDS
            .iter()
            .copied()
            .filter(|control| !row_controls.contains(control))
            .collect();
        if !missing_controls.is_empty() {
            violations.push(Violation {
                row_id: row.id.clone(),
                kind: ViolationKind::IncompleteControlSet,
                detail: format!("missing controls: {}", missing_controls.join(",")),
                hint: format!(
                    "ledger row must claim all 5 controls per `feedback_repeat_mistake_prevention`; add: {}",
                    missing_controls.join(", ")
                ),
            });
        }
    }

    Ok(LedgerReport {
        rows_checked: rows.len(),
        preflight_rows_indexed: preflight.len(),
        violations,
    })
}

/// Returns the name of the first required field that fails the
/// non-empty / ≥-2 invariants. Mirrors the strict schema in
/// `docs/templates/mistakes-ledger-row-template.md`.
fn first_missing_field(row: &LedgerRow) -> Option<&'static str> {
    if row.id.is_empty() {
        return Some("id");
    }
    if row.primitive.is_empty() {
        return Some("primitive");
    }
    if row.failure_mode.is_empty() {
        return Some("failure_mode");
    }
    if row.first_seen.is_empty() {
        return Some("first_seen");
    }
    if row.second_seen.is_empty() {
        return Some("second_seen");
    }
    if row.occurrences < 2 {
        return Some("occurrences"); // by definition ≥ 2 to be a repeat-class
    }
    if row.first_occurrence_evidence.is_empty() {
        return Some("first_occurrence_evidence");
    }
    if row.second_occurrence_evidence.is_empty() {
        return Some("second_occurrence_evidence");
    }
    if row.control_landed_at.is_empty() {
        return Some("control_landed_at");
    }
    if row.preflight_hint.is_empty() {
        return Some("preflight_hint");
    }
    if row.icm_keyword.is_empty() {
        return Some("icm_keyword");
    }
    if row.citation_probe_lane.is_empty() {
        return Some("citation_probe_lane");
    }
    if row.controls.is_empty() {
        return Some("controls");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn well_formed_row(id: &str, primitive: &str, failure_mode: &str) -> LedgerRow {
        LedgerRow {
            id: id.into(),
            primitive: primitive.into(),
            failure_mode: failure_mode.into(),
            first_seen: "2026-05-14".into(),
            second_seen: "2026-05-15".into(),
            occurrences: 2,
            first_occurrence_evidence: "commit a4f3b21".into(),
            second_occurrence_evidence: "commit b9e2c4f".into(),
            control_landed_at: "M01-P17-IP-003".into(),
            preflight_hint: "run the probe".into(),
            icm_keyword: format!("error-class,{primitive},{failure_mode}"),
            citation_probe_lane: "oya-governance-mistakes-ledger-kernel".into(),
            controls: REQUIRED_CONTROL_IDS
                .iter()
                .map(|c| (*c).to_string())
                .collect(),
        }
    }

    fn preflight_with(pairs: &[(&str, &str)]) -> PreflightIndex {
        let mut index = PreflightIndex::new();
        for (primitive, failure_mode) in pairs {
            index.insert(*primitive, *failure_mode);
        }
        index
    }

    #[test]
    fn clean_when_every_row_is_indexed_and_well_formed() {
        let rows = vec![
            well_formed_row("MFL-0014", "gha", "broken-action-sha"),
            well_formed_row("MFL-0015", "nextest", "missing-profile-ci"),
            well_formed_row("MFL-0016", "bash", "missing-shebang"),
        ];
        let preflight = preflight_with(&[
            ("gha", "broken-action-sha"),
            ("nextest", "missing-profile-ci"),
            ("bash", "missing-shebang"),
        ]);
        let report = check(&rows, &preflight).expect("clean rows");
        assert!(report.is_clean(), "expected no violations, got {report:?}");
        assert_eq!(report.rows_checked, 3);
        assert_eq!(report.preflight_rows_indexed, 3);
    }

    #[test]
    fn flags_unindexed_row() {
        let rows = vec![well_formed_row("MFL-0017", "icm", "store-unreachable")];
        let preflight = preflight_with(&[("gha", "broken-action-sha")]);
        let report = check(&rows, &preflight).expect("kernel ok");
        assert!(!report.is_clean());
        let violation = &report.violations[0];
        assert_eq!(violation.kind, ViolationKind::UnindexedByPreflight);
        assert_eq!(violation.row_id, "MFL-0017");
        assert!(violation.hint.contains("preflight.md"));
    }

    #[test]
    fn flags_malformed_row_missing_field() {
        let mut row = well_formed_row("MFL-0018", "cargo", "metadata-failure");
        row.icm_keyword.clear();
        let preflight = preflight_with(&[("cargo", "metadata-failure")]);
        let report = check(&[row], &preflight).expect("kernel ok");
        let kinds: Vec<_> = report.violations.iter().map(|v| v.kind).collect();
        assert!(
            kinds.contains(&ViolationKind::MalformedRow),
            "expected a MalformedRow violation, got {kinds:?}"
        );
        let malformed = report
            .violations
            .iter()
            .find(|v| v.kind == ViolationKind::MalformedRow)
            .unwrap();
        assert!(malformed.detail.contains("icm_keyword"));
    }

    #[test]
    fn flags_row_with_only_one_occurrence() {
        // A row with `occurrences < 2` is not a repeat-class — it does
        // not belong in the ledger and the kernel must flag it.
        let mut row = well_formed_row("MFL-0019", "rustup", "toolchain-drift");
        row.occurrences = 1;
        let preflight = preflight_with(&[("rustup", "toolchain-drift")]);
        let report = check(&[row], &preflight).expect("kernel ok");
        assert!(!report.is_clean());
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.kind == ViolationKind::MalformedRow && v.detail.contains("occurrences"))
        );
    }

    #[test]
    fn flags_incomplete_control_set() {
        let mut row = well_formed_row("MFL-0020", "grit", "registry-bloat");
        row.controls = vec!["preflight".into(), "ledger-row".into()]; // missing 3
        let preflight = preflight_with(&[("grit", "registry-bloat")]);
        let report = check(&[row], &preflight).expect("kernel ok");
        let incomplete = report
            .violations
            .iter()
            .find(|v| v.kind == ViolationKind::IncompleteControlSet)
            .expect("incomplete-control violation");
        assert!(incomplete.detail.contains("fitness-lane"));
        assert!(incomplete.detail.contains("icm"));
        assert!(incomplete.detail.contains("citation"));
    }

    #[test]
    fn rejects_duplicate_row_ids() {
        let rows = vec![
            well_formed_row("MFL-0014", "gha", "broken-action-sha"),
            well_formed_row("MFL-0014", "nextest", "missing-profile-ci"),
        ];
        let preflight = preflight_with(&[
            ("gha", "broken-action-sha"),
            ("nextest", "missing-profile-ci"),
        ]);
        let error = check(&rows, &preflight).expect_err("duplicate ids");
        assert_eq!(error, LedgerInputError::DuplicateRowId("MFL-0014".into()));
    }

    #[test]
    fn rejects_empty_row_id() {
        let mut row = well_formed_row("placeholder", "gha", "broken-action-sha");
        row.id.clear();
        let preflight = preflight_with(&[("gha", "broken-action-sha")]);
        let error = check(&[row], &preflight).expect_err("empty id");
        assert_eq!(error, LedgerInputError::EmptyRowId);
    }

    #[test]
    fn empty_ledger_is_clean() {
        let preflight = preflight_with(&[]);
        let report = check(&[], &preflight).expect("kernel ok");
        assert!(report.is_clean());
        assert_eq!(report.rows_checked, 0);
        assert_eq!(report.preflight_rows_indexed, 0);
    }

    #[test]
    fn preflight_index_helpers() {
        let mut index = PreflightIndex::new();
        assert!(index.is_empty());
        index.insert("gha", "broken-action-sha");
        assert!(!index.is_empty());
        assert_eq!(index.len(), 1);
        assert!(index.contains("gha", "broken-action-sha"));
        assert!(!index.contains("gha", "other"));
    }

    #[test]
    fn violation_kind_str_matches_canonical_names() {
        assert_eq!(ViolationKind::MalformedRow.as_str(), "malformed-row");
        assert_eq!(
            ViolationKind::UnindexedByPreflight.as_str(),
            "unindexed-by-preflight"
        );
        assert_eq!(
            ViolationKind::IncompleteControlSet.as_str(),
            "incomplete-control-set"
        );
    }

    #[test]
    fn ledger_input_error_displays_human_readable() {
        let err = LedgerInputError::DuplicateRowId("MFL-0014".into());
        let rendered = format!("{err}");
        assert!(rendered.contains("MFL-0014"));
        assert!(rendered.contains("unique"));

        let empty = LedgerInputError::EmptyRowId;
        assert!(format!("{empty}").contains("empty"));
    }

    #[test]
    fn report_clean_predicate_tracks_violations_vec() {
        let report = LedgerReport {
            rows_checked: 0,
            preflight_rows_indexed: 0,
            violations: vec![],
        };
        assert!(report.is_clean());

        let report_with_violation = LedgerReport {
            rows_checked: 1,
            preflight_rows_indexed: 0,
            violations: vec![Violation {
                row_id: "MFL-0014".into(),
                kind: ViolationKind::UnindexedByPreflight,
                detail: "x".into(),
                hint: "y".into(),
            }],
        };
        assert!(!report_with_violation.is_clean());
    }
}
