//! Billing kernel — pure subscription/plan/invoice state machines (ADR-0478 D1-D2).
//!
//! No I/O, no async. Owns [`Subscription`], [`Plan`], and [`Invoice`] aggregate
//! roots plus their state-machine transitions.
//!
//! ## Honest-claims note
//!
//! non_claim: state machine logic is deferred to ADR-0478 D1-D5 implementation
//! phases. This crate is a scaffold stub only.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

// TODO: implement per ADR-0478 D1-D5
