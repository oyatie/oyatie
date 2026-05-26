//! # oya-ci-webhook-gateway-app
//!
//! The CI webhook gateway: the FIRST hop of the gated change-coordination
//! pipeline per ADR-0363 (git + Jenkins + self-hosted Forgejo), ADR-0366
//! (self-enforcing pipeline), and ADR-0367 (trustless pre-merge verification).
//!
//! ## What it does
//!
//! 1. Receives Forgejo webhook deliveries at `/webhook/forgejo`.
//! 2. Verifies the `X-Hub-Signature-256` HMAC on the RAW body, fail-closed,
//!    BEFORE any parsing/routing (so unsigned traffic cannot poison state).
//! 3. Parses `pull_request` events (opened / reopened / synchronized) whose
//!    base branch is the gated target (default `dev`).
//! 4. Dispatches the gated pipeline by kicking the Jenkins `oyaCiLane`
//!    pipeline (admission → `oya gate run-all`, the trusted-runner
//!    re-execution that posts the Forgejo commit statuses).
//!
//! ## Why it exists
//!
//! `dev` branch protection requires 15 status contexts. Jenkins already POSTs
//! 14 of them to the Forgejo Commit Status API (`oyaCiLane.groovy`), but
//! nothing TRIGGERS Jenkins from a Forgejo PR event — so historically every
//! merge briefly disabled `enforce_admins` and used an admin-merge. This
//! gateway is the missing trigger: it turns a PR event into a real, gated CI
//! run, retiring the manual admin-relax-merge seam.
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

pub use config::GatewayConfig;
pub use dispatch::{DispatchReceipt, JenkinsDispatcher, PipelineDispatcher, PipelineKickoff};
pub use error::{GatewayError, PipelineStage, Result};
pub use event::{PrAction, PullRequestEvent, RouteOutcome};
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

/// The branch-protection required status contexts the downstream Jenkins lane
/// produces (kept in sync with `infra/ci/jenkins/reported-status-contexts.json`
/// by the `oya-governance-protection-context-match` gate). The gateway does
/// not post these itself — it kicks the pipeline that does — but it knows the
/// set so it can report the boundary.
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
    fn required_contexts_match_branch_protection_count() {
        // dev.json lists 15 required contexts.
        assert_eq!(REQUIRED_STATUS_CONTEXTS.len(), 15);
        assert!(REQUIRED_STATUS_CONTEXTS.contains(&"oya-pr-review"));
    }
}
