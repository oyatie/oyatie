//! # cloud-ci-bnf-layer-suffix (§2.5#4 — MIG-PREREQ floor gate S1)
//!
//! Enforces the ADR-0056 BNF rule that every first-party `oya-*` crate's trailing
//! dash-segment is one of the 12 canonical layer values
//! (`kernel|domain|usecase|app|adapter|infrastructure|cli|rest|grpc|worker|sdk|api`).
//!
//! ## Reuse, not re-derive (CLI-GOVERNANCE-TO-FIREWALL-MIGRATION-PLAN Principle 1, shape a)
//! The policy lives in the PURE, I/O-free `check_predictable_naming_kernel::check()`.
//! The producer (`oya-cloud-ci-accounting-registry-app`) does the I/O — it enumerates the
//! first-party `oya-*` crate package names from the tracked Cargo.toml manifests and feeds them
//! as `rows`. This gate resolves each crate's `declared_role` CARVE-OUT-AWARE and runs `check()`:
//! - `oya-check-*` (self-layering check-family) and doctrinal carve-outs
//!   (`oya-tooling-agent-read`, `oya-ci-gate-contract`) → `declared_role = None` (exempt:
//!   `check()` skips them in the undeclared-role branch);
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

use oya_ci_config_kernel::NamingConfig;
use check_predictable_naming_kernel::{
    CrateNaming, NamingPolicy, NamingViolationKind, check_with_policy,
    is_backend_qualified_adapter_with, is_check_family_with, is_doctrinal_carve_out_with,
};
use serde_json::Value;

/// Map the oya-ci config `[naming]` section onto the kernel's injected [`NamingPolicy`]
/// (OYA-CI-CONFORMANCE-FLOOR-PLAN §3.3). The bundled default reproduces today's `const`s,
/// so the gate's findings are byte-for-byte unchanged.
fn naming_policy(cfg: &NamingConfig) -> NamingPolicy {
    NamingPolicy {
        required_prefix: cfg.required_prefix.clone(),
        allowed_roles: cfg.allowed_roles.clone(),
        check_family_prefix: cfg.check_family_prefix.clone(),
        backend_suffixes: cfg.backend_suffixes.clone(),
        doctrinal_carve_outs: cfg.doctrinal_carve_outs.clone(),
    }
}

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
fn resolve_naming(crate_name: &str, policy: &NamingPolicy) -> CrateNaming {
    let declared_role = if is_check_family_with(crate_name, policy)
        || is_doctrinal_carve_out_with(crate_name, policy)
    {
        // Self-layering (check-family) / doctrinal primitive: no declared role required.
        None
    } else if is_backend_qualified_adapter_with(crate_name, policy) {
        // `oya-<svc>-adapter-<backend>`: the effective layer is `adapter`.
        Some("adapter".to_owned())
    } else {
        // General case: the trailing dash-segment is the declared (== inferred) role.
        crate_name.rsplit_once('-').map(|(_, role)| role.to_owned())
    };
    CrateNaming {
        crate_name: crate_name.to_owned(),
        declared_role,
        declared_context: Some("inferred".to_owned()),
    }
}

/// Pure evaluator over an INJECTED [`NamingConfig`] (ADR-0533 §Decision item 1/2: the naming
/// policy — including `required_prefix` — is the PROFILE-RESOLVED config). Takes
/// `{"rows": [{"crate_name": "..."}, ...]}` and returns one `Finding` per layer-suffix
/// violation. Reuses `check_predictable_naming_kernel::check_with_policy`.
///
/// Under `profile='neutral'` the resolved `required_prefix` is empty, so `MissingOyaPrefix` is
/// never raised (de-brand). Under `profile='oyatie'` the policy is today's consts, byte-identical.
pub fn evaluate_keyed_with(input: &Value, naming: &NamingConfig) -> BTreeSet<Finding> {
    let policy = naming_policy(naming);
    let rows = match input.get("rows") {
        None => {
            let mut out = BTreeSet::new();
            out.insert(Finding::new("bnf_empty_after_prefix", "<missing-rows>"));
            return out;
        }
        Some(rows) => match rows.as_array() {
            Some(rows) if rows.is_empty() => {
                let mut out = BTreeSet::new();
                out.insert(Finding::new("bnf_empty_after_prefix", "<empty-rows>"));
                return out;
            }
            Some(rows) => rows,
            None => {
                let mut out = BTreeSet::new();
                out.insert(Finding::new("bnf_empty_after_prefix", "<malformed-rows>"));
                return out;
            }
        },
    };
    let mut findings = BTreeSet::new();
    let namings: Vec<CrateNaming> = rows
        .iter()
        .filter_map(|row| match row.get("crate_name").and_then(Value::as_str) {
            Some(name) => Some(name),
            None => {
                findings.insert(Finding::new("bnf_empty_after_prefix", "<malformed-row>"));
                None
            }
        })
        .map(|name| resolve_naming(name, &policy))
        .collect();
    let report = match check_with_policy(&namings, &policy) {
        Ok(report) => report,
        // An empty crate name is the only error; treat it as a single blocking finding so the
        // gate never silently passes on malformed input.
        Err(_) => {
            let mut out = BTreeSet::new();
            out.insert(Finding::new("bnf_empty_after_prefix", "<empty-crate-name>"));
            return out;
        }
    };
    findings.extend(
        report
            .violations
            .into_iter()
            .map(|v| Finding::new(code_slug(&v.kind), &v.crate_name)),
    );
    findings
}

/// Bundled-default (oyatie profile) projection of [`evaluate_keyed_with`]. The producer routes
/// the live profile-resolved config; this keeps the legacy pure surface byte-identical.
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
        assert_eq!(
            report.verdict,
            Verdict::Green,
            "got {:?}",
            report.violations
        );
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
        let input = rows(&[
            "oya-check-data-class",
            "oya-check-layered-architecture-discipline",
        ]);
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
    fn malformed_corpus_inputs_fail_closed() {
        for (input, sentinel) in [
            (json!({}), "<missing-rows>"),
            (json!({ "rows": "not-an-array" }), "<malformed-rows>"),
            (json!({ "rows": [] }), "<empty-rows>"),
            (json!({ "rows": [{}] }), "<malformed-row>"),
        ] {
            let findings = evaluate_keyed(&input);
            assert_eq!(
                evaluate(&input).verdict,
                Verdict::Red,
                "input must fail closed: {input:?}"
            );
            assert!(
                findings
                    .iter()
                    .any(|f| f.code == "bnf_empty_after_prefix" && f.key == sentinel),
                "expected bnf_empty_after_prefix sentinel {sentinel}, got {findings:?}"
            );
        }
    }

    /// ADR-0533 de-brand (item 2): under the NEUTRAL profile `required_prefix` is empty, so
    /// `bnf_missing_oya_prefix` is NOT raised for an unprefixed crate. Proves the prefix axis is
    /// profile-sourced. (The role-enum axis is separate; neutral leaves allowed_roles empty.)
    #[test]
    fn neutral_profile_does_not_raise_missing_prefix() {
        let input = rows(&["registry-drift", "acme-thing"]);
        let neutral = NamingConfig {
            required_prefix: String::new(),
            allowed_roles: vec![],
            check_family_prefix: String::new(),
            backend_suffixes: vec![],
            doctrinal_carve_outs: vec![],
        };
        let findings = evaluate_keyed_with(&input, &neutral);
        assert!(
            !findings.iter().any(|f| f.code == "bnf_missing_oya_prefix"),
            "neutral profile must not raise MissingOyaPrefix, got {findings:?}"
        );
        // The SAME corpus under oyatie DOES flag the unprefixed name (safety: oyatie unchanged).
        assert!(
            evaluate_keyed(&input)
                .iter()
                .any(|f| f.code == "bnf_missing_oya_prefix")
        );
    }

    /// The oyatie-profile config reproduces the bundled default exactly (byte-identity).
    #[test]
    fn oyatie_profile_matches_bundled_default() {
        let input = rows(&["oya-foo-runtime", "registry-drift"]);
        assert_eq!(
            evaluate_keyed_with(&input, &NamingConfig::default()),
            evaluate_keyed(&input)
        );
    }
}
