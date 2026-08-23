//! flags — feature flag service for oyatie (OpenFeature-compatible).
//!
//! Single-crate-per-service pattern per ADR-0509.

#![forbid(unsafe_code)]

pub mod config;
pub mod evaluation;
pub mod grpc;
pub mod observability;
pub mod ofrep;
pub mod rest;
pub mod storage;
pub mod targeting;
pub mod tenants;
