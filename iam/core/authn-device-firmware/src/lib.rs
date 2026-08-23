//! authn-device-firmware — authenticator-side WebAuthn firmware stub.
//!
//! Phase-1 reference per ADR-0508: wraps Google's OpenSK (vendored at
//! `tools/opensk-vendored/`). Phase-2+ replaces OpenSK with bespoke
//! Rust fork carrying oyatie attestation root CA.
//!
//! Single-crate pattern per ADR-0509.

#![forbid(unsafe_code)]

pub mod attestation;
pub mod config;
pub mod ctap2;
pub mod observability;
pub mod storage;
pub mod transport;
