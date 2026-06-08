//! # cloud-ci-cargo-prefix (ADR-0017 — §2.5-adjacent, MIG-PREREQ floor gate S3)
//!
//! Enforces the ADR-0017 cargo-prefix fitness rule that every first-party workspace member's
//! crate-id (its member-path leaf) AND its declared `[package].name` begin with the required
//! prefix, and that the two agree.
//!
//! ## Reuse, not re-derive (CLI-GOVERNANCE-TO-FIREWALL-MIGRATION-PLAN Principle 1, shape b)
//! The policy lives in the PURE, I/O-free `oya_intelligence_cargo_prefix_domain::validate_cargo_prefix`
//! — the SAME predicate the `oya gate validate cargo-prefix` dev-cli call uses (the firewall gate
//! and the dev-cli call coexist for now). The producer
//! (`oya-cloud-ci-accounting-registry-app`) does the I/O — it enumerates the first-party `oya-*`
//! workspace members from the tracked Cargo.toml manifests and feeds each as a row of
//! `{"member_path", "package_name"}`. This gate runs `validate_cargo_prefix` over each row
//! INDEPENDENTLY (a single-member iterator) so the verdict is per-crate and surface-all (the
//! upstream `validate_cargo_prefix` is fail-fast over a whole member set; running it per crate
//! turns the first-error contract into one Finding per non-conforming crate without re-deriving
//! the policy).
//!
//! ## Required prefix is CONFIG, not a hardcoded literal
//! The expected prefix comes from the oya-ci config `[naming].required_prefix` (default `oya-`,
//! OYA-CI-CONFORMANCE-FLOOR-PLAN §3.3). The bundled default reproduces today's `oya-`, so the
//! gate's findings are byte-for-byte unchanged.
//!
//! ## Contract
//! `evaluate_keyed` returns one `Finding{code,key}` per violation (`key` = crate name — the
//! member-path crate-id when resolvable, else the package name, else the member path);
//! `evaluate` is the bare-code projection. The producer's `build_gate_baseline` freezes today's
//! keys (`mode: baseline-block-on-new`) so only NEW non-conforming crates block.
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use oya_ci_config_kernel::NamingConfig;
use oya_intelligence_cargo_prefix_domain::{validate_cargo_prefix, CargoPrefixError, CargoPrefixMember};
use serde_json::Value;

/// The gate id, matching the buck2 target + the firewall baseline gate-id.
pub const GATE_ID: &str = "cloud-ci-cargo-prefix";

/// The blocking violation codes (stable slugs the gate emits). Today's corpus only trips
/// `cargo_prefix_violation` (a member-path crate-id or package name that does not start with the
/// required prefix); `cargo_prefix_name_path_mismatch` defends the case where both carry the
/// prefix but disagree, and `cargo_prefix_unresolvable` the malformed-member-path case.
pub const VIOLATION_CODES: [&str; 3] = [
    "cargo_prefix_violation",
    "cargo_prefix_name_path_mismatch",
    "cargo_prefix_unresolvable",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

/// A keyed violation: the stable `code` plus the offending crate `key`.
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

/// Map a fail-fast `CargoPrefixError` (over a single member) to this gate's stable `(code, key)`.
/// The key prefers the member-path crate-id (the dev-cli's notion of identity), falling back to
/// the package name, then the raw member path — so a Finding always carries a human-recognizable
/// crate handle even for the malformed-path case.
fn finding_for(error: &CargoPrefixError) -> Finding {
    match error {
        CargoPrefixError::MemberPathPrefixViolation { crate_id, .. } => {
            Finding::new("cargo_prefix_violation", crate_id)
        }
        CargoPrefixError::PackageNamePrefixViolation { package_name, .. } => {
            Finding::new("cargo_prefix_violation", package_name)
        }
        CargoPrefixError::PackageNamePathMismatch { crate_id, .. } => {
            Finding::new("cargo_prefix_name_path_mismatch", crate_id)
        }
        CargoPrefixError::MemberPathMissingCrateId { member_path } => {
            Finding::new("cargo_prefix_unresolvable", member_path)
        }
        // `EmptyPrefix` / `NoWorkspaceMembers` are config/input errors, not per-crate violations;
        // they cannot arise on the per-member path with a configured prefix (the producer only
        // emits rows under a non-empty prefix). Treat any leak as an unresolvable input.
        CargoPrefixError::EmptyPrefix => {
            Finding::new("cargo_prefix_unresolvable", "<empty-prefix>")
        }
        CargoPrefixError::NoWorkspaceMembers => {
            Finding::new("cargo_prefix_unresolvable", "<no-members>")
        }
    }
}

/// Resolve a row `{"member_path", "package_name"}` into the domain's [`CargoPrefixMember`].
fn member_from_row(row: &Value) -> Option<CargoPrefixMember> {
    let member_path = row.get("member_path").and_then(Value::as_str)?;
    let package_name = row.get("package_name").and_then(Value::as_str)?;
    Some(CargoPrefixMember {
        member_path: member_path.to_owned(),
        package_name: package_name.to_owned(),
    })
}

/// Pure evaluator: takes `{"rows": [{"member_path": "...", "package_name": "..."}, ...]}` and
/// returns one `Finding` per cargo-prefix violation. Reuses
/// `oya_intelligence_cargo_prefix_domain::validate_cargo_prefix` per crate (surface-all).
pub fn evaluate_keyed(input: &Value) -> BTreeSet<Finding> {
    // The required prefix is sourced from the oya-ci config `[naming]` section. In this floor the
    // gate uses the bundled default (== today's `oya-`), so findings are byte-identical; the
    // producer's input-binding routes the live config when it builds the rows.
    let prefix = NamingConfig::default().required_prefix;
    let rows = input
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut findings = BTreeSet::new();
    for row in &rows {
        let Some(member) = member_from_row(row) else {
            continue;
        };
        // Run the policy over a SINGLE member so the first-error contract yields a per-crate
        // verdict (surface-all): a conforming member is `Ok`, a non-conforming one is the `Err`
        // describing its single violation.
        if let Err(error) = validate_cargo_prefix([member], &prefix) {
            findings.insert(finding_for(&error));
        }
    }
    findings
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

    fn rows(members: &[(&str, &str)]) -> Value {
        json!({
            "rows": members
                .iter()
                .map(|(path, name)| json!({ "member_path": path, "package_name": name }))
                .collect::<Vec<_>>()
        })
    }

    #[test]
    fn conformant_members_are_green() {
        let input = rows(&[
            ("libs/oya-check-brand-residue", "oya-check-brand-residue"),
            ("oya/developer-sdk/crates/oya-dev-cli", "oya-dev-cli"),
        ]);
        let report = evaluate(&input);
        assert_eq!(report.verdict, Verdict::Green, "got {:?}", report.violations);
        assert!(evaluate_keyed(&input).is_empty());
    }

    #[test]
    fn unprefixed_member_path_fires_violation() {
        let input = rows(&[("crates/acme-capability-kernel", "oya-intelligence-capability-kernel")]);
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1, "got {findings:?}");
        let f = findings.iter().next().unwrap();
        assert_eq!(f.code, "cargo_prefix_violation");
        assert_eq!(f.key, "acme-capability-kernel");
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn unprefixed_package_name_fires_violation() {
        let input = rows(&[("crates/oya-intelligence-capability-kernel", "acme-capability-kernel")]);
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1, "got {findings:?}");
        let f = findings.iter().next().unwrap();
        assert_eq!(f.code, "cargo_prefix_violation");
        assert_eq!(f.key, "acme-capability-kernel");
    }

    #[test]
    fn name_path_mismatch_fires_its_code() {
        // both carry the prefix but the package name disagrees with the member-path crate-id.
        let input = rows(&[(
            "crates/oya-intelligence-capability-kernel",
            "oya-intelligence-policy-kernel",
        )]);
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1, "got {findings:?}");
        let f = findings.iter().next().unwrap();
        assert_eq!(f.code, "cargo_prefix_name_path_mismatch");
        assert_eq!(f.key, "oya-intelligence-capability-kernel");
    }

    #[test]
    fn surface_all_one_finding_per_non_conformant_crate() {
        let input = rows(&[
            ("libs/oya-good-kernel", "oya-good-kernel"),
            ("crates/acme-bad", "oya-intelligence-acme-bad"),
            ("crates/oya-other", "widget-other"),
        ]);
        let findings = evaluate_keyed(&input);
        // two non-conforming crates → two findings (the conforming one contributes nothing).
        assert_eq!(findings.len(), 2, "got {findings:?}");
        assert!(findings.iter().all(|f| f.code == "cargo_prefix_violation"));
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn evaluate_is_bare_projection_of_evaluate_keyed() {
        let input = rows(&[
            ("crates/acme-bad", "oya-acme-bad"),
            ("libs/oya-good-domain", "oya-good-domain"),
        ]);
        let projected: BTreeSet<String> =
            evaluate_keyed(&input).into_iter().map(|f| f.code).collect();
        assert_eq!(evaluate(&input).violations, projected);
    }

    #[test]
    fn empty_corpus_is_green() {
        assert_eq!(evaluate(&json!({ "rows": [] })).verdict, Verdict::Green);
    }
}
