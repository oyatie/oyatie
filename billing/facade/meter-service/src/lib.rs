//! meter — usage metering service for oyatie.
//!
//! Single-crate-per-service pattern per ADR-0509.

#![forbid(unsafe_code)]

pub mod aggregation;
pub mod config;
pub mod grpc;
pub mod ingest;
pub mod observability;
pub mod pulsar;
pub mod quota;
pub mod rest;
pub mod storage;
