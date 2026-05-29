//! oya-identity — identity service for oyatie.
//!
//! Single-crate-per-service pattern per ADR-0509.
//! Subsystems: auth, oidc, oauth2, webauthn, realms, users, storage, rest, grpc, observability.

#![forbid(unsafe_code)]

pub mod auth;
pub mod config;
pub mod grpc;
pub mod oauth2;
pub mod observability;
pub mod oidc;
pub mod realms;
pub mod rest;
pub mod storage;
pub mod users;
pub mod webauthn;
