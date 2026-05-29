//! Audit-chain emission kernel: port traits and value types.
//!
//! Wave 15-IMPL-truth-up scaffold (2026-05-21). Full extraction from
//! `oya-audit-chain-domain` is tracked under IP-003. The kernel must remain
//! free of Postgres, S3, HTTP, eventing, or file I/O imports per ADR-0105.
#![allow(dead_code)]

/// Identifier for a producing surface (e.g. `tenancy.tenant-onboarded`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProducerSurface(pub String);

/// Pack-local chain coordinate: `(pack, tenant_partition, period)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChainCoordinate {
    pub pack: String,             // data_class: INTERNAL_ONLY
    pub tenant_partition: String, // data_class: INTERNAL_ONLY
    pub period: String,           // data_class: INTERNAL_ONLY
}

/// Port for emitting an audit envelope. Implementations live in
/// `oya-audit-chain-emission-adapter` and are scoped by IP-005.
pub trait AuditEmitter {
    type Envelope;
    type Receipt;
    type Error;
    fn emit(&self, envelope: Self::Envelope) -> Result<Self::Receipt, Self::Error>;
}

/// Port for the write-ahead log used by the sealing worker. Scoped by IP-005.
pub trait WalWriter {
    type Record;
    type Error;
    fn append(&self, record: Self::Record) -> Result<(), Self::Error>;
}

/// Port that resolves a calling principal to a producer surface. Scoped by IP-005.
pub trait PrincipalResolver {
    type Principal;
    type Error;
    fn resolve(&self, principal: Self::Principal) -> Result<ProducerSurface, Self::Error>;
}
