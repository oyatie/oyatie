//! Isolation policy kernel — RLS YAML manifest + tenant-bound-table registry + JWT ports.
//!
//! Wave 15-IMPL-truth-up scaffold; full implementation lands in IP-006 / IP-007 execution.
//! Per ADR-0105 the kernel layer is pure types: zero I/O, zero business logic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(dead_code)]

/// A tenant-bound table that MUST have RLS FORCE applied.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TenantBoundTable {
    pub schema: String,        // data_class: INTERNAL_ONLY
    pub table: String,         // data_class: INTERNAL_ONLY
    pub tenant_column: String, // data_class: INTERNAL_ONLY
}

/// Row-level-security policy specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RlsPolicy {
    pub table: TenantBoundTable, // data_class: INTERNAL_ONLY
    pub policy_name: String,     // data_class: INTERNAL_ONLY
    pub using_expr: String,      // data_class: INTERNAL_ONLY
    pub check_expr: String,      // data_class: INTERNAL_ONLY
}

/// Sealed port for installing / verifying / auditing RLS policies.
pub trait RlsInstaller {
    fn install(&self, policy: &RlsPolicy) -> Result<(), IsolationKernelError>;
    fn verify(&self, policy: &RlsPolicy) -> Result<bool, IsolationKernelError>;
}

/// Ed25519 JWT issuer port (IP-007).
pub trait JwtIssuer {
    fn issue(&self, claims: &[(String, String)]) -> Result<String, IsolationKernelError>;
}

/// JWT verifier port (IP-007).
pub trait JwtVerifier {
    fn verify(&self, token: &str) -> Result<Vec<(String, String)>, IsolationKernelError>;
}

/// OpenBao-backed signing key store port (IP-007).
pub trait SigningKeyStore {
    fn current_key_fingerprint(&self) -> Result<String, IsolationKernelError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IsolationKernelError {
    PolicyMalformed,
    InstallFailed,
    VerifyFailed,
    JwtSignFailed,
    JwtVerifyFailed,
    KeyStoreUnavailable,
}
