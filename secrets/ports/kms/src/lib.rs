//! Agreed cross-owner surface for KMS reference and policy values.
//!
//! Consumers receive Secrets' established exact value identities, typed
//! validation errors, and validation behavior without importing its internal
//! aggregate. This port intentionally excludes key material, provider clients,
//! cryptographic operations, and the KMS directory. The legacy domain remains
//! the defining crate until its large crate root is decomposed in a dedicated
//! Secrets structural lane.

#![forbid(unsafe_code)]

pub use secrets_kms_domain::{
    CiphertextRef, CloudKmsError, DestructionProofRef, KmsKeyId, KmsKeyOrigin, KmsPurpose,
    KmsUseEventId, MaterialRef,
};
