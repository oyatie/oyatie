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
