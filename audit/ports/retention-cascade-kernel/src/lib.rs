//! Audit-chain retention-cascade kernel: policy and cascade ports.
//!
//! Policy-source, cascade-source and redaction-writer ports. Pure traits; the
//! rules applied over them live in `audit/core/retention-cascade-domain`.
#![allow(dead_code)]

/// Port: load retention windows from `policy/retention-matrix.yaml`.
pub trait RetentionPolicySource {
    type Policy;
    type Error;
    fn load(&self) -> Result<Self::Policy, Self::Error>;
}

/// Port: receive DSR cascades from tenancy/cloud-secrets.
pub trait DsrCascadeSource {
    type Request;
    type Error;
    fn next(&self) -> Result<Option<Self::Request>, Self::Error>;
}

/// Port: write redaction markers that preserve proof of original existence.
pub trait RedactionWriter {
    type Marker;
    type Error;
    fn redact(&self, marker: &Self::Marker) -> Result<(), Self::Error>;
}
