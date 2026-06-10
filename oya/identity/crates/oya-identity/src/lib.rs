//! oya-identity — identity service for oyatie.
//!
//! Single-crate-per-service pattern per ADR-0509.
//! Subsystems: auth, oidc, oauth2, realms, users, storage, rest, grpc, observability.
//! (Passkeys/WebAuthn returns behind a port in its own sub-slice — the
//! webauthn-rs -> openssl chain is buck2-unbuildable on current runners,
//! see the friction ledger.)

#![forbid(unsafe_code)]

pub mod auth;
pub mod config;
pub mod grpc;
pub mod oauth2;
pub mod observability;
pub mod oidc;
pub mod realms;
pub mod rest;
pub mod server;
pub mod storage;
pub mod users;

use oya_identity_workload_app::{InMemoryRevocationDenylist, InMemoryWorkloadPrincipalRepository};
use oya_identity_workload_authz_cedar_adapter::CedarWorkloadAuthorizer;
use oya_identity_workload_rest::WorkloadAuthzState;

use crate::observability::TracingAuditSink;

/// The composed application state: in-memory bring-up stores behind the
/// repository/denylist ports (G03 swaps the durable store in behind the same
/// ports), the embedded Cedar PDP, the static JWKS, and the tracing audit sink.
pub type AppState = WorkloadAuthzState<
    InMemoryWorkloadPrincipalRepository,
    InMemoryRevocationDenylist,
    CedarWorkloadAuthorizer,
    TracingAuditSink,
>;
