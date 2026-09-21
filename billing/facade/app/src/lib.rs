//! billing — billing service for oyatie.
//!
//! Single-crate-per-service pattern per ADR-0509.

#![forbid(unsafe_code)]

pub mod config;
pub mod grpc;
pub mod invoicing;
pub mod money;
pub mod observability;
pub mod payment;
pub mod pricing;
pub mod rest;
pub mod storage;
pub mod subscription;
