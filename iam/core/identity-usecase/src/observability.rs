//! Stable OTel event/attribute taxonomy for `identity.token.issue`.
//!
//! This module is pure data mapping over the existing error and status types;
//! issuance logic is unchanged. No runtime dependencies (no `tracing` or
//! `opentelemetry` crates) are introduced — a runtime adapter at the binary
//! boundary is responsible for projecting these value objects into concrete
//! telemetry exporters.
//!
//! # Attribute taxonomy
//!
//! | Attribute        | Value shape          | Notes                              |
//! |------------------|----------------------|------------------------------------|
//! | `surface`        | `&'static str`       | Always `"identity.token.issue"`    |
//! | `outcome`        | [`OutcomeLabel`]     | `"success"` or `"failure"`         |
//! | `error_code`     | `Option<&'static str>`| `None` on success                  |
//! | `purpose`        | `Option<&'static str>`| PascalCase; `None` when unavailable|
//! | `tenant_id_hash` | `u64`                | FNV-1a hash — never raw value      |
//! | `data_class`     | `&'static str`       | Always `"AUDIT"`                   |
//!
//! The `data_class` field is always `"AUDIT"` per `OperationalDataClass::Audit`
//! in `oya-data-boundary-kernel`, marking every event as operational audit data.
//!
//! The `tenant_id_hash` is a low-cardinality FNV-1a 64-bit hash of the raw
//! tenant identifier. The raw value is never stored in the event.

use oya_data_boundary_kernel::parse_purpose_pascal_label;

use crate::{
    IdentityTokenIssueApiError, IdentityTokenIssueApiRequest, IdentityTokenRotationRequest,
};

/// Stable telemetry surface name for identity token issuance.
///
/// Mirrors [`crate::IDENTITY_TOKEN_ISSUE_SURFACE`] so observability consumers
/// can import from a single module without touching the app-boundary public API.
pub const SURFACE: &str = "identity.token.issue";

/// Low-cardinality outcome label for `identity.token.issue` events.
///
/// Designed for use as a Prometheus label or OTel attribute value; the two
/// variants cover every terminal outcome of an issue or rotate call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutcomeLabel {
    /// The issuance or rotation completed successfully.
    Success,
    /// The issuance or rotation was rejected.
    Failure,
}

impl OutcomeLabel {
    /// Stable, low-cardinality string form for telemetry label values.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
        }
    }
}

/// Stable OTel event describing a single `identity.token.issue` call outcome.
///
/// Every field is intentionally low-cardinality:
/// - `surface` and `data_class` are compile-time constants.
/// - `outcome` and `error_code` are `&'static str` labels from the error taxonomy.
/// - `purpose` is an optional `&'static str` label extracted from the request.
/// - `tenant_id_hash` is a FNV-1a hash — the raw tenant identifier is never stored.
///
/// A runtime adapter (e.g. an axum middleware or a tracing subscriber) projects
/// this value object into concrete spans, counters, or log events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityTokenIssueEvent {
    /// Telemetry surface name; always `"identity.token.issue"`.
    pub surface: &'static str, // data_class: INTERNAL_ONLY
    /// Outcome of the issuance call.
    pub outcome: OutcomeLabel, // data_class: INTERNAL_ONLY
    /// Stable error code; `None` on success.
    pub error_code: Option<&'static str>, // data_class: INTERNAL_ONLY
    /// PascalCase purpose label from the request body; `None` when the request
    /// fails validation before a valid purpose can be extracted.
    pub purpose: Option<&'static str>, // data_class: INTERNAL_ONLY
    /// FNV-1a 64-bit hash of the tenant identifier; never the raw value.
    pub tenant_id_hash: u64, // data_class: INTERNAL_ONLY
    /// Operational data class label; always `"AUDIT"`.
    pub data_class: &'static str, // data_class: AUDIT
}

// ── Public constructors ─────────────────────────────────────────────────────

/// Build a success event for `issue_identity_token_from_app`.
pub fn identity_token_issue_event_for_success(
    request: &IdentityTokenIssueApiRequest,
) -> IdentityTokenIssueEvent {
    IdentityTokenIssueEvent {
        surface: SURFACE,
        outcome: OutcomeLabel::Success,
        error_code: None,
        purpose: purpose_label_from_body(&request.body.purpose),
        tenant_id_hash: fnv1a_hash(&request.boundary.tenant_id),
        data_class: AUDIT_LABEL,
    }
}

/// Build a failure event for `issue_identity_token_from_app`.
pub fn identity_token_issue_event_for_error(
    request: &IdentityTokenIssueApiRequest,
    error: &IdentityTokenIssueApiError,
) -> IdentityTokenIssueEvent {
    IdentityTokenIssueEvent {
        surface: SURFACE,
        outcome: OutcomeLabel::Failure,
        error_code: Some(error.code().as_str()),
        purpose: purpose_label_from_body(&request.body.purpose),
        tenant_id_hash: fnv1a_hash(&request.boundary.tenant_id),
        data_class: AUDIT_LABEL,
    }
}

/// Build a success event for `rotate_identity_token_from_app`.
pub fn identity_token_rotate_event_for_success(
    request: &IdentityTokenRotationRequest,
) -> IdentityTokenIssueEvent {
    identity_token_issue_event_for_success(&request.replacement)
}

/// Build a failure event for `rotate_identity_token_from_app`.
pub fn identity_token_rotate_event_for_error(
    request: &IdentityTokenRotationRequest,
    error: &IdentityTokenIssueApiError,
) -> IdentityTokenIssueEvent {
    identity_token_issue_event_for_error(&request.replacement, error)
}

// ── Private helpers ─────────────────────────────────────────────────────────

/// Stable label for the operational audit data class.
///
/// Matches `OperationalDataClass::Audit.label()` from `oya-data-boundary-kernel`
/// without importing the crate at the observability module level.
const AUDIT_LABEL: &str = "AUDIT";

/// Extract a stable PascalCase purpose label from a raw purpose string.
///
/// Returns `None` when the label does not match a supported purpose — this
/// preserves the invariant that `purpose` in the event is always a validated
/// static string rather than arbitrary user input.
fn purpose_label_from_body(purpose: &str) -> Option<&'static str> {
    parse_purpose_pascal_label(purpose).map(|p| p.pascal_label())
}

/// FNV-1a 64-bit hash of an arbitrary string.
///
/// Used to produce a low-cardinality `tenant_id_hash` attribute that can be
/// used for bucketing in metrics without storing the raw tenant identifier in
/// the telemetry event.
fn fnv1a_hash(value: &str) -> u64 {
    let mut state: u64 = 0xcbf29ce484222325;
    for byte in value.bytes() {
        state ^= u64::from(byte);
        state = state.wrapping_mul(0x100000001b3);
    }
    state
}

#[cfg(test)]
mod tests {
    use super::{AUDIT_LABEL, OutcomeLabel, SURFACE, fnv1a_hash, purpose_label_from_body};

    #[test]
    fn surface_constant_value_is_stable() {
        assert_eq!(SURFACE, "identity.token.issue");
    }

    #[test]
    fn audit_label_constant_matches_operational_data_class() {
        assert_eq!(AUDIT_LABEL, "AUDIT");
    }

    #[test]
    fn outcome_label_strings_are_stable() {
        assert_eq!(OutcomeLabel::Success.as_str(), "success");
        assert_eq!(OutcomeLabel::Failure.as_str(), "failure");
    }

    #[test]
    fn fnv1a_hash_is_deterministic() {
        assert_eq!(fnv1a_hash("ten_alpha"), fnv1a_hash("ten_alpha"));
        assert_ne!(fnv1a_hash("ten_alpha"), fnv1a_hash("ten_beta"));
    }

    #[test]
    fn purpose_label_from_body_returns_none_for_unknown_labels() {
        assert_eq!(purpose_label_from_body("not-a-purpose"), None);
        assert_eq!(purpose_label_from_body(""), None);
        assert_eq!(purpose_label_from_body("Banana"), None);
    }

    #[test]
    fn purpose_label_from_body_returns_static_pascal_label_for_known_purposes() {
        assert_eq!(
            purpose_label_from_body("CapabilityInvocation"),
            Some("CapabilityInvocation")
        );
        assert_eq!(purpose_label_from_body("CoreService"), Some("CoreService"));
    }
}
