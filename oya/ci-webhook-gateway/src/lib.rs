//! # oya-ci-webhook-gateway-app
//!
//! The CI webhook gateway: the FIRST hop of the gated change-coordination
//! pipeline per ADR-0363 (git + Jenkins + GitHub (interim)), ADR-0366
//! (self-enforcing pipeline), and ADR-0367 (trustless pre-merge verification).
//!
//! ## What it does
//!
//! 1. Receives GitHub webhook deliveries at `/webhook/github`.
//! 2. Verifies the `X-Hub-Signature-256` HMAC on the RAW body, fail-closed,
//!    BEFORE any parsing/routing (so unsigned traffic cannot poison state).
//! 3. Parses `pull_request` events (opened / reopened / synchronized) whose
//!    base branch is the gated target (default `dev`).
//! 4. Dispatches the historical Jenkins `oyaCiLane` bridge for provenance
//!    and local replay only. Protected-branch authority is the cloud-ci
//!    Rust gate packet that posts the single `oya-ci-required` context.
//!
//! ## Why it exists
//!
//! Earlier bridge-era `dev` branch protection expected many Jenkins-produced
//! contexts. This gateway remains as a historical/provenance trigger for that
//! lane; it does not define current merge authority, which lives behind the
//! protected `oya-ci-required` context.
//!
//! ## Honest boundaries
//!
//! The adversarial reviewer gate (ADR-0367 D2, powered by the Intelligence
//! service) and the speculative merge-queue (ADR-0111, parked per ADR-0363 §3)
//! are NOT yet stood up. They are expressed as the typed
//! [`error::GatewayError::Unimplemented`] boundary (HTTP 501) and tracked in
//! `registry/placeholder-debt/adr-follow-ups.yaml` — no lying stub.
//!
//! ## Layering (clean architecture)
//!
//! - [`signature`] / [`event`] — pure-domain (no IO): HMAC verify + routing.
//! - [`dispatch`] — the port trait + the Jenkins-backed adapter.
//! - [`receiver`] — the axum HTTP boundary, depends only on the port.
//! - [`config`] — env-resolved configuration + secret resolution.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod config;
pub mod dispatch;
pub mod error;
pub mod event;
pub mod receiver;
pub mod signature;

pub use config::{DispatcherKind, GatewayConfig};
pub use dispatch::{
    ControllerDispatcher, DispatchReceipt, DispatchSubject, GateRunBody, JenkinsDispatcher,
    PipelineDispatcher, PipelineKickoff,
};
pub use error::{GatewayError, PipelineStage, Result};
pub use event::{
    CiEvent, IssueAction, IssueSnapshotEvent, PrAction, PullRequestEvent, PushSnapshotEvent,
    RouteOutcome,
};
pub use receiver::{HEALTHZ_PATH, ReceiverState, WEBHOOK_PATH, router};
pub use signature::WebhookSecret;

/// Microservice identity constants (mirrors the `crm` scaffold convention).
pub const MICROSERVICE: &str = "ci-webhook-gateway";
pub const SERVICE_TITLE: &str = "CI Webhook Gateway";
pub const PACKAGE_NAME: &str = "oya-ci-webhook-gateway-app";
pub const BOUNDED_CONTEXT: &str = "change-coordination-substrate";
pub const OWNER_TEAM: &str = "council-architecture + ops-platform";
pub const PRIMARY_DESIGN_ADR: &str = "ADR-0374";
pub const SUBSTRATE_ADR: &str = "ADR-0363";

/// Historical Jenkins bridge status contexts. Current branch-protection merge
/// authority is the single cloud-ci `oya-ci-required` context; this list is
/// retained only so the deprecated gateway can report its bridge boundary.
pub const REQUIRED_STATUS_CONTEXTS: &[&str] = &[
    "cargo-fmt",
    "cargo-check",
    "cargo-clippy",
    "cargo-nextest",
    "oya-vcs-admission",
    "oya-vcs-provider-execution",
    "oya-governance-supply-chain",
    "oya-governance-cohesion",
    "oya-governance-api-semver",
    "oya-governance-honest-claims",
    "oya-governance-aspirational-enforcement",
    "oya-governance-banned-primitives",
    "oya-governance-protection-context-match",
    "oya-governance-dependency-seam",
    "oya-pr-review",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_constants_are_stable() {
        assert_eq!(MICROSERVICE, "ci-webhook-gateway");
        assert_eq!(PACKAGE_NAME, "oya-ci-webhook-gateway-app");
        assert_eq!(PRIMARY_DESIGN_ADR, "ADR-0374");
    }

    #[test]
    fn historical_bridge_contexts_remain_enumerated_for_boundary_reporting() {
        // The retired Jenkins bridge produced 15 historical contexts; current
        // branch-protection authority remains the single `oya-ci-required` context.
        assert_eq!(REQUIRED_STATUS_CONTEXTS.len(), 15);
        assert!(REQUIRED_STATUS_CONTEXTS.contains(&"oya-pr-review"));
    }
}
