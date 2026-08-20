//! # cloud-ci-staleness-reaper (GATE-3)
//!
//! The staleness-reaper gate (PHASE-0-FIREWALL-PLAN §5.2; pillar-G sinker++). It consumes
//! the `ttl-policy.generated.json` budgets + the accounting registry and REPORTS — never
//! reaps — artifacts that are over their TTL budget AND unreachable as ARCHIVE candidates.
//! Auto-archive is `report -> git mv -> _archive/`, second-verifier-gated, NEVER `rm`
//! (founder rule: never delete on an unverified verdict). It evaluates a fixture/registry
//! `Value` and emits `{verdict, violations}`; its tests assert
//! `report.violations == fixture.expected_violations` over
//! `specs/fixtures/staleness-reaper/tc-*.json`.
//!
//! ## Blocking violation codes (the contract — literal strings the gate emits)
//! - `stale_over_budget_unreachable` — a row whose `age_days` exceeds its TTL `budget_days`
//!   AND whose `reachable_from` is empty (unreachable). The archive-candidate signal:
//!   age alone is NOT stale; reachability alone is NOT stale; BOTH must hold. Protected
//!   classes (`ttl.protected:true`) are never flagged.
//! - `untyped_staleness`             — a row that cannot be aged because it carries no
//!   TTL type (`ttl.ttl_class` empty/missing): staleness is undecidable, which is itself
//!   a defect (every resource must be typed so it CAN be aged).
//! - `reap_without_report`           — a row carrying a reap action (`reaped:true`) that
//!   was not first REPORTED + archived (`reported_then_archived` not true): a reap that
//!   skipped the report -> git mv -> _archive/ discipline (the `rm` defect).
//!
//! The evaluator is pure: fixtures (data-under-test) drive it; there are no scanner
//! special-cases. ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde_json::Value;

/// The gate id, matching the buck2 target + the §5.2 contract.
pub const GATE_ID: &str = "cloud-ci-staleness-reaper";

/// The three blocking codes, in canonical order. The fixtures pin exact subsets.
pub const VIOLATION_CODES: [&str; 3] = [
    "stale_over_budget_unreachable",
    "untyped_staleness",
    "reap_without_report",
];

const STALENESS_ROWS_KEY: &str = "<cloud-ci-staleness-reaper#rows>";
const STALENESS_ROW_KEY: &str = "<cloud-ci-staleness-reaper#row>";

/// The gate report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

/// A keyed violation: the bare `code` (the existing contract) PLUS the stable `key`
/// (the registry row `path`). The going-live ratchet baselines per `(code, key)`;
/// `evaluate()` is the bare-code projection of `evaluate_keyed()`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
}

impl Finding {
    fn new(code: &str, key: &str) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
        }
    }
}

impl Report {
    fn from_violations(violations: BTreeSet<String>) -> Self {
        let verdict = if violations.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        };
        Self {
            verdict,
            violations,
        }
    }
}

/// Evaluate a staleness-reaper fixture/registry `Value` into a report.
///
/// The fixture shape mirrors the registry rows + a per-row staleness signal:
/// ```jsonc
/// {
///   "rows": [
///     {
///       "path": "docs/scratch/_partial-foo.md",
///       "age_days": 120,
///       "reachable_from": [],
///       "ttl": {"ttl_class": "husk", "budget_days": 14, "protected": false, "action": "archive"},
///       "reaped": false,                 // a reap action was applied to this row
///       "reported_then_archived": false  // it was first reported + git-mv archived
///     }
///   ]
/// }
/// ```
/// Bare-code projection of [`evaluate_keyed`]: identical detection logic, keys dropped.
/// Every `tc-*.json` fixture keeps asserting bare codes against it byte-for-byte.
pub fn evaluate(fixture: &Value) -> Report {
    let violations = evaluate_keyed(fixture)
        .into_iter()
        .map(|finding| finding.code)
        .collect();
    Report::from_violations(violations)
}

/// Evaluate a staleness-reaper registry into the keyed finding set — the single source
/// of truth for the gate's detection logic. Each finding is keyed by the row `path`.
pub fn evaluate_keyed(fixture: &Value) -> BTreeSet<Finding> {
    let mut findings: BTreeSet<Finding> = BTreeSet::new();

    let rows = match fixture.get("rows").and_then(Value::as_array) {
        Some(rows) if !rows.is_empty() => rows,
        _ => {
            findings.insert(Finding::new("untyped_staleness", STALENESS_ROWS_KEY));
            return findings;
        }
    };

    for row in rows {
        evaluate_row(row, &mut findings);
    }

    findings
}

fn evaluate_row(row: &Value, findings: &mut BTreeSet<Finding>) {
    let Some(key) = row
        .get("path")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        findings.insert(Finding::new("untyped_staleness", STALENESS_ROW_KEY));
        return;
    };

    // reap_without_report: a reap that skipped report -> git mv -> _archive/.
    let reaped = row.get("reaped").and_then(Value::as_bool) == Some(true);
    let reported_then_archived =
        row.get("reported_then_archived").and_then(Value::as_bool) == Some(true);
    if reaped && !reported_then_archived {
        findings.insert(Finding::new("reap_without_report", key));
    }

    let ttl = row.get("ttl");
    let ttl_class_present = ttl
        .and_then(|t| t.get("ttl_class"))
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty());

    // untyped_staleness: an untyped row cannot be aged at all.
    if !ttl_class_present {
        findings.insert(Finding::new("untyped_staleness", key));
        return;
    }

    // Protected classes are never reaped, regardless of age (age alone != stale).
    let protected = ttl
        .and_then(|t| t.get("protected"))
        .and_then(Value::as_bool)
        == Some(true);
    if protected {
        return;
    }

    // stale_over_budget_unreachable: over budget AND unreachable.
    let budget_days = ttl
        .and_then(|t| t.get("budget_days"))
        .and_then(Value::as_u64);
    let age_days = row.get("age_days").and_then(Value::as_u64);
    let unreachable = row
        .get("reachable_from")
        .and_then(Value::as_array)
        .map(Vec::is_empty)
        .unwrap_or(true);

    if let (Some(budget), Some(age)) = (budget_days, age_days)
        && age > budget
        && unreachable
    {
        findings.insert(Finding::new("stale_over_budget_unreachable", key));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn absent_malformed_or_empty_rows_fail_closed() {
        for fixture in [
            json!({}),
            json!({"rows": "not-an-array"}),
            json!({"rows": []}),
        ] {
            let findings = evaluate_keyed(&fixture);
            assert!(findings.contains(&Finding::new("untyped_staleness", STALENESS_ROWS_KEY)));
            assert_eq!(evaluate(&fixture).verdict, Verdict::Red);
        }
    }

    #[test]
    fn malformed_row_without_path_fails_closed_with_stable_key() {
        let fixture = json!({"rows":[{
            "age_days": 120,
            "reachable_from": [],
            "ttl": {"ttl_class": "husk", "budget_days": 14, "protected": false, "action": "archive"}
        }]});

        let findings = evaluate_keyed(&fixture);
        assert!(findings.contains(&Finding::new("untyped_staleness", STALENESS_ROW_KEY)));
        assert_eq!(evaluate(&fixture).verdict, Verdict::Red);
    }

    #[test]
    fn old_but_reachable_is_green() {
        // Age alone is NOT stale: an old ADR that is still reachable passes.
        let fixture = json!({
            "rows": [{
                "path": "docs/adr-archive/ADR-0001-cohesion-thesis-one-product-flat-catalog.md",
                "age_days": 400,
                "reachable_from": ["masterplan"],
                "ttl": {"ttl_class": "doc", "budget_days": null, "protected": false, "action": "report"}
            }]
        });
        assert_eq!(evaluate(&fixture).verdict, Verdict::Green);
    }

    #[test]
    fn protected_class_is_never_reaped() {
        let fixture = json!({
            "rows": [{
                "path": "specs/masterplan.json",
                "age_days": 9999,
                "reachable_from": [],
                "ttl": {"ttl_class": "spec", "budget_days": 30, "protected": true, "action": "report"}
            }]
        });
        assert_eq!(evaluate(&fixture).verdict, Verdict::Green);
    }

    #[test]
    fn over_budget_and_unreachable_fires() {
        let fixture = json!({
            "rows": [{
                "path": "docs/scratch/_partial-foo.md",
                "age_days": 120,
                "reachable_from": [],
                "ttl": {"ttl_class": "husk", "budget_days": 14, "protected": false, "action": "archive"}
            }]
        });
        assert!(
            evaluate(&fixture)
                .violations
                .contains("stale_over_budget_unreachable")
        );
    }

    #[test]
    fn evaluate_keyed_carries_the_row_path_as_key() {
        let fixture = json!({"rows":[{
            "path":"docs/scratch/_partial-foo.md","age_days":120,"reachable_from":[],
            "ttl":{"ttl_class":"husk","budget_days":14,"protected":false,"action":"archive"}
        }]});
        let findings = evaluate_keyed(&fixture);
        assert!(findings.contains(&Finding::new(
            "stale_over_budget_unreachable",
            "docs/scratch/_partial-foo.md"
        )));
        let projected: BTreeSet<String> = findings.iter().map(|f| f.code.clone()).collect();
        assert_eq!(evaluate(&fixture).violations, projected);
    }

    #[test]
    fn each_code_fires_in_isolation() {
        // untyped_staleness
        assert!(
            evaluate(&json!({"rows":[{"path":"a","age_days":100,"reachable_from":[],"ttl":{}}]}))
                .violations
                .contains("untyped_staleness")
        );
        // reap_without_report
        assert!(evaluate(&json!({"rows":[{"path":"a","reaped":true,"reported_then_archived":false,"ttl":{"ttl_class":"husk","budget_days":14,"protected":false}}]}))
            .violations.contains("reap_without_report"));
    }

    #[test]
    fn stale_reported_then_archived_is_green() {
        // report -> git mv -> _archive/ (no rm): the disciplined path passes.
        let fixture = json!({
            "rows": [{
                "path": "_archive/docs/scratch/_partial-foo.md",
                "age_days": 120,
                "reachable_from": ["archive-index"],
                "ttl": {"ttl_class": "husk", "budget_days": 14, "protected": false, "action": "archive"},
                "reaped": true,
                "reported_then_archived": true
            }]
        });
        assert_eq!(evaluate(&fixture).verdict, Verdict::Green);
    }
}
