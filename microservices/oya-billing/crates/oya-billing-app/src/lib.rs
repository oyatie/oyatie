//! Billing composition root (ADR-0083 layer 12, ADR-0478 D4-D5).
//!
//! Wires oya-billing-kernel + oya-billing-rest + PostgreSQL adapter +
//! Pulsar event bus into a runnable µservice process.
//!
//! ## Honest-claims note
//!
//! non_claim: PostgreSQL and Pulsar wiring deferred to ADR-0478 D4-D5.
//! This crate is a scaffold stub only.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

// TODO: implement per ADR-0478 D1-D5
