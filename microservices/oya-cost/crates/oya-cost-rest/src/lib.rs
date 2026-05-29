//! Cost-allocation REST surface — axum HTTP + Connect-RPC handlers (ADR-0480 D3).
//!
//! Exposes cost-allocation operations over HTTP. Delegates all domain logic to
//! `oya-cost-kernel`; performs no persistence or messaging I/O.
//!
//! ## Honest-claims note
//!
//! non_claim: route mounting and Connect-RPC codegen deferred to ADR-0480 D3.
//! This crate is a scaffold stub only.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

// TODO: implement per ADR-0480 D1-D5
