//! # cloud-ci-friction-accounting (ADR-0544)
//!
//! The FRIC-total-accounting meta-gate. Founder decision 2026-06-10: every friction-ledger row
//! must terminate in a gate, an automation, or an explicit accepted-risk entry, enforced by a gate
//! so unconverted frictions block merges like code debt. This is the closed-loop accounting tracker
//! for the friction ledger — the Google SRE postmortem **action-item** model reimplemented
//! Rust-native: every action item (here, every friction) must have a declared disposition and, once
//! terminal or accepted-risk, verifiable closure (evidence).
//!
//! ## Born pack-shaped
//! The crate is a NEUTRAL engine. All repo-specifics — the ledger path, the free-text status
//! taxonomy (`status -> {open|terminal|accepted-risk}`), the required-field set, the evidence policy
//! — are DATA in `friction-accounting-policy.json`. Nothing oyatie-specific is hardcoded in Rust; a
//! different repo adopts the gate by repointing the policy at its own ledger.
//!
//! ## Kernel contract
//! - [`collect_observed_frictions`] `(root, policy) -> {rows:[..]}` reads the ledger file (the only
//!   I/O; read-only, no temp files).
//! - [`evaluate_keyed`] `(policy, observed) -> BTreeSet<Finding>` is PURE and unit-testable without a
//!   filesystem; it folds the event-sourced append rows onto their friction id and applies the
//!   closed-loop invariants.
//! - [`evaluate`] is the bare-code projection of `evaluate_keyed`, the single source of the verdict.
//!
//! ## Ratchet semantics (never discourage logging)
//! Appending a friction row never fails the gate by itself. The blocking codes police schema
//! validity, declared disposition, and closure integrity. Closure-integrity codes
//! (`friction_closed_without_evidence`, `friction_accepted_risk_without_evidence`,
//! `friction_duplicate_primary_row`) are born-blocking frozen-empty: the live ledger satisfies them
//! today, so any NEW occurrence fails closed. Schema/disposition/taxonomy codes baseline today's
//! legacy debt behind a reviewed shrink-only ceiling (the live-repo test owns it) so the gate is not
//! launderable by same-PR baseline regeneration (FRIC-1781112000).
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `friction_policy_gate_id_mismatch`        — policy `gate_id` != [`GATE_ID`].
//! - `friction_missing_required_field`         — a PRIMARY row omits/blanks a required field.
//! - `friction_unknown_status`                 — a friction's effective status maps to no taxonomy class.
//! - `friction_no_disposition`                 — a friction declares no non-blank `enforcement_fix`
//!                                               and is not in the accepted-risk class.
//! - `friction_closed_without_evidence`        — a terminal-class friction cites no evidence.
//! - `friction_accepted_risk_without_evidence` — an accepted-risk friction cites no evidence.
//! - `friction_duplicate_primary_row`          — two PRIMARY rows share one `id` (appends are legitimate).
//! - `friction_orphan_update_row`              — a friction id has ONLY update-shaped rows and no
//!                                               anchoring PRIMARY record. Without a primary the
//!                                               schema/required-field/disposition checks cannot
//!                                               bind, so an update-only row would otherwise evade
//!                                               every check and be silently unaccounted.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Value, json};

/// The gate id, matching the buck2 target + the oya-ci registry id.
pub const GATE_ID: &str = "cloud-ci-friction-accounting";

/// The eight blocking violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 8] = [
    "friction_policy_gate_id_mismatch",
    "friction_missing_required_field",
    "friction_unknown_status",
    "friction_no_disposition",
    "friction_closed_without_evidence",
    "friction_accepted_risk_without_evidence",
    "friction_duplicate_primary_row",
    "friction_orphan_update_row",
];

/// The sentinel key for codes that are policy-level rather than per-friction.
const POLICY_KEY: &str = "<policy>";

/// The taxonomy class a status maps to. `Unknown` is a fail-closed sentinel for NEW statuses that
/// match no taxonomy entry (legacy unknowns are baselined by the live-repo ceiling).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClass {
    Open,
    Terminal,
    AcceptedRisk,
    Unknown,
}

/// Errors collecting the observed ledger. The kernel returns these instead of panicking so the
/// caller (CI / a controller) decides how to surface them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    MissingLedgerPath,
    Io(String),
    Parse { line: usize, message: String },
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::MissingLedgerPath => {
                write!(f, "policy `ledger_path` must be a non-empty string")
            }
            CollectError::Io(message) => write!(f, "friction ledger io: {message}"),
            CollectError::Parse { line, message } => {
                write!(f, "friction ledger line {line} is not valid JSON: {message}")
            }
        }
    }
}

impl std::error::Error for CollectError {}

/// Collect the observed friction ledger described by the policy's `ledger_path`.
///
/// Reads the JSONL ledger relative to `root`, parsing each non-blank line into a row. The output is
/// `{ "rows": [ <row>, .. ] }` mirroring the on-disk physical order (folding happens in the pure
/// evaluator). Read-only: writes no temporary files, so each run cleans up after itself by
/// construction. A blank ledger (no rows) is valid and yields `{ "rows": [] }`.
pub fn collect_observed_frictions(root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let ledger_path = policy
        .get("ledger_path")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(CollectError::MissingLedgerPath)?;
    let absolute = root.join(ledger_path);
    let text = fs::read_to_string(&absolute)
        .map_err(|e| CollectError::Io(format!("read {}: {e}", absolute.display())))?;
    let mut rows = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let row: Value = serde_json::from_str(line).map_err(|e| CollectError::Parse {
            line: index + 1,
            message: e.to_string(),
        })?;
        rows.push(row);
    }
    Ok(json!({ "rows": rows }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub detail: String,
}

impl Finding {
    fn new(code: &str, key: &str, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

impl Report {
    fn from_findings(findings: &BTreeSet<Finding>) -> Self {
        let violations = findings
            .iter()
            .map(|finding| finding.code.clone())
            .collect::<BTreeSet<_>>();
        Self {
            verdict: if violations.is_empty() {
                Verdict::Green
            } else {
                Verdict::Red
            },
            violations,
        }
    }
}

fn non_blank_str<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
}

/// Classify a free-text status against the policy taxonomy. With `status_match == "prefix"` (the
/// default), the LONGEST matching taxonomy key wins, so `RESOLVED-fully` resolves via `RESOLVED`
/// while a more specific key (e.g. `escalated-to-leader`) overrides a broader one. Exact match is
/// also supported (`status_match == "exact"`).
pub fn classify_status(policy: &Value, status: &str) -> StatusClass {
    let taxonomy = policy.get("status_taxonomy").and_then(Value::as_object);
    let Some(taxonomy) = taxonomy else {
        return StatusClass::Unknown;
    };
    let prefix_mode = policy
        .get("status_match")
        .and_then(Value::as_str)
        .map(|mode| mode != "exact")
        .unwrap_or(true);
    let status = status.trim();
    let mut best: Option<(usize, StatusClass)> = None;
    for (key, class) in taxonomy {
        let matches = if prefix_mode {
            status == key || status.starts_with(key)
        } else {
            status == key
        };
        if !matches {
            continue;
        }
        let class = match class.as_str() {
            Some("open") => StatusClass::Open,
            Some("terminal") => StatusClass::Terminal,
            Some("accepted-risk") => StatusClass::AcceptedRisk,
            _ => StatusClass::Unknown,
        };
        let weight = key.len();
        if best.map(|(w, _)| weight > w).unwrap_or(true) {
            best = Some((weight, class));
        }
    }
    best.map(|(_, class)| class).unwrap_or(StatusClass::Unknown)
}

/// The folded, effective state of one friction id across its physical rows.
#[derive(Debug, Default, Clone)]
struct FrictionState {
    /// Count of PRIMARY rows (rows carrying both `status` and `friction`): >1 is a duplicate defect.
    primary_count: usize,
    /// The effective status: the latest `status_update` if any update rows exist, else the primary
    /// `status`. Empty if neither is present.
    effective_status: String,
    /// True if any row for this id carries a non-blank `enforcement_fix`.
    has_disposition: bool,
    /// True if any row for this id carries a non-blank `evidence`.
    has_evidence: bool,
}

fn is_primary(row: &Value) -> bool {
    row.get("status").and_then(Value::as_str).is_some()
        && row.get("friction").and_then(Value::as_str).is_some()
}

/// Pure evaluator. `policy` is DATA (`friction-accounting-policy.json`); `observed` is the collected
/// ledger shaped as `{ "rows": [ <row>, .. ] }` in physical order.
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if non_blank_str(policy, "gate_id") != Some(GATE_ID) {
        findings.insert(Finding::new(
            "friction_policy_gate_id_mismatch",
            POLICY_KEY,
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    let required_fields: Vec<String> = policy
        .get("required_primary_fields")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let terminal_requires_evidence = policy
        .get("terminal_requires_evidence")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let accepted_risk_requires_evidence = policy
        .get("accepted_risk_requires_evidence")
        .and_then(Value::as_bool)
        .unwrap_or(true);

    let rows = observed
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    // Fold physical rows onto their friction id (the event-sourced append model). `states` is a
    // BTreeMap (key-sorted, deterministic) so per-friction findings are order-independent; the
    // BTreeSet output is sorted anyway.
    let mut states: BTreeMap<String, FrictionState> = BTreeMap::new();
    for row in &rows {
        let id = non_blank_str(row, "id").unwrap_or("");
        if id.is_empty() {
            // A row with no usable id cannot be accounted: schema violation keyed to a sentinel.
            findings.insert(Finding::new(
                "friction_missing_required_field",
                "<row-without-id>",
                "ledger row carries no non-blank `id`",
            ));
            continue;
        }
        let state = states.entry(id.to_owned()).or_default();

        if is_primary(row) {
            state.primary_count += 1;
            // Required-field check applies to PRIMARY rows (they carry the full record).
            for field in &required_fields {
                if non_blank_str(row, field).is_none() {
                    findings.insert(Finding::new(
                        "friction_missing_required_field",
                        id,
                        format!("primary row missing non-blank `{field}`"),
                    ));
                }
            }
            if let Some(status) = non_blank_str(row, "status") {
                // The primary status is the baseline effective status; an update overrides it below.
                if state.effective_status.is_empty() {
                    state.effective_status = status.to_owned();
                }
            }
        }
        // Update rows carry the latest disposition transition; they always win the effective status.
        if let Some(update) = non_blank_str(row, "status_update") {
            state.effective_status = update.to_owned();
        }
        if non_blank_str(row, "enforcement_fix").is_some() {
            state.has_disposition = true;
        }
        if non_blank_str(row, "evidence").is_some() {
            state.has_evidence = true;
        }
    }

    for (id, state) in &states {
        if state.primary_count == 0 {
            // ONLY update-shaped rows exist for this id: there is no anchoring PRIMARY record, so the
            // required-field/disposition/duplicate/class checks cannot bind. An update-only row is the
            // cheapest way to evade the born-blocking schema check (status_update=RESOLVED + evidence
            // would otherwise fold to a clean terminal state and pass), so the missing primary is
            // itself the violation — and the ONLY one we emit for this id, so a fixed orphan drops a
            // single baseline key rather than churning several. This is baseline-block-on-new: the
            // live ledger's pre-existing orphan ids are frozen as shrinkable legacy debt, and any NEW
            // orphan-update id fails closed.
            findings.insert(Finding::new(
                "friction_orphan_update_row",
                id,
                "friction has only update-shaped rows and no anchoring primary record; \
                 log a primary row (id/seen_at/friction/enforcement_fix/status) for this id",
            ));
            continue;
        }
        if state.primary_count > 1 {
            findings.insert(Finding::new(
                "friction_duplicate_primary_row",
                id,
                format!("{} primary rows share id `{id}`; updates append, primaries do not", state.primary_count),
            ));
        }

        let class = classify_status(policy, &state.effective_status);
        match class {
            StatusClass::Unknown => {
                findings.insert(Finding::new(
                    "friction_unknown_status",
                    id,
                    format!(
                        "effective status `{}` maps to no taxonomy class",
                        state.effective_status
                    ),
                ));
            }
            StatusClass::Open => {
                if !state.has_disposition {
                    findings.insert(Finding::new(
                        "friction_no_disposition",
                        id,
                        "open friction declares no non-blank `enforcement_fix` (disposition)",
                    ));
                }
            }
            StatusClass::Terminal => {
                if !state.has_disposition {
                    findings.insert(Finding::new(
                        "friction_no_disposition",
                        id,
                        "terminal friction declares no non-blank `enforcement_fix` (disposition)",
                    ));
                }
                if terminal_requires_evidence && !state.has_evidence {
                    findings.insert(Finding::new(
                        "friction_closed_without_evidence",
                        id,
                        "terminal-status friction cites no `evidence` (gate id / ADR / tool path)",
                    ));
                }
            }
            StatusClass::AcceptedRisk => {
                // Accepted-risk frictions (founder-held / escalated) are not agent-closeable and are
                // not open debt; their disposition IS the recorded acceptance, proven by evidence.
                if accepted_risk_requires_evidence && !state.has_evidence {
                    findings.insert(Finding::new(
                        "friction_accepted_risk_without_evidence",
                        id,
                        "accepted-risk friction cites no `evidence` for the holder/decision",
                    ));
                }
            }
        }
    }

    findings
}

/// Bare-code projection of [`evaluate_keyed`]; the single source of truth for the verdict.
// Decides whether the observed action items satisfy the policy.
pub fn evaluate(policy: &Value, observed: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(policy, observed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn policy() -> Value {
        json!({
            "gate_id": GATE_ID,
            "required_primary_fields": ["id", "seen_at", "friction", "enforcement_fix", "status"],
            "status_match": "prefix",
            "terminal_requires_evidence": true,
            "accepted_risk_requires_evidence": true,
            "status_taxonomy": {
                "open": "open",
                "queued-G11": "open",
                "escalated-to-leader": "accepted-risk",
                "interim-accepted": "accepted-risk",
                "RESOLVED": "terminal",
                "ADR-": "terminal"
            }
        })
    }

    fn primary(id: &str, status: &str) -> Value {
        json!({
            "id": id,
            "seen_at": "2026-06-10",
            "friction": "something went wrong",
            "pipeline_defect": "a defect",
            "enforcement_fix": "wire a gate",
            "status": status
        })
    }

    fn observed(rows: Vec<Value>) -> Value {
        json!({ "rows": rows })
    }

    #[test]
    fn fully_accounted_open_friction_is_green() {
        let report = evaluate(&policy(), &observed(vec![primary("FRIC-1", "open")]));
        assert_eq!(report.verdict, Verdict::Green);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn empty_ledger_is_green() {
        assert_eq!(evaluate(&policy(), &observed(vec![])).verdict, Verdict::Green);
    }

    #[test]
    fn appending_a_well_formed_row_never_fails_the_gate() {
        // The ratchet must never punish logging: a brand-new valid open friction is green.
        let findings = evaluate_keyed(&policy(), &observed(vec![primary("FRIC-NEW", "queued-G11")]));
        assert!(findings.is_empty(), "logging a valid friction must not block: {findings:#?}");
    }

    #[test]
    fn unknown_status_on_new_row_fails_closed() {
        let findings = evaluate_keyed(
            &policy(),
            &observed(vec![primary("FRIC-X", "totally-made-up-status")]),
        );
        assert!(findings.iter().any(|f| {
            f.code == "friction_unknown_status" && f.key == "FRIC-X"
        }));
    }

    #[test]
    fn blank_enforcement_fix_on_open_row_is_no_disposition() {
        let mut row = primary("FRIC-ND", "open");
        row["enforcement_fix"] = json!("   ");
        let findings = evaluate_keyed(&policy(), &observed(vec![row]));
        assert!(findings.iter().any(|f| {
            f.code == "friction_no_disposition" && f.key == "FRIC-ND"
        }));
        // The blank required field is ALSO a schema violation.
        assert!(findings.iter().any(|f| {
            f.code == "friction_missing_required_field" && f.key == "FRIC-ND"
        }));
    }

    #[test]
    fn terminal_status_without_evidence_fails_closed() {
        let findings = evaluate_keyed(&policy(), &observed(vec![primary("FRIC-T", "RESOLVED")]));
        assert!(findings.iter().any(|f| {
            f.code == "friction_closed_without_evidence" && f.key == "FRIC-T"
        }));
    }

    #[test]
    fn terminal_status_with_evidence_is_green() {
        let mut row = primary("FRIC-T2", "RESOLVED-fully");
        row["evidence"] = json!("PR #669 merged @ 16f2e3b54: enforcement-liveness gate");
        let findings = evaluate_keyed(&policy(), &observed(vec![row]));
        assert!(findings.is_empty(), "terminal+evidence must be green: {findings:#?}");
    }

    #[test]
    fn accepted_risk_without_evidence_fails_closed() {
        let mut row = primary("FRIC-AR", "escalated-to-leader-for-force-complete");
        // accepted-risk does not require enforcement_fix, but DOES require evidence.
        row["enforcement_fix"] = json!("escalated");
        let findings = evaluate_keyed(&policy(), &observed(vec![row]));
        assert!(findings.iter().any(|f| {
            f.code == "friction_accepted_risk_without_evidence" && f.key == "FRIC-AR"
        }));
    }

    #[test]
    fn accepted_risk_with_evidence_is_green() {
        let mut row = primary("FRIC-AR2", "interim-accepted");
        row["evidence"] = json!("founder-held 2026-06-10; leader-side transition pending");
        let findings = evaluate_keyed(&policy(), &observed(vec![row]));
        assert!(findings.is_empty(), "accepted-risk+evidence must be green: {findings:#?}");
    }

    #[test]
    fn duplicate_primary_row_fails_closed_but_appends_do_not() {
        // Two PRIMARY rows sharing an id is a defect.
        let dup = evaluate_keyed(
            &policy(),
            &observed(vec![primary("FRIC-D", "open"), primary("FRIC-D", "open")]),
        );
        assert!(dup.iter().any(|f| {
            f.code == "friction_duplicate_primary_row" && f.key == "FRIC-D"
        }));

        // A primary + an append (update row) sharing an id is LEGITIMATE event-sourcing.
        let append = json!({
            "id": "FRIC-D2",
            "seen_at": "2026-06-10",
            "status_update": "RESOLVED",
            "evidence": "PR #700 merged @ deadbeef: gate landed"
        });
        let folded = evaluate_keyed(
            &policy(),
            &observed(vec![primary("FRIC-D2", "open"), append]),
        );
        assert!(
            !folded.iter().any(|f| f.code == "friction_duplicate_primary_row"),
            "an append must not count as a duplicate primary: {folded:#?}"
        );
    }

    #[test]
    fn orphan_update_only_friction_fails_closed_as_sole_finding() {
        // A friction with ONLY update rows (no primary) must fail closed: it would otherwise fold to
        // a clean terminal-with-evidence state and evade every schema/disposition/closure check.
        let orphan = json!({
            "id": "FRIC-ORPHAN",
            "seen_at": "2026-06-10",
            "status_update": "RESOLVED",
            "enforcement_fix": "looks disposed",
            "evidence": "looks closed"
        });
        let findings = evaluate_keyed(&policy(), &observed(vec![orphan]));
        let mine: Vec<_> = findings.iter().filter(|f| f.key == "FRIC-ORPHAN").collect();
        assert_eq!(mine.len(), 1, "orphan emits exactly one finding: {findings:#?}");
        assert_eq!(mine[0].code, "friction_orphan_update_row");
    }

    #[test]
    fn primary_plus_updates_is_not_an_orphan() {
        // A real friction (primary + later update rows) must NOT be flagged as an orphan.
        let append = json!({
            "id": "FRIC-REAL",
            "seen_at": "2026-06-10",
            "status_update": "RESOLVED",
            "evidence": "PR #700 merged @ deadbeef"
        });
        let findings = evaluate_keyed(
            &policy(),
            &observed(vec![primary("FRIC-REAL", "open"), append]),
        );
        assert!(
            !findings.iter().any(|f| f.code == "friction_orphan_update_row"),
            "primary+updates must not be an orphan: {findings:#?}"
        );
    }

    #[test]
    fn update_row_overrides_primary_status_for_effective_state() {
        // Primary is open (no evidence needed); update closes it (now needs evidence).
        let append_no_evidence = json!({
            "id": "FRIC-U",
            "seen_at": "2026-06-10",
            "status_update": "RESOLVED-structurally"
        });
        let findings = evaluate_keyed(
            &policy(),
            &observed(vec![primary("FRIC-U", "open"), append_no_evidence]),
        );
        assert!(
            findings.iter().any(|f| f.code == "friction_closed_without_evidence" && f.key == "FRIC-U"),
            "update closing a friction must require evidence: {findings:#?}"
        );

        // With evidence on the closing update, it is green.
        let append_with_evidence = json!({
            "id": "FRIC-U2",
            "seen_at": "2026-06-10",
            "status_update": "RESOLVED-structurally",
            "evidence": "PR #661 merged @ 28154faa7"
        });
        let green = evaluate_keyed(
            &policy(),
            &observed(vec![primary("FRIC-U2", "open"), append_with_evidence]),
        );
        assert!(green.is_empty(), "closed-with-evidence via update must be green: {green:#?}");
    }

    #[test]
    fn gate_id_mismatch_in_policy_fails_closed() {
        let mut bad = policy();
        bad["gate_id"] = json!("cloud-ci-wrong");
        let findings = evaluate_keyed(&bad, &observed(vec![]));
        assert!(findings.iter().any(|f| f.code == "friction_policy_gate_id_mismatch"));
    }

    #[test]
    fn prefix_match_prefers_longest_taxonomy_key() {
        // `escalated-to-leader` (accepted-risk) must win over a hypothetical broader `escalated` key.
        let mut p = policy();
        p["status_taxonomy"]["escalated"] = json!("open");
        assert_eq!(
            classify_status(&p, "escalated-to-leader-for-force-complete"),
            StatusClass::AcceptedRisk
        );
    }

    #[test]
    fn evaluate_is_bare_projection_of_evaluate_keyed() {
        let rows = vec![
            primary("FRIC-A", "totally-unknown"),
            primary("FRIC-B", "RESOLVED"),
        ];
        let input = observed(rows);
        let projected: BTreeSet<String> = evaluate_keyed(&policy(), &input)
            .into_iter()
            .map(|finding| finding.code)
            .collect();
        assert_eq!(evaluate(&policy(), &input).violations, projected);
    }
}
