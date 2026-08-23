//! # iam-pdp-app
//!
//! The runnable iam policy-decision-point service (ADR-0559, G004
//! slice 1).
//!
//! ## Posture
//! iam IS the IdP substrate, and the Cedar PDP + policy-bundle
//! distribution live here (three-plane identity doctrine; ADR-0536 D-2).
//! This app is the composition root: it loads ONE declarative policy bundle
//! through the [`iam_pdp_kernel::PolicyBundleStore`] port
//! (file/ConfigMap transport in slice 1), compiles it into the shared
//! embedded Cedar engine (`iam/adapters/pdp-cedar` — the single
//! decision algorithm, ADR-0243), and serves authorization decisions over
//! gRPC + REST with health/readiness and one attributable audit record per
//! decision.
//!
//! ## Doctrine bindings
//! - **Default-deny everywhere**: Cedar denies absent a permit; unknown
//!   routes 404; refusals are NEVER allows. RBAC + ABAC + PBAC are all
//!   expressible (Cedar natively; the API carries PARC + entity slice +
//!   context, never an RBAC-only shape).
//! - **Fail-closed boot**: a policy bundle that cannot load REFUSES the boot
//!   (the identity precedent). A serving process is a
//!   correctly-configured process.
//! - **PDP, not PEP**: decisions are deterministic + side-effect-free per
//!   request — same request + same bundle ⇒ same decision content. The only
//!   emission is the audit record, which never affects the decision.
//! - **API-only service** (cli_surface_policy): no CLI surface; declarative
//!   policy bundles are the management surface (ConfigMap in slice 1, the
//!   policy-bundle CRD + operator distribution as destination).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::sync::Arc;

use iam_pdp_cedar::CedarPdp;
use iam_pdp_kernel::DecisionAuditSink;
use shared_pdp_kernel::{EntitySlice, PdpError, PolicyDecisionPoint};
use shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, AuthorizationResponse, PolicyVersion,
};

pub mod audit;
pub mod client_cert_verifier;
pub mod grpc;
pub mod idgen;
pub mod mtls;
pub mod mtls_transport;
pub mod observability;
pub mod rest;
pub mod server;

pub use iam_pdp_kernel::{
    ENV_BUNDLE_PATH, ENV_DECISION_CACHE_CAPACITY, ENV_GRPC_ADDR, ENV_REST_ADDR, PdpConfig,
};

/// Shared service state: the embedded Cedar PDP plus the audit-emission
/// port. BOTH delivery surfaces (REST + gRPC) decide through [`PdpState::decide`],
/// so the two protocols can never drift and the audit-per-decision invariant
/// holds at exactly one place.
pub struct PdpState {
    pdp: CedarPdp,
    audit: Arc<dyn DecisionAuditSink>,
}

impl PdpState {
    /// Assemble the state from a loaded PDP and an audit sink.
    #[must_use]
    pub fn new(pdp: CedarPdp, audit: Arc<dyn DecisionAuditSink>) -> Self {
        Self { pdp, audit }
    }

    /// The policy-store version token of the currently serving bundle
    /// (readiness surface + zookie echo).
    #[must_use]
    pub fn loaded_policy_version(&self) -> PolicyVersion {
        self.pdp.loaded_policy_version()
    }

    /// Decide one PARC request against the supplied entity slice and emit
    /// the audit record. Every [`PdpError`] is a REFUSAL, not a decision —
    /// callers map it to a non-success protocol status and PEPs MUST treat
    /// it as deny (fail-closed).
    ///
    /// # Errors
    /// [`PdpError`] when the PDP refuses to decide (invalid request, stale
    /// zookie pin, unknown action, evaluation failure).
    pub fn decide(
        &self,
        request: &AuthorizationRequest,
        entities: &EntitySlice,
    ) -> Result<AuthorizationResponse, PdpError> {
        let outcome = self.pdp.authorize(request, entities)?;
        // Audit emission is best-effort by PORT CONTRACT (a sink failure
        // never surfaces as an allow or a refusal); the sink itself owns
        // swallowing its errors.
        self.audit.record(&outcome.audit);
        Ok(outcome.response)
    }

    /// Structured refusal log line (one per refused request, so unauthorized
    /// probing is visible even though no decision record exists — refusals
    /// are not decisions and never enter the decision-audit chain).
    pub(crate) fn log_refusal(request_id: &str, error: &PdpError) {
        tracing::warn!(
            target: "cloud_iam_pdp::refusal",
            request_id,
            error = %error,
            "authorization request refused (fail-closed: PEPs must treat as deny)",
        );
    }
}
