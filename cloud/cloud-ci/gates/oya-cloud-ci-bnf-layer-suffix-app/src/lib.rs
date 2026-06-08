//! # cloud-ci-bnf-layer-suffix (§2.5#4 — MIG-PREREQ floor gate S1)
//!
//! Enforces the ADR-0056 BNF rule that every first-party `oya-*` crate's trailing
//! dash-segment is one of the 13 canonical layer values
//! (`kernel|domain|usecase|app|adapter|infrastructure|cli|rest|grpc|graphql|worker|sdk|api`).
//!
//! ## Reuse, not re-derive (CLI-GOVERNANCE-TO-FIREWALL-MIGRATION-PLAN Principle 1, shape a)
//! The policy lives in the PURE, I/O-free `oya_governance_predictable_naming_kernel::check()`.
//! The producer (`oya-cloud-ci-accounting-registry-app`) does the I/O — it enumerates the
//! first-party `oya-*` crate package names from the tracked Cargo.toml manifests and feeds them
//! as `rows`. This gate resolves each crate's `declared_role` CARVE-OUT-AWARE and runs `check()`:
//! - `oya-check-*` (self-layering check-family) and the doctrinal carve-out
//!   (`oya-tooling-agent-read`) → `declared_role = None` (exempt: `check()` skips them in the
//!   undeclared-role branch);
//! - `oya-*-adapter-<backend>` (backend-qualified adapter) → `declared_role = Some("adapter")`
//!   (effective layer is `adapter`, so `check()` sees a match — no violation);
//! - every other crate → `declared_role = Some(<trailing dash-segment>)`.
//!   `declared_context = Some(..)` for all (this gate scopes to the layer-SUFFIX axis, §2.5#4;
//!   the undeclared-context axis is a different concern and is intentionally not flagged here).
//!
//! Net effect: `check()` emits `UnknownRole` for EXACTLY the crates whose trailing segment is
//! not a canonical layer — the §2.5#4 violation set — and nothing else on a lowercase,
//! `oya-`-prefixed corpus.
//!
//! ## Contract
//! `evaluate_keyed` returns one `Finding{code,key}` per violation (`key` = crate name);
//! `evaluate` is the bare-code projection. The producer's `build_gate_baseline` freezes
//! today's keys (`mode: baseline-block-on-new`) so only NEW non-canonical suffixes block.
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use oya_governance_predictable_naming_kernel::{
    check, is_backend_qualified_adapter, is_check_family, is_doctrinal_carve_out, CrateNaming,
    NamingViolationKind,
};
use serde_json::Value;

/// The gate id, matching the buck2 target + the firewall baseline gate-id.
pub const GATE_ID: &str = "cloud-ci-bnf-layer-suffix";

/// The blocking violation codes (stable slugs the gate emits). The §2.5#4 corpus today only
/// trips `bnf_unknown_role` (a non-canonical trailing segment); the others are defended in
/// case a future crate trips them (e.g. a non-`oya-` name, or a declared≠inferred mismatch).
pub const VIOLATION_CODES: [&str; 7] = [
    "bnf_unknown_role",
    "bnf_role_mismatch",
    "bnf_missing_oya_prefix",
    "bnf_empty_after_prefix",
    "bnf_undeclared_role",
    "bnf_undeclared_context",
    "bnf_name_uppercase",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

/// A keyed violation: the stable `code` plus the offending crate name `key`.
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

impl Report {
    fn from_codes(violations: BTreeSet<String>) -> Self {
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

/// Map a kernel `NamingViolationKind` to this gate's stable code slug. (The kernel's
/// `as_str()` returns human prose, not a slug, so the mapping lives here.)
fn code_slug(kind: &NamingViolationKind) -> &'static str {
    match kind {
        NamingViolationKind::MissingOyaPrefix => "bnf_missing_oya_prefix",
        NamingViolationKind::EmptyAfterPrefix => "bnf_empty_after_prefix",
        NamingViolationKind::RoleMismatch { .. } => "bnf_role_mismatch",
        NamingViolationKind::UnknownRole { .. } => "bnf_unknown_role",
        NamingViolationKind::UndeclaredRole => "bnf_undeclared_role",
        NamingViolationKind::UndeclaredContext => "bnf_undeclared_context",
        NamingViolationKind::NameContainsUppercase => "bnf_name_uppercase",
    }
}

/// Resolve a crate name into the typed [`CrateNaming`] the kernel consumes, applying the
/// ADR-0105 carve-outs so `check()` only flags genuine layer-suffix violations. The
/// trailing dash-segment is the inferred role for the general case; `declared_context` is
/// always `Some` because this gate scopes to the layer-suffix axis only.
fn resolve_naming(crate_name: &str) -> CrateNaming {
    let declared_role = if is_check_family(crate_name) || is_doctrinal_carve_out(crate_name) {
        // Self-layering (check-family) / doctrinal primitive: no declared role required.
        None
    } else if is_backend_qualified_adapter(crate_name) {
        // `oya-<svc>-adapter-<backend>`: the effective layer is `adapter`.
        Some("adapter".to_owned())
    } else {
        // General case: the trailing dash-segment is the declared (== inferred) role.
        crate_name
            .rsplit_once('-')
            .map(|(_, role)| role.to_owned())
    };
    CrateNaming {
        crate_name: crate_name.to_owned(),
        declared_role,
        declared_context: Some("inferred".to_owned()),
    }
}

/// Pure evaluator: takes `{"rows": [{"crate_name": "oya-..."}, ...]}` and returns one
/// `Finding` per layer-suffix violation. Reuses `oya_governance_predictable_naming_kernel::check`.
pub fn evaluate_keyed(input: &Value) -> BTreeSet<Finding> {
    let rows = input
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let namings: Vec<CrateNaming> = rows
        .iter()
        .filter_map(|row| row.get("crate_name").and_then(Value::as_str))
        .map(resolve_naming)
        .collect();
    let report = match check(&namings) {
        Ok(report) => report,
        // An empty crate name is the only error; treat it as a single blocking finding so the
        // gate never silently passes on malformed input.
        Err(_) => {
            let mut out = BTreeSet::new();
            out.insert(Finding::new("bnf_empty_after_prefix", "<empty-crate-name>"));
            return out;
        }
    };
    report
        .violations
        .into_iter()
        .map(|v| Finding::new(code_slug(&v.kind), &v.crate_name))
        .collect()
}

/// Bare-code projection of [`evaluate_keyed`] — the single source of truth for the verdict.
pub fn evaluate(input: &Value) -> Report {
    let codes: BTreeSet<String> = evaluate_keyed(input)
        .into_iter()
        .map(|f| f.code)
        .collect();
    Report::from_codes(codes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn rows(names: &[&str]) -> Value {
        json!({ "rows": names.iter().map(|n| json!({ "crate_name": n })).collect::<Vec<_>>() })
    }

    #[test]
    fn canonical_suffixes_are_green() {
        let input = rows(&[
            "oya-medical-domain",
            "oya-tenancy-kernel",
            "oya-workflow-approvals-usecase",
            "oya-cloud-cli",
            "oya-foo-app",
            "oya-bar-infrastructure",
            "oya-baz-adapter",
        ]);
        let report = evaluate(&input);
        assert_eq!(report.verdict, Verdict::Green, "got {:?}", report.violations);
        assert!(evaluate_keyed(&input).is_empty());
    }

    #[test]
    fn non_canonical_suffix_fires_unknown_role() {
        let input = rows(&["oya-foo-runtime", "oya-bar-core", "oya-baz-service"]);
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 3, "got {findings:?}");
        assert!(findings.iter().all(|f| f.code == "bnf_unknown_role"));
        assert!(findings.iter().any(|f| f.key == "oya-foo-runtime"));
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn check_family_is_exempt() {
        // `oya-check-data-class` ends in "class" (not a layer) but is self-layering → GREEN.
        let input = rows(&["oya-check-data-class", "oya-check-layered-architecture-discipline"]);
        assert!(
            evaluate_keyed(&input).is_empty(),
            "check-family must be exempt: {:?}",
            evaluate_keyed(&input)
        );
    }

    #[test]
    fn backend_qualified_adapter_is_exempt() {
        // `oya-records-adapter-postgres` → effective layer `adapter` → GREEN.
        let input = rows(&["oya-records-adapter-postgres", "oya-secrets-adapter-aws"]);
        assert!(
            evaluate_keyed(&input).is_empty(),
            "backend-qualified adapter must be exempt: {:?}",
            evaluate_keyed(&input)
        );
    }

    #[test]
    fn doctrinal_carve_out_is_exempt() {
        let input = rows(&["oya-tooling-agent-read"]);
        assert!(evaluate_keyed(&input).is_empty());
    }

    #[test]
    fn non_oya_prefix_fires_missing_prefix() {
        let input = rows(&["registry-drift"]);
        let findings = evaluate_keyed(&input);
        assert!(findings.iter().any(|f| f.code == "bnf_missing_oya_prefix"));
    }

    #[test]
    fn evaluate_is_bare_projection_of_evaluate_keyed() {
        let input = rows(&["oya-foo-runtime", "oya-good-domain"]);
        let projected: BTreeSet<String> =
            evaluate_keyed(&input).into_iter().map(|f| f.code).collect();
        assert_eq!(evaluate(&input).violations, projected);
    }

    #[test]
    fn empty_corpus_is_green() {
        assert_eq!(evaluate(&json!({ "rows": [] })).verdict, Verdict::Green);
    }
}
