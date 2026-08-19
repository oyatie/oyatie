//! Foundry-owned live workload harness for FD-001 backbone microservices.
//!
//! The crate intentionally keeps the harness in integration tests so it can
//! compose app/usecase/adapter seams without teaching an individual adapter
//! crate about peer adapters.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub const BACKBONE_WORKLOAD_LIVE_HARNESS: &str = "backbone-workload-live";
