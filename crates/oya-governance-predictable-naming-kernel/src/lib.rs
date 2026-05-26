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

/// The 13-value canonical layer enum per ADR-0056 (amended by ADR-0105
/// and ADR-0106).
///
/// History:
///   2026-05-15 ADR-0105: added `api` (12 → 13).
///   2026-05-15 ADR-0106: renamed `application` → `usecase`.
///   2026-05-15 ADR-0107 (Superseded by self): removed the
///     "tools/-implicit-app" carve-out. Every tools/ crate MUST take a
///     canonical layer suffix; binary tools use `-app`. The only
///     doctrinal lock is `oya-tooling-agent-read` (ADR-0053 sanctioned
///     primitive), recorded in `DOCTRINAL_CARVE_OUTS` below — NOT a
///     layer-enum exception.
///   `runtime` and `test` were removed: `runtime` is slated for
///   per-crate rename to `app` per ADR-0056 §"Concrete migration";
///   `test` was never in the canonical enum (test-only crates take
///   canonical layer suffixes like any other).
pub const ALLOWED_ROLES: [&str; 13] = [
    "kernel",
    "domain",
    "usecase",
    "app",
    "adapter",
    "infrastructure",
    "cli",
    "rest",
    "grpc",
    "graphql",
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
/// higher-tier contract (CLAUDE.md sanctioned primitives per ADR-0053),
/// NOT by a layer-enum exception. These crates are exempted from the
/// canonical-suffix requirement only because their name is part of the
/// agent-operating contract.
///
/// This set is closed. Adding an entry requires a new ADR (do NOT
/// expand without one).
///
/// Sole entry as of ADR-0107's 2026-05-15 supersede: `oya-tooling-agent-read`.
pub const DOCTRINAL_CARVE_OUTS: [&str; 1] = ["oya-tooling-agent-read"];

/// True iff the crate name is in the closed `DOCTRINAL_CARVE_OUTS` set.
pub fn is_doctrinal_carve_out(crate_name: &str) -> bool {
    DOCTRINAL_CARVE_OUTS.contains(&crate_name)
}

/// True iff the crate name follows the `oya-check-<feature>` pattern, in
/// which case the role-requirement is satisfied by the prefix alone.
pub fn is_check_family(crate_name: &str) -> bool {
    crate_name.starts_with(CHECK_FAMILY_PREFIX)
        && crate_name.len() > CHECK_FAMILY_PREFIX.len()
        && !crate_name[CHECK_FAMILY_PREFIX.len()..].contains('/')
}

/// True iff the crate name follows the `*-adapter-<backend>` pattern,
/// where `<backend>` is in [`BACKEND_SUFFIXES`]. In that case the
/// effective layer is `adapter` (the trailing token is the backend
/// qualifier, not the layer).
pub fn is_backend_qualified_adapter(crate_name: &str) -> bool {
    let segments: Vec<&str> = crate_name.split('-').collect();
    if segments.len() < 2 {
        return false;
    }
    let last = segments[segments.len() - 1];
    let penult = segments[segments.len() - 2];
    penult == "adapter" && BACKEND_SUFFIXES.contains(&last)
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

pub fn check(rows: &[CrateNaming]) -> Result<NamingReport, NamingError> {
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
        let Some(rest) = row.crate_name.strip_prefix(REQUIRED_PREFIX) else {
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
        // Doctrinal carve-outs (CLAUDE.md sanctioned primitives per
        // ADR-0053) are exempt from canonical-suffix enforcement — NOT
        // because of an enum exception, but because their name is
        // locked at the agent-operating-contract layer.
        let check_family = is_check_family(&row.crate_name);
        let backend_qualified = is_backend_qualified_adapter(&row.crate_name);
        let doctrinal_carve_out = is_doctrinal_carve_out(&row.crate_name);

        let inferred = if backend_qualified {
            "adapter"
        } else if check_family {
            // No layer to infer; the family is the layer.
            ""
        } else {
            infer_role(&row.crate_name).unwrap_or("")
        };

        match &row.declared_role {
            None => {
                // check-family is self-layering (no declared_role required).
                // Doctrinal carve-outs (ADR-0053 sanctioned primitives) are
                // exempt — their name is locked by the agent-operating contract.
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
                if !ALLOWED_ROLES.contains(&declared.as_str()) {
                    violations.push(NamingViolation {
                        crate_name: row.crate_name.clone(),
                        kind: NamingViolationKind::UnknownRole {
                            role: declared.clone(),
                        },
                    });
                } else if !check_family && ALLOWED_ROLES.contains(&inferred) && declared != inferred
                {
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
            "oya-foundry-account-helper",
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
    fn allowed_roles_match_13_value_canonical_enum() {
        // Regression for ADR-0105 (added `api`) + ADR-0106 (renamed
        // `application` → `usecase`). Old enum had 14 roles incl
        // runtime+test; new canonical is 13 incl api+usecase.
        assert_eq!(ALLOWED_ROLES.len(), 13);
        assert!(ALLOWED_ROLES.contains(&"api"));
        assert!(ALLOWED_ROLES.contains(&"usecase"));
        assert!(!ALLOWED_ROLES.contains(&"application"));
        assert!(!ALLOWED_ROLES.contains(&"runtime"));
        assert!(!ALLOWED_ROLES.contains(&"test"));
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
            "oya-foundry-account-runtime",
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
        assert!(!is_check_family("oya-foundry-check-x"));
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
        assert!(!is_backend_qualified_adapter("oya-foundry-account-adapter")); // no backend
        assert!(!is_backend_qualified_adapter(
            "oya-intelligence-account-domain"
        ));
        assert!(!is_backend_qualified_adapter("oya-adapter-foo")); // 'adapter' not penultimate-after-something
    }

    #[test]
    fn is_doctrinal_carve_out_helper() {
        // Sole entry per ADR-0107 supersede 2026-05-15.
        assert!(is_doctrinal_carve_out("oya-tooling-agent-read"));
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
