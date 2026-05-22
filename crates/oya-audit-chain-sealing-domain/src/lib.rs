//! Audit-chain sealing domain: Merkle math reference.
//!
//! Wave 15-IMPL-truth-up scaffold (2026-05-21). Wraps the existing
//! `oya_audit_chain_domain::MerkleTree` and re-exports it until IP-007
//! reconciles RFC-6962 domain separation vs. the current length-prefixed
//! scheme described in `policy/seal-integrity.md` SI-01.
#![allow(dead_code)]

pub use oya_audit_chain_domain::{MerkleTree, Sha256Hash};

/// Domain-level seal error variants. Full enum in IP-007.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SealingDomainError {
    EmptyPack,
    EmptyTenantPartition,
    InvalidLeafCount,
    InvalidProofPath,
}
