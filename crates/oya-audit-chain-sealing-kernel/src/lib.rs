//! Audit-chain sealing kernel: signer/publisher port traits and key epoch types.
//!
//! Wave 15-IMPL-truth-up scaffold (2026-05-21). Full extraction tracked under
//! IP-006. Kernel must remain free of PKCS#11, S3, Postgres, Mimir, and HTTP
//! imports per policy/seal-integrity.md SI-06 through SI-13.
#![allow(dead_code)]

/// Reference to a signing key handle held inside an HSM.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SigningKeyRef {
    pub key_id: String, // data_class: INTERNAL_ONLY
}

/// Pack-scoped key epoch covering a half-open period range. Full schema in IP-006.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackEpoch {
    pub pack: String,                        // data_class: AUDIT
    pub tenant_partition: String,            // data_class: AUDIT
    pub period_lo: String,                   // data_class: AUDIT
    pub period_hi: String,                   // data_class: AUDIT
    pub active_key: SigningKeyRef,           // data_class: INTERNAL_ONLY
    pub retiring_key: Option<SigningKeyRef>, // data_class: INTERNAL_ONLY
}

/// Lifecycle status of a SealRecord. Full enum in IP-006.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SealStatus {
    Accepted,
    Unsealed,
    Sealed,
    Published,
    Verified,
    Redacted,
    Retained,
}

/// Seal record summary; persisted by `oya-audit-chain-sealing-adapter-postgres`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SealRecord {
    pub pack: String,               // data_class: AUDIT
    pub tenant_partition: String,   // data_class: AUDIT
    pub period_id: String,          // data_class: AUDIT
    pub leaf_count: u64,            // data_class: AUDIT
    pub merkle_root: String,        // data_class: AUDIT
    pub prior_root: Option<String>, // data_class: AUDIT
    pub signing_key: SigningKeyRef, // data_class: INTERNAL_ONLY
    pub status: SealStatus,         // data_class: AUDIT
}

/// Merkle root construction port. Pure; implementation in sealing-domain.
pub trait MerkleEngine {
    type Leaf;
    type Root;
    type Error;
    fn root(&self, leaves: &[Self::Leaf]) -> Result<Self::Root, Self::Error>;
}

/// Signer port: receives root bytes, returns key id plus signature metadata.
pub trait SignerPort {
    type Root;
    type Signature;
    type Error;
    fn sign(&self, root: &Self::Root, epoch: &PackEpoch) -> Result<Self::Signature, Self::Error>;
}

/// Publisher port: emits root references to WORM, Mimir, and GitHub channels.
pub trait RootPublisher {
    type Root;
    type Reference;
    type Error;
    fn publish(&self, root: &Self::Root) -> Result<Self::Reference, Self::Error>;
}

/// Append-only seal index writer.
pub trait IndexWriter {
    type Error;
    fn insert(&self, record: &SealRecord) -> Result<(), Self::Error>;
}

/// Append-only blob writer for raw proof material.
pub trait ObjectStoreWriter {
    type Blob;
    type Error;
    fn put(&self, blob: &Self::Blob) -> Result<(), Self::Error>;
}
