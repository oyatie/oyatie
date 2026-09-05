//! # shared-pdp-kernel
//!
//! Embedded-PDP port kernel for FD-001 (story G004, ADR-0536 D-2).
//!
//! ## Posture
//! ADR-0536 D-2: the PDP is embedded in-process in every service — an
//! authorization decision never takes a network hop — and a central policy
//! store compiles, signs, and pushes content-addressed policy bundles to
//! every PDP. Precedent: Cedar / Amazon Verified Permissions (embedded,
//! formally verified evaluator + central policy store); Google Zanzibar
//! (zookie freshness tokens; isolation is structural, not conventional).
//!
//! This crate is the vendor-neutral PORT: the [`PolicyDecisionPoint`] trait
//! over the locked PDP contract family in
//! `shared-platform-contracts-kernel::pdp`, plus the value types every
//! engine adapter consumes — [`PolicyBundle`] (version-bearing policy bundle
//! as pushed by the policy store), [`EntitySlice`] (the PIP entity slice a
//! PEP assembles per request), [`DecisionCache`] keyed on
//! `(request-fingerprint, policy-version)` per the G004 acceptance shape,
//! and [`DecisionAuditRecord`] (audit record per decision — every decision,
//! allow or deny, cached or evaluated, is attributable).
//!
//! Ports-for-owned-stack review ("would this trait change at W5 cutover?"):
//! no — Cedar is the TERMINAL engine decision per ADR-0536 D-2 (formally
//! verified upstream crate), and this port models the destination decision
//! surface (PARC request in, attributable decision + audit record out),
//! not any transient engine detail.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

mod bundle;
mod cache;
mod decision_authz;
mod entity;
mod error;
mod port;
mod runtime;

pub use bundle::{PolicyBundle, TemplateLink, TemplateSrc};
pub use cache::{CachedDecision, DecisionCache, DecisionCacheKey, request_fingerprint};
pub use decision_authz::{
    DecisionAuthorizer, DecisionAuthzError, DecisionAuthzRequest, FailClosedDecisionAuthorizer,
};
pub use entity::{EntityRecord, EntitySlice};
pub use error::PdpError;
pub use port::{DecisionAuditRecord, PdpOutcome, PolicyDecisionPoint};
pub use runtime::{
    PdpCircuitState, PdpRuntimeConfig, PdpRuntimeGuard, PdpRuntimeMetrics,
    PdpRuntimeMetricsSnapshot,
};

#[cfg(test)]
mod tests;
