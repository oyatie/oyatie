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

mod authz;
mod bundle;
mod cache;
mod decision;
mod entity;
mod error;
mod guard;
mod metrics;
mod runtime_config;

pub use authz::*;
pub use bundle::*;
pub use cache::*;
pub use decision::*;
pub use entity::*;
pub use error::*;
pub use guard::*;
pub use metrics::*;
pub(crate) use metrics::{duration_millis_u64, p99_latency_ms};
pub use runtime_config::*;

pub(crate) use std::collections::{BTreeMap, HashMap, VecDeque};
pub(crate) use std::fmt;
pub(crate) use std::panic::{self, AssertUnwindSafe};
pub(crate) use std::sync::atomic::{AtomicU32, Ordering};
pub(crate) use std::sync::{Arc, Mutex};
pub(crate) use std::time::{Duration, Instant};

pub(crate) use serde::{Deserialize, Serialize};

pub(crate) use shared_platform_contracts_kernel::ContractViolation;
pub(crate) use shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, AuthorizationResponse, Decision, EntityRef, Obligation, PolicyVersion,
};
