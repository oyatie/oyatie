//! Typed error surface for the CI webhook gateway (ADR-0083 tier-1 strict
//! error handling: no `unwrap`/`expect`/`panic` outside `cfg(test)`).
//!
//! The `GatewayError::Unimplemented` variant is the EXPLICIT typed boundary
//! for downstream pipeline stages that the substrate has not yet stood up
//! (e.g. the Intelligence-service reviewer gate, the merge-queue). It is NOT
//! a silent stub: the variant carries the stage name, returns a distinct HTTP
//! status (501), and every occurrence is recorded in
//! `registry/placeholder-debt/`. This keeps the honest-claims gate green.

use std::fmt;

/// HTTP-mappable result type for the gateway.
pub type Result<T> = std::result::Result<T, GatewayError>;

/// Pipeline stage identifiers, used by both the dispatcher and the
/// `Unimplemented` boundary so deferred work is named, not anonymous.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PipelineStage {
    /// Repo-entry admission (the wired `oya-vcs-admission` governance check).
    Admission,
    /// Historical local CI mirror gate: `oya gate run-all` bridge replay.
    /// Current merge authority is cloud-ci `oya-ci-required`; the gateway only
    /// dispatches the deprecated bridge.
    GateRunAll,
    /// Narrow board projection snapshot path. This is intentionally not
    /// the legacy local replay; label/claim-ref webhooks must not trigger CI.
    BoardProjection,
    /// Adversarial reviewer gate — a CI stage powered by the Intelligence
    /// service (ADR-0367 D2). Distinct identity from the author.
    ReviewerGate,
    /// Merge-queue admission (ADR-0111 speculative rebase; parked per
    /// ADR-0363 §3 until concurrent-PR volume justifies it).
    MergeQueue,
}

impl PipelineStage {
    /// Stable kebab-case identifier (matches the placeholder-debt token and
    /// the GitHub commit-status context where one is posted).
    pub const fn id(self) -> &'static str {
        match self {
            PipelineStage::Admission => "oya-vcs-admission",
            PipelineStage::GateRunAll => "oya-gate-run-all",
            PipelineStage::BoardProjection => "oya-board-projection",
            PipelineStage::ReviewerGate => "oya-pr-review",
            PipelineStage::MergeQueue => "merge-queue-admit",
        }
    }
}

impl fmt::Display for PipelineStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

#[derive(Debug)]
pub enum GatewayError {
    /// The `X-Hub-Signature-256` (GitHub HMAC) header was absent. Fail closed.
    MissingSignature,
    /// The HMAC signature header was malformed (not `sha256=<hex>`). Fail closed.
    MalformedSignature,
    /// HMAC verification failed: the computed digest did not match the header
    /// under a constant-time comparison. Fail closed BEFORE any dedup/routing.
    SignatureMismatch,
    /// The webhook secret is not configured (no `sref` resolved). The gateway
    /// refuses to start verification it cannot perform — fail closed.
    SecretUnavailable,
    /// The webhook payload was not valid JSON or lacked required fields.
    MalformedPayload(String),
    /// The `(event, action)` pair is not in the gateway's closed router table
    /// (e.g. a `wiki` event). Logged + rejected, not silently dropped.
    UnroutableEvent { event: String, action: String },
    /// A downstream pipeline stage is not yet built in the substrate. Carries
    /// the named stage; mapped to HTTP 501. Tracked in placeholder-debt.
    Unimplemented {
        stage: PipelineStage,
        debt_token: &'static str,
    },
    /// A configured downstream (e.g. the Jenkins dispatch endpoint) returned a
    /// transport-level failure. Carries a human-readable cause.
    DispatchTransport(String),
}

impl GatewayError {
    /// The HTTP status this error maps to on the receiver surface.
    pub const fn http_status(&self) -> u16 {
        match self {
            // Signature failures are 401 — the caller is unauthenticated.
            GatewayError::MissingSignature
            | GatewayError::MalformedSignature
            | GatewayError::SignatureMismatch => 401,
            // A secret we cannot load is our fault, not the caller's.
            GatewayError::SecretUnavailable => 503,
            GatewayError::MalformedPayload(_) => 400,
            // Unroutable but authentic: accept-and-ignore semantics use 422 so
            // GitHub does not infinitely redeliver an event we will never act on.
            GatewayError::UnroutableEvent { .. } => 422,
            // Explicit not-built boundary.
            GatewayError::Unimplemented { .. } => 501,
            GatewayError::DispatchTransport(_) => 502,
        }
    }
}

impl fmt::Display for GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GatewayError::MissingSignature => {
                f.write_str("webhook rejected: missing X-Hub-Signature-256 header (fail-closed)")
            }
            GatewayError::MalformedSignature => {
                f.write_str("webhook rejected: malformed signature header (expected sha256=<hex>)")
            }
            GatewayError::SignatureMismatch => {
                f.write_str("webhook rejected: HMAC signature mismatch (fail-closed)")
            }
            GatewayError::SecretUnavailable => {
                f.write_str("webhook secret unavailable: refusing to verify (fail-closed)")
            }
            GatewayError::MalformedPayload(why) => write!(f, "malformed webhook payload: {why}"),
            GatewayError::UnroutableEvent { event, action } => {
                write!(
                    f,
                    "unroutable event: ({event}, {action}) not in router table"
                )
            }
            GatewayError::Unimplemented { stage, debt_token } => write!(
                f,
                "pipeline stage {stage} is not yet built in this substrate \
                 (tracked: registry/placeholder-debt/adr-follow-ups.yaml#{debt_token})"
            ),
            GatewayError::DispatchTransport(why) => write!(f, "dispatch transport failure: {why}"),
        }
    }
}

impl std::error::Error for GatewayError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_failures_map_to_401() {
        assert_eq!(GatewayError::MissingSignature.http_status(), 401);
        assert_eq!(GatewayError::MalformedSignature.http_status(), 401);
        assert_eq!(GatewayError::SignatureMismatch.http_status(), 401);
    }

    #[test]
    fn unimplemented_maps_to_501_and_names_the_stage() {
        let err = GatewayError::Unimplemented {
            stage: PipelineStage::ReviewerGate,
            debt_token: "adr-0374-reviewer-gate-dispatch",
        };
        assert_eq!(err.http_status(), 501);
        assert!(err.to_string().contains("oya-pr-review"));
        assert!(err.to_string().contains("placeholder-debt"));
    }

    #[test]
    fn stage_ids_are_stable() {
        assert_eq!(PipelineStage::Admission.id(), "oya-vcs-admission");
        assert_eq!(PipelineStage::BoardProjection.id(), "oya-board-projection");
        assert_eq!(PipelineStage::ReviewerGate.id(), "oya-pr-review");
    }
}
