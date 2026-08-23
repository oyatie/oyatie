//! cost — cost attribution + cost-as-SLI service for oyatie.
//! Single-crate-per-service pattern per ADR-0509.

#![forbid(unsafe_code)]

pub mod allocation;
pub mod attribution;
pub mod config;
pub mod grpc;
pub mod observability;
pub mod rest;
pub mod sli;
pub mod storage;
