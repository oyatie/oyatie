//! Cost-allocation kernel — pure usage aggregation, allocation rules, and
//! chargeback value objects (ADR-0480 D1-D2).
//!
//! No I/O, no async. Owns cost-allocation aggregate roots plus their
//! state-machine transitions.
//!
//! ## Honest-claims note
//!
//! non_claim: state machine logic is deferred to ADR-0480 D1-D5 implementation
//! phases. This crate is a scaffold stub only.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

// TODO: implement per ADR-0480 D1-D5
