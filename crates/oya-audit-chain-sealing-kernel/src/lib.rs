//! Audit-chain sealing kernel: signer/publisher port traits and key epoch types.
//!
//! Wave 15-IMPL-truth-up scaffold (2026-05-21). Full extraction tracked under
//! IP-006. Kernel must remain free of PKCS#11, S3, Postgres, Mimir, and HTTP
//! imports per policy/seal-integrity.md SI-06 through SI-13.
#![allow(dead_code)]

/// Reference to a signing key handle held inside an HSM.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SigningKeyRef {
    pub key_id: String,
}

/// Pack-scoped key epoch covering a half-open period range. Full schema in IP-006.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackEpoch {
    pub pack: String,
    pub tenant_partition: String,
    pub period_lo: String,
    pub period_hi: String,
    pub active_key: SigningKeyRef,
    pub retiring_key: Option<SigningKeyRef>,
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
    pub pack: String,
    pub tenant_partition: String,
    pub period_id: String,
    pub leaf_count: u64,
    pub merkle_root: String,
    pub prior_root: Option<String>,
    pub signing_key: SigningKeyRef,
    pub status: SealStatus,
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
