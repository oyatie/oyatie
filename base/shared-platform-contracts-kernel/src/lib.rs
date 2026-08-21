//! # oya-shared-platform-contracts-kernel
//!
//! FD-001 shared platform contract models — the contract-lock seed for the
//! tenancy/RBAC microservice core (`FD-001-tenancy-rbac-microservice-core`).
//!
//! ## Posture (Smithy-architecture seed)
//! AWS models every service API as typed, protocol-agnostic shapes first
//! (Smithy), then derives wire bindings from the locked model. This crate is
//! that seed reimplemented as plain Rust per the owned-Rust-stack directive:
//! pure `serde` types + explicit invariants, NO handlers, NO IO, NO transport
//! coupling. The masterplan rule `api_first_contracts_must_exist_before_handlers`
//! makes this crate the lock that parallel service lanes build against.
//!
//! ## Contract families
//! - [`identity`] — principal, credential, token claims, identity domain
//!   (precedent: AWS IAM / Google Cloud IAM identity models, SPIFFE workload
//!   identity, RFC 7519/RFC 9068 token claims).
//! - [`pdp`] — authorization request/response with decision id and an opaque
//!   policy-version freshness token (precedent: Google Zanzibar "zookie"
//!   consistency tokens; Cedar/AVP policy-store version pinning).
//! - [`tenancy`] — tenant resource, lifecycle states, isolation posture
//!   (precedent: AWS SaaS Well-Architected silo/pool/bridge isolation models,
//!   cell-based architecture).
//! - [`shell_bff`] — capability registry entry + module route registration for
//!   the app-shell backend-for-frontend (precedent: SoundCloud/Netflix BFF,
//!   micro-frontend route registries).
//!
//! ## Cedar seed
//! The `cedar/` directory carries the FD-001 entity-type schema (Tenant,
//! Principal, WorkloadIdentity, TenantResource), the structural cell/tenant
//! isolation `forbid` invariant, and example RBAC/ABAC/PBAC policies. The
//! integration tests validate all of it against the real `cedar-policy` engine
//! (dev-dependency only; the production surface of this crate stays Cedar-free
//! per ADR-0183 policy-engine separation).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

pub mod identity;
pub mod pdp;
pub mod shell_bff;
pub mod tenancy;

/// A single contract-invariant violation. Validation is surface-all: every
/// `validate()` in this crate returns the FULL violation set, never just the
/// first failure (matching the repo gate style).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContractViolation {
    /// A required field is empty or absent.
    MissingValue { field: &'static str },
    /// A field exceeds its maximum length.
    TooLong {
        field: &'static str,
        max: usize,
        actual: usize,
    },
    /// A field contains characters outside its allowed charset.
    InvalidCharset { field: &'static str, value: String },
    /// A temporal invariant is violated (e.g. expiry not after issuance).
    InvalidTemporalOrder { field: &'static str, detail: String },
    /// A lifecycle transition that the state machine forbids.
    InvalidTransition {
        from: &'static str,
        operation: &'static str,
    },
    /// A structural/shape invariant violation not covered by the above.
    InvalidShape { field: &'static str, detail: String },
    /// A cross-record referential invariant violation (duplicate id, dangling
    /// reference, ambiguous route, cross-tenant/cell mismatch).
    BrokenReference { field: &'static str, detail: String },
}

impl fmt::Display for ContractViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingValue { field } => write!(f, "{field}: required value is missing"),
            Self::TooLong { field, max, actual } => {
                write!(f, "{field}: length {actual} exceeds maximum {max}")
            }
            Self::InvalidCharset { field, value } => {
                write!(f, "{field}: value {value:?} contains disallowed characters")
            }
            Self::InvalidTemporalOrder { field, detail } => {
                write!(f, "{field}: temporal order violated ({detail})")
            }
            Self::InvalidTransition { from, operation } => {
                write!(
                    f,
                    "lifecycle: operation {operation} is not allowed from state {from}"
                )
            }
            Self::InvalidShape { field, detail } => write!(f, "{field}: {detail}"),
            Self::BrokenReference { field, detail } => write!(f, "{field}: {detail}"),
        }
    }
}

impl std::error::Error for ContractViolation {}

/// Maximum length for slug-form identifiers across all contract families.
pub const MAX_ID_LEN: usize = 128;
/// Maximum length for human-readable display names.
pub const MAX_DISPLAY_NAME_LEN: usize = 256;
/// Maximum length for opaque tokens (policy versions, page tokens, refs).
pub const MAX_OPAQUE_TOKEN_LEN: usize = 512;

fn is_slug_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '.' | '_')
}

/// Surface-all slug check: non-empty, starts with `[a-z0-9]`, charset
/// `[a-z0-9._-]`, bounded length. Shared by every id-bearing contract field.
pub(crate) fn check_slug(
    field: &'static str,
    value: &str,
    max: usize,
    out: &mut Vec<ContractViolation>,
) {
    if value.is_empty() {
        out.push(ContractViolation::MissingValue { field });
        return;
    }
    if value.len() > max {
        out.push(ContractViolation::TooLong {
            field,
            max,
            actual: value.len(),
        });
    }
    let first_ok = value
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    if !first_ok || !value.chars().all(is_slug_char) {
        out.push(ContractViolation::InvalidCharset {
            field,
            value: value.to_owned(),
        });
    }
}

/// Surface-all check for non-empty bounded free text (display names etc.).
pub(crate) fn check_text(
    field: &'static str,
    value: &str,
    max: usize,
    out: &mut Vec<ContractViolation>,
) {
    if value.trim().is_empty() {
        out.push(ContractViolation::MissingValue { field });
        return;
    }
    if value.len() > max {
        out.push(ContractViolation::TooLong {
            field,
            max,
            actual: value.len(),
        });
    }
}

/// Surface-all check for an opaque non-empty bounded token.
pub(crate) fn check_opaque_token(
    field: &'static str,
    value: &str,
    out: &mut Vec<ContractViolation>,
) {
    if value.is_empty() {
        out.push(ContractViolation::MissingValue { field });
        return;
    }
    if value.len() > MAX_OPAQUE_TOKEN_LEN {
        out.push(ContractViolation::TooLong {
            field,
            max: MAX_OPAQUE_TOKEN_LEN,
            actual: value.len(),
        });
    }
    if value.chars().any(char::is_whitespace) {
        out.push(ContractViolation::InvalidCharset {
            field,
            value: value.to_owned(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_accepts_canonical_ids() {
        for ok in ["acme", "tenant-1", "a.b_c-d", "0abc"] {
            let mut out = Vec::new();
            check_slug("field", ok, MAX_ID_LEN, &mut out);
            assert!(out.is_empty(), "{ok}: {out:?}");
        }
    }

    #[test]
    fn slug_rejects_empty_uppercase_and_bad_first_char() {
        for bad in ["", "Acme", "-leading-dash", "has space", "uni\u{e9}"] {
            let mut out = Vec::new();
            check_slug("field", bad, MAX_ID_LEN, &mut out);
            assert!(!out.is_empty(), "{bad}: expected violation");
        }
    }

    #[test]
    fn slug_surfaces_length_and_charset_together() {
        let long_upper = "A".repeat(MAX_ID_LEN + 1);
        let mut out = Vec::new();
        check_slug("field", &long_upper, MAX_ID_LEN, &mut out);
        assert_eq!(
            out.len(),
            2,
            "surface-all must report both violations: {out:?}"
        );
    }

    #[test]
    fn opaque_token_rejects_whitespace_and_empty() {
        let mut out = Vec::new();
        check_opaque_token("token", "", &mut out);
        assert_eq!(
            out,
            vec![ContractViolation::MissingValue { field: "token" }]
        );
        out.clear();
        check_opaque_token("token", "a b", &mut out);
        assert!(matches!(
            out.as_slice(),
            [ContractViolation::InvalidCharset { .. }]
        ));
    }

    #[test]
    fn violations_render_human_legible_messages() {
        let v = ContractViolation::TooLong {
            field: "tenant_id",
            max: 8,
            actual: 9,
        };
        assert_eq!(v.to_string(), "tenant_id: length 9 exceeds maximum 8");
    }
}
