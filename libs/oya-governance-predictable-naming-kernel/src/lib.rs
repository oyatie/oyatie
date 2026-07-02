//! Predictable-naming fitness kernel (M01-P11-IP-002).
//!
//! Validates workspace crate names against the
//! `<context>-<bounded-context>-<capability>-<role>` convention so that
//! navigability tooling can infer ownership and architectural layer
//! from a crate name alone.
//!
//! I/O-free. Runners walk Cargo metadata + registry/catalog and feed
//! typed [`CrateNaming`] records into [`check`].
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub const REQUIRED_PREFIX: &str = "oya-";

/// The injectable naming policy (OYA-CI-CONFORMANCE-FLOOR-PLAN §3.3): the prefix + role
/// enum + adopted-pattern tables that were hardcoded as the `const`s below, lifted into a
/// value the producer can source from `oya-ci.toml`'s `[naming]` section. The bundled
/// default ([`NamingPolicy::bundled_default`]) reproduces the `const`s exactly, so
/// [`check_with_policy`] with the default is byte-for-byte identical to the legacy [`check`].
///
/// The kernel stays dependency-free: this is a plain owned struct; the config crate
/// (`oya-ci-config-kernel`) maps its `[naming]` section onto it at the producer boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamingPolicy {
    pub required_prefix: String,
    pub allowed_roles: Vec<String>,
    pub check_family_prefix: String,
    pub backend_suffixes: Vec<String>,
    pub doctrinal_carve_outs: Vec<String>,
}

impl NamingPolicy {
    /// The bundled default — byte-for-byte the legacy `const`s (REQUIRED_PREFIX / ALLOWED_ROLES
    /// / CHECK_FAMILY_PREFIX / BACKEND_SUFFIXES / DOCTRINAL_CARVE_OUTS).
    pub fn bundled_default() -> Self {
        Self {
            required_prefix: REQUIRED_PREFIX.to_owned(),
            allowed_roles: ALLOWED_ROLES.iter().map(|s| (*s).to_owned()).collect(),
            check_family_prefix: CHECK_FAMILY_PREFIX.to_owned(),
            backend_suffixes: BACKEND_SUFFIXES.iter().map(|s| (*s).to_owned()).collect(),
            doctrinal_carve_outs: DOCTRINAL_CARVE_OUTS
                .iter()
                .map(|s| (*s).to_owned())
                .collect(),
        }
    }
}

impl Default for NamingPolicy {
    fn default() -> Self {
        Self::bundled_default()
    }
}

/// The 12-value canonical layer enum per ADR-0056 (amended by ADR-0105,
/// ADR-0106, and ADR-0565).
///
/// History:
///   2026-05-15 ADR-0105: added `api` (12 → 13).
///   2026-05-15 ADR-0106: renamed `application` → `usecase`.
///   2026-06-21 ADR-0565: removed `graphql` (13 → 12). The owned stack
///     carries NO GraphQL surface; de-blessing the role here makes this
///     gate fail-CLOSED against any future `*-graphql` crate.
///   2026-05-15 ADR-0107 (Superseded by self): removed the
///     "tools/-implicit-app" carve-out. Every tools/ crate MUST take a
///     canonical layer suffix; binary tools use `-app`. Doctrinal locks
///     (`oya-tooling-agent-read` per ADR-0053, `oya-ci-gate-contract` per
///     ADR-0528) are recorded in `DOCTRINAL_CARVE_OUTS` below — NOT
///     layer-enum exceptions.
///   `runtime` and `test` were removed: `runtime` is slated for
///   per-crate rename to `app` per ADR-0056 §"Concrete migration";
///   `test` was never in the canonical enum (test-only crates take
///   canonical layer suffixes like any other).
pub const ALLOWED_ROLES: [&str; 12] = [
    "kernel",
    "domain",
    "usecase",
    "app",
    "adapter",
    "infrastructure",
    "cli",
    "rest",
    "grpc",
    "worker",
    "sdk",
    "api",
];

/// Crate-name prefix that marks the check-family self-layering convention
/// (ADR-0105 §Adopted Patterns). Crates matching `^oya-check-<feature>$`
/// are dual-purpose lib+bin µservices where the feature IS the layer; no
/// `-kernel`/`-app` suffix is required.
pub const CHECK_FAMILY_PREFIX: &str = "oya-check-";

/// Allowed backend-qualifier suffixes for the `*-adapter-<backend>`
/// pattern (ADR-0105 §Adopted Patterns). The layer is `adapter`; the
/// backend is a sub-suffix denoting which external system.
pub const BACKEND_SUFFIXES: [&str; 9] = [
    "fake", "inmemory", "aws", "oci", "gcp", "azure", "postgres", "redis", "sqlite",
];

/// Doctrinal carve-outs: crate names whose name is locked by a
/// higher-tier contract (sanctioned primitive per ADR-0053 or semver'd gate contract per
/// ADR-0528),
/// NOT by a layer-enum exception. These crates are exempted from the
/// canonical-suffix requirement only because their name is part of the
/// agent/gate-operating contract.
///
/// This set is closed. Adding an entry requires a new ADR (do NOT
/// expand without one).
///
/// Entries as of ADR-0528 REMED-001: `oya-tooling-agent-read`, `oya-ci-gate-contract`.
pub const DOCTRINAL_CARVE_OUTS: [&str; 2] = ["oya-tooling-agent-read", "oya-ci-gate-contract"];

/// True iff the crate name is in the policy's closed doctrinal-carve-out set.
pub fn is_doctrinal_carve_out_with(crate_name: &str, policy: &NamingPolicy) -> bool {
    policy.doctrinal_carve_outs.iter().any(|c| c == crate_name)
}

/// True iff the crate name follows the `<check_family_prefix><feature>` pattern, in which
/// case the role-requirement is satisfied by the prefix alone.
pub fn is_check_family_with(crate_name: &str, policy: &NamingPolicy) -> bool {
    let prefix = policy.check_family_prefix.as_str();
    crate_name.starts_with(prefix)
        && crate_name.len() > prefix.len()
        && !crate_name[prefix.len()..].contains('/')
}

/// True iff the crate name follows the `*-adapter-<backend>` pattern, where `<backend>` is in
/// the policy's `backend_suffixes`. In that case the effective layer is `adapter`.
pub fn is_backend_qualified_adapter_with(crate_name: &str, policy: &NamingPolicy) -> bool {
    let segments: Vec<&str> = crate_name.split('-').collect();
    if segments.len() < 2 {
        return false;
    }
    let last = segments[segments.len() - 1];
    let penult = segments[segments.len() - 2];
    penult == "adapter" && policy.backend_suffixes.iter().any(|b| b == last)
}

/// True iff the crate name is in the closed `DOCTRINAL_CARVE_OUTS` set (bundled-default
/// projection of [`is_doctrinal_carve_out_with`]).
pub fn is_doctrinal_carve_out(crate_name: &str) -> bool {
    is_doctrinal_carve_out_with(crate_name, &NamingPolicy::bundled_default())
}

/// True iff the crate name follows the `oya-check-<feature>` pattern (bundled-default
/// projection of [`is_check_family_with`]).
pub fn is_check_family(crate_name: &str) -> bool {
    is_check_family_with(crate_name, &NamingPolicy::bundled_default())
}

/// True iff the crate name follows the `*-adapter-<backend>` pattern (bundled-default
/// projection of [`is_backend_qualified_adapter_with`]).
pub fn is_backend_qualified_adapter(crate_name: &str) -> bool {
    is_backend_qualified_adapter_with(crate_name, &NamingPolicy::bundled_default())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrateNaming {
    pub crate_name: String,               // data_class: INTERNAL_ONLY
    pub declared_role: Option<String>,    // data_class: INTERNAL_ONLY
    pub declared_context: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum NamingViolationKind {
    MissingOyaPrefix,
    EmptyAfterPrefix,
    RoleMismatch { declared: String, inferred: String },
    UnknownRole { role: String },
    UndeclaredRole,
    UndeclaredContext,
    NameContainsUppercase,
}

impl NamingViolationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingOyaPrefix => "crate name does not start with 'oya-'",
            Self::EmptyAfterPrefix => "crate name has nothing after 'oya-' prefix",
            Self::RoleMismatch { .. } => "trailing-segment role does not match catalog role",
            Self::UnknownRole { .. } => "role is not in the allowed set",
            Self::UndeclaredRole => "catalog entry is missing role declaration",
            Self::UndeclaredContext => "catalog entry is missing context declaration",
            Self::NameContainsUppercase => "crate name contains uppercase characters",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamingViolation {
    pub crate_name: String,        // data_class: INTERNAL_ONLY
    pub kind: NamingViolationKind, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamingReport {
    // data_class: INTERNAL_ONLY
    pub crates_checked: usize, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub violations: Vec<NamingViolation>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NamingError {
    EmptyCrateName,
}

impl NamingError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyCrateName => "crate name is empty".to_owned(),
        }
    }
}

/// Infer the role from the trailing dash-segment of the crate name.
fn infer_role(crate_name: &str) -> Option<&str> {
    crate_name.rsplit_once('-').map(|(_, role)| role)
}

/// Validate crate names against an INJECTED [`NamingPolicy`] (the config-driven entry point;
/// OYA-CI-CONFORMANCE-FLOOR-PLAN §3.3). [`check`] is the bundled-default projection of this.
pub fn check_with_policy(
    rows: &[CrateNaming],
    policy: &NamingPolicy,
) -> Result<NamingReport, NamingError> {
    let mut violations = Vec::new();
    for row in rows {
        if row.crate_name.is_empty() {
            return Err(NamingError::EmptyCrateName);
        }
        if !row.crate_name.chars().all(|c| !c.is_ascii_uppercase()) {
            violations.push(NamingViolation {
                crate_name: row.crate_name.clone(),
                kind: NamingViolationKind::NameContainsUppercase,
            });
        }
        let Some(rest) = row.crate_name.strip_prefix(policy.required_prefix.as_str()) else {
            violations.push(NamingViolation {
                crate_name: row.crate_name.clone(),
                kind: NamingViolationKind::MissingOyaPrefix,
            });
            continue;
        };
        if rest.is_empty() {
            violations.push(NamingViolation {
                crate_name: row.crate_name.clone(),
                kind: NamingViolationKind::EmptyAfterPrefix,
            });
            continue;
        }

        // Adopted-pattern shortcuts (ADR-0105 §Adopted Patterns).
        // check-family is self-layering — no declared_role required.
        // backend-qualified adapter has effective layer `adapter`.
        // Doctrinal carve-outs (ADR-0053/ADR-0528 locked names) are exempt from
        // canonical-suffix enforcement — NOT because of an enum exception, but
        // because their name is locked at a higher contract layer.
        let check_family = is_check_family_with(&row.crate_name, policy);
        let backend_qualified = is_backend_qualified_adapter_with(&row.crate_name, policy);
        let doctrinal_carve_out = is_doctrinal_carve_out_with(&row.crate_name, policy);

        let inferred = if backend_qualified {
            "adapter"
        } else if check_family {
            // No layer to infer; the family is the layer.
            ""
        } else {
            infer_role(&row.crate_name).unwrap_or("")
        };
        let role_allowed = |role: &str| policy.allowed_roles.iter().any(|r| r == role);

        match &row.declared_role {
            None => {
                // check-family is self-layering (no declared_role required).
                // Doctrinal carve-outs are exempt — their name is locked by a
                // higher contract.
                // All other crates (including tools/* per ADR-0107 amendment
                // 2026-05-15) MUST declare a role; there is no directory-
                // implicit naming surface.
                if !check_family && !doctrinal_carve_out {
                    violations.push(NamingViolation {
                        crate_name: row.crate_name.clone(),
                        kind: NamingViolationKind::UndeclaredRole,
                    });
                }
            }
            Some(declared) => {
                if !role_allowed(declared.as_str()) {
                    violations.push(NamingViolation {
                        crate_name: row.crate_name.clone(),
                        kind: NamingViolationKind::UnknownRole {
                            role: declared.clone(),
                        },
                    });
                } else if !check_family && role_allowed(inferred) && declared != inferred {
                    violations.push(NamingViolation {
                        crate_name: row.crate_name.clone(),
                        kind: NamingViolationKind::RoleMismatch {
                            declared: declared.clone(),
                            inferred: inferred.to_owned(),
                        },
                    });
                }
            }
        }

        if row.declared_context.is_none() {
            violations.push(NamingViolation {
                crate_name: row.crate_name.clone(),
                kind: NamingViolationKind::UndeclaredContext,
            });
        }
    }

    Ok(NamingReport {
        crates_checked: rows.len(),
        violations,
    })
}

/// Validate crate names against the bundled-default policy (the legacy `const`-backed
/// contract). Byte-for-byte equivalent to `check_with_policy(rows, &NamingPolicy::bundled_default())`.
pub fn check(rows: &[CrateNaming]) -> Result<NamingReport, NamingError> {
    check_with_policy(rows, &NamingPolicy::bundled_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str, role: Option<&str>, ctx: Option<&str>) -> CrateNaming {
        CrateNaming {
            crate_name: name.into(),
            declared_role: role.map(String::from),
            declared_context: ctx.map(String::from),
        }
    }

    #[test]
    fn well_formed_crate_passes() {
        let r = check(&[row(
            "oya-intelligence-account-kernel",
            Some("kernel"),
            Some("foundry"),
        )])
        .unwrap();
        assert!(r.violations.is_empty(), "{:?}", r.violations);
    }

    #[test]
    fn missing_oya_prefix_flagged() {
        let r = check(&[row(
            "foundry-account-kernel",
            Some("kernel"),
            Some("foundry"),
        )])
        .unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == NamingViolationKind::MissingOyaPrefix)
        );
    }

    #[test]
    fn empty_after_prefix_flagged() {
        let r = check(&[row("oya-", Some("kernel"), Some("foundry"))]).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == NamingViolationKind::EmptyAfterPrefix)
        );
    }

    #[test]
    fn role_mismatch_flagged() {
        let r = check(&[row(
            "oya-intelligence-account-domain",
            Some("kernel"),
            Some("foundry"),
        )])
        .unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| matches!(v.kind, NamingViolationKind::RoleMismatch { .. }))
        );
    }

    #[test]
    fn unknown_role_flagged() {
        let r = check(&[row(
            "oya-intelligence-account-helper",
            Some("helper"),
            Some("foundry"),
        )])
        .unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| matches!(v.kind, NamingViolationKind::UnknownRole { .. }))
        );
    }

    #[test]
    fn undeclared_role_flagged() {
        let r = check(&[row(
            "oya-intelligence-account-kernel",
            None,
            Some("foundry"),
        )])
        .unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == NamingViolationKind::UndeclaredRole)
        );
    }

    #[test]
    fn undeclared_context_flagged() {
        let r = check(&[row("oya-intelligence-account-kernel", Some("kernel"), None)]).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == NamingViolationKind::UndeclaredContext)
        );
    }

    #[test]
    fn uppercase_in_name_flagged() {
        let r = check(&[row(
            "Oya-Foundry-Account-Kernel",
            Some("kernel"),
            Some("foundry"),
        )])
        .unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == NamingViolationKind::NameContainsUppercase)
        );
    }

    #[test]
    fn empty_input_returns_empty_report() {
        let r = check(&[]).unwrap();
        assert_eq!(r.crates_checked, 0);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn empty_crate_name_errors() {
        assert!(matches!(
            check(&[row("", None, None)]),
            Err(NamingError::EmptyCrateName)
        ));
    }

    #[test]
    fn allowed_roles_set_distinct() {
        use std::collections::HashSet;
        let s: HashSet<_> = ALLOWED_ROLES.iter().collect();
        assert_eq!(s.len(), ALLOWED_ROLES.len());
    }

    #[test]
    fn allowed_roles_match_12_value_canonical_enum() {
        // Regression for ADR-0105 (added `api`) + ADR-0106 (renamed
        // `application` → `usecase`) + ADR-0565 (removed `graphql`).
        // Old enum had 14 roles incl runtime+test; ADR-0105/0106 gave 13
        // incl api+usecase; ADR-0565 dropped graphql, leaving 12.
        assert_eq!(ALLOWED_ROLES.len(), 12);
        assert!(ALLOWED_ROLES.contains(&"api"));
        assert!(ALLOWED_ROLES.contains(&"usecase"));
        assert!(!ALLOWED_ROLES.contains(&"application"));
        assert!(!ALLOWED_ROLES.contains(&"runtime"));
        assert!(!ALLOWED_ROLES.contains(&"test"));
        // ADR-0565: the owned stack carries NO GraphQL surface, so the
        // role is de-blessed; this keeps the naming gate fail-CLOSED
        // against any future `*-graphql` crate.
        assert!(!ALLOWED_ROLES.contains(&"graphql"));
    }

    #[test]
    fn check_family_recognized_no_declared_role_required() {
        // Per ADR-0105: oya-check-<feature> is self-layering.
        let r = check(&[row("oya-check-brand-residue", None, Some("foundry"))]).unwrap();
        assert!(
            !r.violations
                .iter()
                .any(|v| v.kind == NamingViolationKind::UndeclaredRole),
            "{:?}",
            r.violations
        );
    }

    #[test]
    fn backend_qualified_adapter_layer_inferred_correctly() {
        // Per ADR-0105: oya-intelligence-account-adapter-inmemory has layer
        // `adapter`, not `inmemory`. Declared role `adapter` should match.
        let r = check(&[row(
            "oya-intelligence-account-adapter-inmemory",
            Some("adapter"),
            Some("foundry"),
        )])
        .unwrap();
        assert!(
            !r.violations
                .iter()
                .any(|v| matches!(v.kind, NamingViolationKind::RoleMismatch { .. })),
            "{:?}",
            r.violations
        );
    }

    #[test]
    fn usecase_layer_accepted() {
        let r = check(&[row("oya-dsr-usecase", Some("usecase"), Some("dsr"))]).unwrap();
        assert!(r.violations.is_empty(), "{:?}", r.violations);
    }

    #[test]
    fn api_layer_accepted() {
        let r = check(&[row("oya-cloud-compute-vm-api", Some("api"), Some("cloud"))]).unwrap();
        assert!(r.violations.is_empty(), "{:?}", r.violations);
    }

    #[test]
    fn runtime_role_no_longer_accepted() {
        let r = check(&[row(
            "oya-intelligence-account-runtime",
            Some("runtime"),
            Some("foundry"),
        )])
        .unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| matches!(v.kind, NamingViolationKind::UnknownRole { .. })),
            "{:?}",
            r.violations
        );
    }

    #[test]
    fn is_check_family_helper() {
        assert!(is_check_family("oya-check-brand-residue"));
        assert!(is_check_family("oya-check-x"));
        assert!(!is_check_family("oya-check-"));
        assert!(!is_check_family("oya-checker"));
        assert!(!is_check_family("oya-governance-check-x"));
    }

    #[test]
    fn is_backend_qualified_adapter_helper() {
        assert!(is_backend_qualified_adapter(
            "oya-intelligence-account-adapter-inmemory"
        ));
        assert!(is_backend_qualified_adapter(
            "oya-cloud-billing-adapter-aws"
        ));
        assert!(is_backend_qualified_adapter("oya-x-adapter-postgres"));
        assert!(!is_backend_qualified_adapter(
            "oya-intelligence-account-adapter"
        )); // no backend
        assert!(!is_backend_qualified_adapter(
            "oya-intelligence-account-domain"
        ));
        assert!(!is_backend_qualified_adapter("oya-adapter-foo")); // 'adapter' not penultimate-after-something
    }

    #[test]
    fn is_doctrinal_carve_out_helper() {
        // Closed entries per ADR-0107 and ADR-0528.
        assert!(is_doctrinal_carve_out("oya-tooling-agent-read"));
        assert!(is_doctrinal_carve_out("oya-ci-gate-contract"));
        // Random non-carve-out names are not exempt.
        assert!(!is_doctrinal_carve_out("oya-governance-portfolio-citation"));
        assert!(!is_doctrinal_carve_out("oya-adapter-substitution-test"));
        assert!(!is_doctrinal_carve_out(""));
    }

    #[test]
    fn doctrinal_carve_out_passes_without_role() {
        // ADR-0053 sanctioned primitive — locked name, no canonical suffix
        // required at the layer-enum surface. Must NOT raise UndeclaredRole.
        let r = check(&[row("oya-tooling-agent-read", None, Some("tooling"))]).unwrap();
        assert!(
            !r.violations
                .iter()
                .any(|v| v.kind == NamingViolationKind::UndeclaredRole),
            "doctrinal carve-out incorrectly flagged: {:?}",
            r.violations
        );

        // ADR-0528 semver'd gate-contract crate — locked by doctrine, not by adding a broad
        // `contract` layer role.
        let r = check(&[row("oya-ci-gate-contract", None, Some("cloud-ci"))]).unwrap();
        assert!(
            !r.violations
                .iter()
                .any(|v| v.kind == NamingViolationKind::UndeclaredRole),
            "gate-contract doctrinal carve-out incorrectly flagged: {:?}",
            r.violations
        );
    }

    #[test]
    fn tools_crate_without_canonical_suffix_flagged() {
        // Post-ADR-0107 supersede 2026-05-15: tools/ crates are NOT
        // implicitly app-layer. A tools/ crate without a declared canonical
        // role MUST raise UndeclaredRole (no directory-implicit naming).
        let r = check(&[row(
            "oya-governance-portfolio-citation",
            None,
            Some("foundry"),
        )])
        .unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == NamingViolationKind::UndeclaredRole),
            "tools/ crate without canonical suffix not flagged: {:?}",
            r.violations
        );
    }
}
