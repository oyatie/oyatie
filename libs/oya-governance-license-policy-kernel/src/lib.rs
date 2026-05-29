//! License-policy fitness kernel — verifies third-party license
//! identifiers against an allow/deny policy. Per M01-P15-IP-003.
//!
//! I/O-free. Runners parse `Cargo.lock` / SBOM into typed
//! [`CrateLicense`] records and call [`check`].
//!
//! Policy:
//! - `allowed` is an explicit allowlist of SPDX identifiers.
//! - `denied` blocks specific identifiers even if otherwise matched.
//! - `compound_separator` ("OR" / "AND") splits compound SPDX strings;
//!   "OR" passes if any disjunct is allowed and none are denied; "AND"
//!   passes only if every conjunct is allowed.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrateLicense {
    pub crate_name: String,   // data_class: INTERNAL_ONLY
    pub version: String,      // data_class: INTERNAL_ONLY
    pub license_spdx: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LicensePolicy {
    pub allowed: Vec<String>, // data_class: INTERNAL_ONLY
    pub denied: Vec<String>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LicenseViolation {
    pub crate_name: String,             // data_class: INTERNAL_ONLY
    pub version: String,                // data_class: INTERNAL_ONLY
    pub license_spdx: String,           // data_class: INTERNAL_ONLY
    pub reason: LicenseViolationReason, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LicenseViolationReason {
    NotAllowed,
    ExplicitlyDenied { token: String },
    EmptyLicense,
    Unparseable,
}

impl LicenseViolationReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NotAllowed => "license not on allowlist",
            Self::ExplicitlyDenied { .. } => "license token explicitly denied",
            Self::EmptyLicense => "license field empty",
            Self::Unparseable => "license string could not be parsed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LicensePolicyReport {
    pub crates_checked: usize,             // data_class: INTERNAL_ONLY
    pub violations: Vec<LicenseViolation>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LicensePolicyError {
    EmptyPolicy,
    EmptyCrateName,
}

impl LicensePolicyError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyPolicy => "license policy has empty allowlist".to_owned(),
            Self::EmptyCrateName => "crate license record has empty crate_name".to_owned(),
        }
    }
}

/// Split a compound SPDX expression into (operator, tokens). Returns
/// `None` if the expression is unparseable.
fn split_spdx(expr: &str) -> Option<(SpdxOp, Vec<&str>)> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parens_balanced = trimmed.chars().fold(0i32, |acc, c| match c {
        '(' => acc + 1,
        ')' => acc - 1,
        _ => acc,
    }) == 0;
    if !parens_balanced {
        return None;
    }

    // Detect operators at the top level only.
    let has_or = trimmed.contains(" OR ");
    let has_and = trimmed.contains(" AND ");
    if has_or && has_and {
        // Mixed compound — require parens we don't deeply parse; treat as
        // unparseable rather than guess intent.
        return None;
    }
    if has_or {
        Some((SpdxOp::Or, trimmed.split(" OR ").map(str::trim).collect()))
    } else if has_and {
        Some((SpdxOp::And, trimmed.split(" AND ").map(str::trim).collect()))
    } else {
        Some((SpdxOp::Single, vec![trimmed]))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SpdxOp {
    Single,
    Or,
    And,
}

fn token_outcome(token: &str, policy: &LicensePolicy) -> TokenOutcome {
    if let Some(d) = policy.denied.iter().find(|d| d.as_str() == token) {
        return TokenOutcome::Denied(d.clone());
    }
    if policy.allowed.iter().any(|a| a.as_str() == token) {
        TokenOutcome::Allowed
    } else {
        TokenOutcome::NotAllowed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TokenOutcome {
    Allowed,
    Denied(String),
    NotAllowed,
}

pub fn check(
    licenses: &[CrateLicense],
    policy: &LicensePolicy,
) -> Result<LicensePolicyReport, LicensePolicyError> {
    if policy.allowed.is_empty() {
        return Err(LicensePolicyError::EmptyPolicy);
    }

    let mut violations = Vec::new();

    for cl in licenses {
        if cl.crate_name.is_empty() {
            return Err(LicensePolicyError::EmptyCrateName);
        }
        if cl.license_spdx.trim().is_empty() {
            violations.push(LicenseViolation {
                crate_name: cl.crate_name.clone(),
                version: cl.version.clone(),
                license_spdx: cl.license_spdx.clone(),
                reason: LicenseViolationReason::EmptyLicense,
            });
            continue;
        }

        let Some((op, tokens)) = split_spdx(&cl.license_spdx) else {
            violations.push(LicenseViolation {
                crate_name: cl.crate_name.clone(),
                version: cl.version.clone(),
                license_spdx: cl.license_spdx.clone(),
                reason: LicenseViolationReason::Unparseable,
            });
            continue;
        };

        let outcomes: Vec<TokenOutcome> = tokens.iter().map(|t| token_outcome(t, policy)).collect();

        let denied_token = outcomes.iter().find_map(|o| match o {
            TokenOutcome::Denied(t) => Some(t.clone()),
            _ => None,
        });

        let passes = match op {
            SpdxOp::Single => matches!(outcomes[0], TokenOutcome::Allowed),
            SpdxOp::Or => {
                denied_token.is_none()
                    && outcomes.iter().any(|o| matches!(o, TokenOutcome::Allowed))
            }
            SpdxOp::And => outcomes.iter().all(|o| matches!(o, TokenOutcome::Allowed)),
        };

        if !passes {
            let reason = match denied_token {
                Some(tok) => LicenseViolationReason::ExplicitlyDenied { token: tok },
                None => LicenseViolationReason::NotAllowed,
            };
            violations.push(LicenseViolation {
                crate_name: cl.crate_name.clone(),
                version: cl.version.clone(),
                license_spdx: cl.license_spdx.clone(),
                reason,
            });
        }
    }

    Ok(LicensePolicyReport {
        crates_checked: licenses.len(),
        violations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lic(name: &str, version: &str, spdx: &str) -> CrateLicense {
        CrateLicense {
            crate_name: name.into(),
            version: version.into(),
            license_spdx: spdx.into(),
        }
    }

    fn policy() -> LicensePolicy {
        LicensePolicy {
            allowed: vec![
                "Apache-2.0".into(),
                "MIT".into(),
                "BSD-3-Clause".into(),
                "Unicode-3.0".into(),
            ],
            denied: vec!["AGPL-3.0".into(), "SSPL-1.0".into()],
        }
    }

    #[test]
    fn single_allowed_passes() {
        let r = check(&[lic("serde", "1.0", "Apache-2.0")], &policy()).unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn single_not_allowed_flagged() {
        let r = check(&[lic("x", "0.1", "GPL-3.0")], &policy()).unwrap();
        assert_eq!(r.violations.len(), 1);
        assert!(matches!(
            r.violations[0].reason,
            LicenseViolationReason::NotAllowed
        ));
    }

    #[test]
    fn denied_flagged_even_if_in_or() {
        let r = check(&[lic("x", "0.1", "MIT OR AGPL-3.0")], &policy()).unwrap();
        assert_eq!(r.violations.len(), 1);
        assert!(matches!(
            r.violations[0].reason,
            LicenseViolationReason::ExplicitlyDenied { .. }
        ));
    }

    #[test]
    fn or_with_one_allowed_passes() {
        let r = check(&[lic("x", "0.1", "MIT OR Apache-2.0")], &policy()).unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn or_with_no_allowed_flagged() {
        let r = check(&[lic("x", "0.1", "GPL-3.0 OR LGPL-3.0")], &policy()).unwrap();
        assert_eq!(r.violations.len(), 1);
        assert!(matches!(
            r.violations[0].reason,
            LicenseViolationReason::NotAllowed
        ));
    }

    #[test]
    fn and_requires_all_allowed() {
        let r = check(&[lic("x", "0.1", "MIT AND Apache-2.0")], &policy()).unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn and_one_missing_flagged() {
        let r = check(&[lic("x", "0.1", "MIT AND GPL-3.0")], &policy()).unwrap();
        assert_eq!(r.violations.len(), 1);
    }

    #[test]
    fn empty_license_flagged() {
        let r = check(&[lic("x", "0.1", "")], &policy()).unwrap();
        assert_eq!(r.violations.len(), 1);
        assert!(matches!(
            r.violations[0].reason,
            LicenseViolationReason::EmptyLicense
        ));
    }

    #[test]
    fn whitespace_only_license_flagged_as_empty() {
        let r = check(&[lic("x", "0.1", "   ")], &policy()).unwrap();
        assert_eq!(r.violations.len(), 1);
        assert!(matches!(
            r.violations[0].reason,
            LicenseViolationReason::EmptyLicense
        ));
    }

    #[test]
    fn mixed_or_and_unparseable() {
        let r = check(
            &[lic("x", "0.1", "MIT OR Apache-2.0 AND GPL-3.0")],
            &policy(),
        )
        .unwrap();
        assert_eq!(r.violations.len(), 1);
        assert!(matches!(
            r.violations[0].reason,
            LicenseViolationReason::Unparseable
        ));
    }

    #[test]
    fn empty_policy_errors() {
        let p = LicensePolicy {
            allowed: vec![],
            denied: vec![],
        };
        let err = check(&[lic("x", "0.1", "MIT")], &p).unwrap_err();
        assert!(matches!(err, LicensePolicyError::EmptyPolicy));
    }

    #[test]
    fn empty_crate_name_errors() {
        let err = check(&[lic("", "0.1", "MIT")], &policy()).unwrap_err();
        assert!(matches!(err, LicensePolicyError::EmptyCrateName));
    }

    #[test]
    fn multiple_crates_aggregate() {
        let licenses = vec![
            lic("a", "1.0", "Apache-2.0"),
            lic("b", "1.0", "GPL-3.0"),
            lic("c", "1.0", "MIT"),
            lic("d", "1.0", "AGPL-3.0"),
        ];
        let r = check(&licenses, &policy()).unwrap();
        assert_eq!(r.violations.len(), 2);
        assert_eq!(r.crates_checked, 4);
    }

    #[test]
    fn unbalanced_parens_unparseable() {
        let r = check(&[lic("x", "0.1", "(MIT OR Apache-2.0")], &policy()).unwrap();
        assert_eq!(r.violations.len(), 1);
        assert!(matches!(
            r.violations[0].reason,
            LicenseViolationReason::Unparseable
        ));
    }

    #[test]
    fn unicode_license_in_allowlist_passes() {
        let r = check(&[lic("icu", "1.0", "Unicode-3.0")], &policy()).unwrap();
        assert!(r.violations.is_empty());
    }
}
