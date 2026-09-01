//! Foundry ontology facade: the vertical's first real process.
//!
//! It composes what the governed spine already proves — the durable action
//! log and its separate denial trail, the registry the writer stamps
//! against, and the projector fold — behind an HTTP surface. This lane
//! serves liveness, readiness and metrics; the write, read and migration
//! surfaces land in their own lanes, and every operator surface refuses
//! until a policy decision point is composed.
//!
//! Boot is fail-closed throughout: an unopenable durable store, an aliased
//! pair of logs, an empty tenant roster or a registry the kernel refuses all
//! stop the process before it serves anything.
#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod authz;
pub mod composition;
pub mod config;
pub mod metrics;
pub mod observability;
pub mod pdp;
pub mod routes;
pub mod seed;

pub use authz::{Caller, PolicyEnforcementPoint};
pub use composition::{AppState, BootError, TenantState, compose};
pub use config::{Config, ConfigError};
pub use pdp::{PepError, Surface};
pub use routes::router;
pub use seed::SeedError;
