//! Predictable-naming fitness kernel (M-CC-P04-IP-002).
//!
//! Validates workspace crate names against the
//! `<context>-<bounded-context>-<capability>-<role>` convention so that
//! navigability tooling can infer ownership and architectural layer
//! from a crate name alone.
//!
//! I/O-free. Runners walk Cargo metadata + registry/catalog and feed
//! typed [`CrateNaming`] records into [`check`].

pub const REQUIRED_PREFIX: &str = "oya-";
pub const ALLOWED_ROLES: [&str; 13] = [
    "kernel",
    "domain",
    "application",
    "app",
    "adapter",
    "infrastructure",
    "rest",
    "grpc",
    "graphql",
    "cli",
    "worker",
    "sdk",
    "runtime",
];

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
    pub crates_checked: usize,
    pub violations: Vec<NamingViolation>,
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

        let inferred = infer_role(&row.crate_name).unwrap_or("");

        match &row.declared_role {
            None => violations.push(NamingViolation {
                crate_name: row.crate_name.clone(),
                kind: NamingViolationKind::UndeclaredRole,
            }),
            Some(declared) => {
                if !ALLOWED_ROLES.contains(&declared.as_str()) {
                    violations.push(NamingViolation {
                        crate_name: row.crate_name.clone(),
                        kind: NamingViolationKind::UnknownRole {
                            role: declared.clone(),
                        },
                    });
                } else if ALLOWED_ROLES.contains(&inferred) && declared != inferred {
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
            "oya-foundry-account-kernel",
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
            "oya-foundry-account-domain",
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
        let r = check(&[row("oya-foundry-account-kernel", None, Some("foundry"))]).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == NamingViolationKind::UndeclaredRole)
        );
    }

    #[test]
    fn undeclared_context_flagged() {
        let r = check(&[row("oya-foundry-account-kernel", Some("kernel"), None)]).unwrap();
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
}
