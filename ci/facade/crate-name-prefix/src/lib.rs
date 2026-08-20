//! # cloud-ci-cargo-prefix (ADR-0017 — §2.5-adjacent, MIG-PREREQ floor gate S3)
//!
//! Enforces the ADR-0017 cargo-prefix fitness rule for blocking-scoped workspace members: the
//! crate-id (its member-path leaf) AND its declared `[package].name` must begin with the required
//! prefix, and the two must agree.
//!
//! ## Reuse, not re-derive (CLI-GOVERNANCE-TO-FIREWALL-MIGRATION-PLAN Principle 1, shape b)
//! The policy lives in the PURE, I/O-free `intelligence_cargo_prefix_domain::validate_cargo_prefix`
//! — the SAME predicate the `oya gate validate cargo-prefix` dev-cli call uses (the firewall gate
//! and the dev-cli call coexist for now). The producer
//! (`oya-cloud-ci-accounting-registry-app`) does the I/O — it enumerates every tracked first-party
//! workspace member candidate and feeds each as a row of
//! `{"member_path", "package_name", "cargo_prefix_scope"}`. This gate runs
//! `validate_cargo_prefix` over each blocking-scoped row INDEPENDENTLY (a single-member iterator)
//! so the verdict is per-crate and surface-all, while advisory-scoped de-branded rows remain
//! visible in the face without becoming born-blocking baseline debt.
//!
//! ## Required prefix is CONFIG, not a hardcoded literal
//! The expected prefix comes from the oya-ci config `[naming].required_prefix` (default `oya-`,
//! OYA-CI-CONFORMANCE-FLOOR-PLAN §3.3). The bundled default keeps the prefix rule for rows whose
//! crate-id and package name still carry the configured brand prefix; de-branded rows are advisory.
//!
//! ## Contract
//! `evaluate_keyed` returns one `Finding{code,key}` per blocking-scoped violation (`key` = crate
//! name — the member-path crate-id when resolvable, else the package name, else the member path);
//! `evaluate` is the bare-code projection. Advisory-scoped rows are candidate coverage only and
//! cannot create new `baseline-block-on-new` cargo-prefix regressions.
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use intelligence_cargo_prefix_domain::{
    CargoPrefixError, CargoPrefixMember, validate_cargo_prefix,
};
use oya_ci_config_kernel::NamingConfig;
use serde_json::Value;

/// The gate id, matching the buck2 target + the firewall baseline gate-id.
pub const GATE_ID: &str = "cloud-ci-cargo-prefix";

/// The blocking violation codes (stable slugs the gate emits). `cargo_prefix_violation` covers a
/// blocking-scoped member-path crate-id or package name that does not start with the required
/// prefix; `cargo_prefix_name_path_mismatch` defends the case where both carry the prefix but
/// disagree, and `cargo_prefix_unresolvable` the malformed-member-path case.
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
/// Candidate rows scoped as advisory are visible producer coverage, not blocking cargo-prefix debt.
fn is_advisory_row(row: &Value) -> bool {
    row.get("cargo_prefix_scope").and_then(Value::as_str) == Some("advisory")
}

/// Pure evaluator over an INJECTED [`NamingConfig`] (ADR-0533 §Decision item 3: the required
/// prefix is the PROFILE-RESOLVED config loaded from `oya-ci.toml`, NOT a hardcoded literal).
/// Takes `{"rows": [{"member_path": "...", "package_name": "...", "cargo_prefix_scope": "..."},
/// ...]}` and returns one `Finding` per blocking-scoped cargo-prefix violation. Reuses
/// `intelligence_cargo_prefix_domain::validate_cargo_prefix` per crate (surface-all).
///
/// Under `profile='neutral'` the resolved `required_prefix` is empty, so no
/// `cargo_prefix_violation` is raised — the de-brand. Under `profile='oyatie'`, rows explicitly
/// scoped advisory by the producer are visible candidate coverage but do not block.
pub fn evaluate_keyed_with(input: &Value, naming: &NamingConfig) -> BTreeSet<Finding> {
    let prefix = naming.required_prefix.clone();
    let mut findings = BTreeSet::new();

    let Some(rows_value) = input.get("rows") else {
        findings.insert(Finding::new("cargo_prefix_unresolvable", "<missing-rows>"));
        return findings;
    };
    let Some(rows) = rows_value.as_array() else {
        findings.insert(Finding::new(
            "cargo_prefix_unresolvable",
            "<non-array-rows>",
        ));
        return findings;
    };
    if rows.is_empty() {
        findings.insert(Finding::new("cargo_prefix_unresolvable", "<empty-rows>"));
        return findings;
    }

    for row in rows {
        let Some(member) = member_from_row(row) else {
            findings.insert(Finding::new("cargo_prefix_unresolvable", "<malformed-row>"));
            continue;
        };
        if is_advisory_row(row) {
            continue;
        }
        // Run the policy over a SINGLE blocking-scoped member so the first-error contract yields
        // a per-crate verdict (surface-all): a conforming member is `Ok`, a non-conforming one is
        // the `Err` describing its single violation. An empty configured prefix (neutral profile)
        // makes every member advisory-equivalent — the de-brand path.
        if prefix.is_empty() {
            continue;
        }
        if let Err(error) = validate_cargo_prefix([member], &prefix) {
            findings.insert(finding_for(&error));
        }
    }
    findings
}

/// Bundled-default (oyatie profile) projection of [`evaluate_keyed_with`]. The producer routes
/// the live profile-resolved config via `evaluate_keyed_with`; this projection keeps the legacy
/// pure surface byte-identical (default `required_prefix == "oya-"`).
pub fn evaluate_keyed(input: &Value) -> BTreeSet<Finding> {
    evaluate_keyed_with(input, &NamingConfig::default())
}

/// Bare-code projection of [`evaluate_keyed_with`] over an injected config.
pub fn evaluate_with(input: &Value, naming: &NamingConfig) -> Report {
    let codes: BTreeSet<String> = evaluate_keyed_with(input, naming)
        .into_iter()
        .map(|f| f.code)
        .collect();
    Report::from_codes(codes)
}

/// Bare-code projection of [`evaluate_keyed`] — the single source of truth for the verdict.
pub fn evaluate(input: &Value) -> Report {
    evaluate_with(input, &NamingConfig::default())
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
        assert_eq!(
            report.verdict,
            Verdict::Green,
            "got {:?}",
            report.violations
        );
        assert!(evaluate_keyed(&input).is_empty());
    }

    #[test]
    fn unprefixed_member_path_fires_violation() {
        let input = rows(&[(
            "crates/acme-capability-kernel",
            "oya-intelligence-capability-kernel",
        )]);
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1, "got {findings:?}");
        let f = findings.iter().next().unwrap();
        assert_eq!(f.code, "cargo_prefix_violation");
        assert_eq!(f.key, "acme-capability-kernel");
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn unprefixed_package_name_fires_violation() {
        let input = rows(&[(
            "crates/oya-intelligence-capability-kernel",
            "acme-capability-kernel",
        )]);
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
    fn missing_rows_fail_closed_as_unresolvable() {
        let findings = evaluate_keyed(&json!({}));
        assert_eq!(findings.len(), 1, "got {findings:?}");
        let f = findings.iter().next().unwrap();
        assert_eq!(f.code, "cargo_prefix_unresolvable");
        assert_eq!(f.key, "<missing-rows>");
        assert_eq!(evaluate(&json!({})).verdict, Verdict::Red);
    }

    #[test]
    fn non_array_rows_fail_closed_as_unresolvable() {
        let input = json!({ "rows": "not-an-array" });
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1, "got {findings:?}");
        let f = findings.iter().next().unwrap();
        assert_eq!(f.code, "cargo_prefix_unresolvable");
        assert_eq!(f.key, "<non-array-rows>");
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn empty_corpus_fails_closed_as_unresolvable() {
        let input = json!({ "rows": [] });
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1, "got {findings:?}");
        let f = findings.iter().next().unwrap();
        assert_eq!(f.code, "cargo_prefix_unresolvable");
        assert_eq!(f.key, "<empty-rows>");
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn malformed_rows_fail_closed_as_unresolvable() {
        let input = json!({ "rows": [{}] });
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1, "got {findings:?}");
        let f = findings.iter().next().unwrap();
        assert_eq!(f.code, "cargo_prefix_unresolvable");
        assert_eq!(f.key, "<malformed-row>");
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn advisory_scoped_unprefixed_candidates_are_not_blocking_debt() {
        let input = json!({
            "rows": [
                {
                    "member_path": "crates/acme-capability-kernel",
                    "package_name": "acme-capability-kernel",
                    "cargo_prefix_scope": "advisory"
                },
                {
                    "member_path": "crates/oya-policy-kernel",
                    "package_name": "acme-policy-kernel",
                    "cargo_prefix_scope": "blocking"
                }
            ]
        });

        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1, "got {findings:?}");
        let f = findings.iter().next().unwrap();
        assert_eq!(f.code, "cargo_prefix_violation");
        assert_eq!(f.key, "acme-policy-kernel");
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }
    /// ADR-0533 de-brand: under the NEUTRAL profile the resolved `required_prefix` is empty, so a
    /// member that would trip `cargo_prefix_violation` under oyatie is GREEN — the prefix rule is
    /// de-branded. Proves the gate is profile-sourced, not hardcoded to `oya-`.
    #[test]
    fn neutral_profile_empty_prefix_raises_no_prefix_violation() {
        let input = rows(&[
            ("crates/acme-capability-kernel", "acme-capability-kernel"),
            ("crates/widget-thing", "widget-thing"),
        ]);
        let neutral = NamingConfig {
            required_prefix: String::new(),
            allowed_roles: vec![],
            check_family_prefix: String::new(),
            backend_suffixes: vec![],
            doctrinal_carve_outs: vec![],
        };
        let findings = evaluate_keyed_with(&input, &neutral);
        assert!(
            findings.is_empty(),
            "neutral profile must raise no prefix violations, got {findings:?}"
        );
        assert_eq!(evaluate_with(&input, &neutral).verdict, Verdict::Green);
        // The SAME corpus under the oyatie default still RED (safety property: oyatie unchanged).
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    /// The oyatie-profile config reproduces the bundled default exactly (byte-identity).
    #[test]
    fn oyatie_profile_matches_bundled_default() {
        let input = rows(&[("crates/acme-bad", "acme-bad")]);
        assert_eq!(
            evaluate_keyed_with(&input, &NamingConfig::default()),
            evaluate_keyed(&input)
        );
    }
}
