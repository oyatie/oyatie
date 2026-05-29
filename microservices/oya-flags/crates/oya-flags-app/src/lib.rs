//! # oya-flags-app
//!
//! Application composition root for the oya-flags OpenFeature server (ADR-0481).
//! Wires kernel [`FlagResolver`] + REST adapter into a runnable service.
//!
//! Binary: `oya-flags`
//! Endpoints:
//!   POST /ofrep/v1/evaluate/flags/{key}
//!   GET  /healthz

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub use oya_flags_kernel::{DefaultFlagResolver, FlagResolver};
pub use oya_flags_rest::{OFREPRequest, OFREPResponse};
