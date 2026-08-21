//! Audit-chain kernel: append-only tamper-evident event chain.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod merkle_tree;
pub use merkle_tree::{MerkleTree, Sha256Hash};

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use data_boundary_kernel::{DataClass, DataClassification, Purpose};
use sha2::{Digest, Sha256};

const GENESIS_HASH: &str = "GENESIS";
const EMPTY_MERKLE_ROOT: &str = "merkle-sha256:GENESIS";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Plane {
    Control,
    Data,
    Audit,
    Analytics,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantShardId {
    pub value: String, // data_class: INTERNAL_ONLY
}

impl TenantShardId {
    pub fn for_tenant(tenant_id: &str) -> Self {
        Self {
            value: format!("tenant:{tenant_id}"),
        }
    }

    pub fn new(value: impl Into<String>) -> Result<Self, AuditChainError> {
        let value = value.into();
        if value.trim().is_empty() {
            Err(AuditChainError::EmptyTenantShard)
        } else {
            Ok(Self { value })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerkleRoot {
    pub value: String, // data_class: INTERNAL_ONLY
}

impl MerkleRoot {
    pub fn genesis() -> Self {
        Self {
            value: EMPTY_MERKLE_ROOT.to_string(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ed25519VerificationKey {
    pub key_id: String,         // data_class: INTERNAL_ONLY
    pub public_key_hex: String, // data_class: INTERNAL_ONLY
}

impl Ed25519VerificationKey {
    pub fn as_bytes(&self) -> Result<[u8; 32], AuditChainError> {
        decode_hex_array::<32>(&self.public_key_hex)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Ed25519VerificationKeySet {
    keys: Vec<Ed25519VerificationKey>, // data_class: INTERNAL_ONLY
}

impl Ed25519VerificationKeySet {
    pub fn from_keys(
        keys: impl IntoIterator<Item = Ed25519VerificationKey>,
    ) -> Result<Self, AuditChainError> {
        let mut unique = Vec::new();
        for key in keys {
            if key.key_id.trim().is_empty() {
                return Err(AuditChainError::EmptySigningKeyId);
            }
            if unique
                .iter()
                .any(|existing: &Ed25519VerificationKey| existing.key_id == key.key_id)
            {
                return Err(AuditChainError::DuplicateTrustedEd25519Key { key_id: key.key_id });
            }
            unique.push(key);
        }
        Ok(Self { keys: unique })
    }

    pub fn single(key: Ed25519VerificationKey) -> Result<Self, AuditChainError> {
        Self::from_keys([key])
    }

    pub fn trusted_key_for(
        &self,
        key_id: &str,
    ) -> Result<&Ed25519VerificationKey, AuditChainError> {
        self.keys
            .iter()
            .find(|key| key.key_id == key_id)
            .ok_or_else(|| AuditChainError::MissingTrustedEd25519Key {
                key_id: key_id.to_string(),
            })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Ed25519SigningKey {
    key_id: String,  // data_class: INTERNAL_ONLY
    key: SigningKey, // data_class: SECRET
}

impl std::fmt::Debug for Ed25519SigningKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Ed25519SigningKey")
            .field("key_id", &self.key_id)
            .finish_non_exhaustive()
    }
}

impl Ed25519SigningKey {
    pub fn from_seed_bytes(
        key_id: impl Into<String>,
        seed: [u8; 32],
    ) -> Result<Self, AuditChainError> {
        let key_id = key_id.into();
        if key_id.trim().is_empty() {
            return Err(AuditChainError::EmptySigningKeyId);
        }
        Ok(Self {
            key_id,
            key: SigningKey::from_bytes(&seed),
        })
    }

    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    pub fn verification_key(&self) -> Ed25519VerificationKey {
        Ed25519VerificationKey {
            key_id: self.key_id.clone(),
            public_key_hex: encode_hex(&self.key.verifying_key().to_bytes()),
        }
    }

    pub fn sign(&self, payload: &[u8]) -> Ed25519Signature {
        let signature: Signature = self.key.sign(payload);
        Ed25519Signature {
            key_id: self.key_id.clone(),
            public_key_hex: encode_hex(&self.key.verifying_key().to_bytes()),
            signature_hex: encode_hex(&signature.to_bytes()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ed25519Signature {
    pub key_id: String,         // data_class: INTERNAL_ONLY
    pub public_key_hex: String, // data_class: INTERNAL_ONLY
    pub signature_hex: String,  // data_class: INTERNAL_ONLY
}

impl Ed25519Signature {
    pub fn verification_key(&self) -> Ed25519VerificationKey {
        Ed25519VerificationKey {
            key_id: self.key_id.clone(),
            public_key_hex: self.public_key_hex.clone(),
        }
    }

    pub fn verify_with_trusted_key(
        &self,
        payload: &[u8],
        trusted_key: &Ed25519VerificationKey,
    ) -> Result<(), AuditChainError> {
        if self.key_id != trusted_key.key_id || self.public_key_hex != trusted_key.public_key_hex {
            return Err(AuditChainError::Ed25519SignatureKeyMismatch {
                key_id: self.key_id.clone(),
            });
        }
        let public_key = trusted_key.as_bytes()?;
        let signature = decode_hex_array::<64>(&self.signature_hex)?;
        let verifying_key = VerifyingKey::from_bytes(&public_key)
            .map_err(|_| AuditChainError::InvalidEd25519PublicKey)?;
        let signature = Signature::from_bytes(&signature);
        verifying_key
            .verify_strict(payload, &signature)
            .map_err(|_| AuditChainError::InvalidEd25519Signature)
    }

    pub fn verify_with_trusted_keys(
        &self,
        payload: &[u8],
        trusted_keys: &Ed25519VerificationKeySet,
    ) -> Result<(), AuditChainError> {
        let trusted_key = trusted_keys.trusted_key_for(&self.key_id)?;
        self.verify_with_trusted_key(payload, trusted_key)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEvent {
    pub sequence: u64,                               // data_class: INTERNAL_ONLY
    pub tenant_shard: TenantShardId,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                           // data_class: INTERNAL_ONLY
    pub surface: String,                             // data_class: INTERNAL_ONLY
    pub plane: Plane,                                // data_class: INTERNAL_ONLY
    pub purpose: Purpose,                            // data_class: INTERNAL_ONLY
    pub data_classes: Vec<DataClass>,                // data_class: INTERNAL_ONLY
    pub decision: String,                            // data_class: INTERNAL_ONLY
    pub previous_hash: String,                       // data_class: INTERNAL_ONLY
    pub hash: String,                                // data_class: INTERNAL_ONLY
    pub merkle_root: MerkleRoot,                     // data_class: INTERNAL_ONLY
    pub ed25519_signature: Option<Ed25519Signature>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AuditChainScope {
    SingleTenantShard,
    MultiTenantShard,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditChain {
    events: Vec<AuditEvent>, // data_class: INTERNAL_ONLY
    scope: AuditChainScope,  // data_class: INTERNAL_ONLY
}

impl Default for AuditChain {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            scope: AuditChainScope::SingleTenantShard,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditAppendInput {
    pub tenant_id: String,            // data_class: INTERNAL_ONLY
    pub surface: String,              // data_class: INTERNAL_ONLY
    pub plane: Plane,                 // data_class: INTERNAL_ONLY
    pub purpose: Purpose,             // data_class: INTERNAL_ONLY
    pub data_classes: Vec<DataClass>, // data_class: INTERNAL_ONLY
    pub decision: String,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditChainError {
    EmptyTenantId,
    EmptyTenantShard,
    EmptySigningKeyId,
    InvalidChain,
    TenantShardMismatch {
        expected: TenantShardId,
        actual: TenantShardId,
    },
    MissingEd25519Signature {
        sequence: u64,
    },
    MissingTrustedEd25519Key {
        key_id: String,
    },
    DuplicateTrustedEd25519Key {
        key_id: String,
    },
    Ed25519SignatureKeyMismatch {
        key_id: String,
    },
    InvalidHexLength {
        expected_bytes: usize,
        actual_hex_len: usize,
    },
    InvalidHexDigit,
    InvalidEd25519PublicKey,
    InvalidEd25519Signature,
}

impl AuditChain {
    /// Build an aggregate ledger that still stores independently verifiable
    /// per-tenant shard chains.
    ///
    /// `AuditChain::default()` remains a single-shard chain because the
    /// canonical file-ledger/replay path is one tenant shard at a time. The
    /// foundation application uses this aggregate shape to keep one in-memory
    /// audit surface while preserving per-shard sequence, previous-hash, and
    /// Merkle-prefix invariants.
    pub fn multi_tenant_shards() -> Self {
        Self {
            events: Vec::new(),
            scope: AuditChainScope::MultiTenantShard,
        }
    }

    pub fn from_events(events: Vec<AuditEvent>) -> Result<Self, AuditChainError> {
        let chain = Self {
            events,
            scope: AuditChainScope::SingleTenantShard,
        };
        verify_chain(&chain)?;
        Ok(chain)
    }

    pub fn from_multi_tenant_shard_events(
        events: Vec<AuditEvent>,
    ) -> Result<Self, AuditChainError> {
        let chain = Self {
            events,
            scope: AuditChainScope::MultiTenantShard,
        };
        verify_chain(&chain)?;
        Ok(chain)
    }

    pub fn from_signed_events(
        events: Vec<AuditEvent>,
        trusted_keys: &Ed25519VerificationKeySet,
    ) -> Result<Self, AuditChainError> {
        let chain = Self::from_events(events)?;
        chain.verify_signed_with_keys(trusted_keys)?;
        Ok(chain)
    }

    fn append_legacy_data_classes(
        &mut self,
        tenant_id: impl Into<String>,
        surface: impl Into<String>,
        plane: Plane,
        purpose: Purpose,
        data_classes: Vec<DataClass>,
        decision: impl Into<String>,
    ) -> Result<&AuditEvent, AuditChainError> {
        // ADR-0083 Tier 1 (amendment 2026-05-15): private helper propagates
        // the fallible `try_append_legacy_data_classes` instead of erasing
        // its two real failure modes (`EmptyTenantId`, `TenantShardMismatch`)
        // behind `.expect(...)`.
        self.try_append_legacy_data_classes(
            AuditAppendInput {
                tenant_id: tenant_id.into(),
                surface: surface.into(),
                plane,
                purpose,
                data_classes,
                decision: decision.into(),
            },
            None,
        )
    }

    fn try_append_legacy_data_classes(
        &mut self,
        input: AuditAppendInput,
        signer: Option<&Ed25519SigningKey>,
    ) -> Result<&AuditEvent, AuditChainError> {
        let tenant_id = input.tenant_id;
        if tenant_id.trim().is_empty() {
            return Err(AuditChainError::EmptyTenantId);
        }
        let tenant_shard = TenantShardId::for_tenant(&tenant_id);
        let (sequence, previous_hash, existing_hashes) = match self.scope {
            AuditChainScope::SingleTenantShard => {
                if let Some(existing) = self.events.first()
                    && existing.tenant_shard != tenant_shard
                {
                    return Err(AuditChainError::TenantShardMismatch {
                        expected: existing.tenant_shard.clone(),
                        actual: tenant_shard,
                    });
                }
                (
                    self.events.len() as u64,
                    self.events
                        .last()
                        .map(|event| event.hash.clone())
                        .unwrap_or_else(|| GENESIS_HASH.to_string()),
                    self.events
                        .iter()
                        .map(|event| event.hash.clone())
                        .collect::<Vec<_>>(),
                )
            }
            AuditChainScope::MultiTenantShard => {
                let shard_events = self
                    .events
                    .iter()
                    .filter(|event| event.tenant_shard == tenant_shard)
                    .collect::<Vec<_>>();
                (
                    shard_events.len() as u64,
                    shard_events
                        .last()
                        .map(|event| event.hash.clone())
                        .unwrap_or_else(|| GENESIS_HASH.to_string()),
                    shard_events
                        .iter()
                        .map(|event| event.hash.clone())
                        .collect::<Vec<_>>(),
                )
            }
        };
        let mut event = AuditEvent {
            sequence,
            tenant_shard,
            tenant_id,
            surface: input.surface,
            plane: input.plane,
            purpose: input.purpose,
            data_classes: input.data_classes,
            decision: input.decision,
            previous_hash,
            hash: String::new(),
            merkle_root: MerkleRoot::genesis(),
            ed25519_signature: None,
        };
        event.hash = event_hash(&event);
        event.merkle_root = merkle_root_for_prefix(existing_hashes.iter(), &event.hash);
        if let Some(signer) = signer {
            event.ed25519_signature = Some(signer.sign(&event.signing_payload()));
        }
        // ADR-0083 Tier 1: index by the now-known length instead of
        // `.last().expect("just pushed")`. The push above guarantees the
        // slot exists, but we avoid `.expect()` on the public path by going
        // through deterministic indexing.
        let new_len = self.events.len() + 1;
        self.events.push(event);
        Ok(&self.events[new_len - 1])
    }

    /// Append typed field classifications while preserving the legacy
    /// `DataClass` audit payload and hash input. This is the compatibility seam
    /// for append-only ledger replay while callers migrate operational markers
    /// such as `AUDIT` out of privacy-program `DataClass` construction.
    ///
    /// Returns `Err(AuditChainError::EmptyTenantId)` if `tenant_id` is blank
    /// and `Err(AuditChainError::TenantShardMismatch { .. })` if the chain is
    /// in `SingleTenantShard` scope and the incoming tenant shard differs from
    /// the existing one. See ADR-0083 amendment 2026-05-15 — Tier 1 forbids
    /// erasing these matchable failure modes behind `.expect(...)`. Crate
    /// version `0.2.0`.
    pub fn append_classifications<C>(
        &mut self,
        tenant_id: impl Into<String>,
        surface: impl Into<String>,
        plane: Plane,
        purpose: Purpose,
        data_classifications: impl IntoIterator<Item = C>,
        decision: impl Into<String>,
    ) -> Result<&AuditEvent, AuditChainError>
    where
        C: Into<DataClassification>,
    {
        self.append_legacy_data_classes(
            tenant_id,
            surface,
            plane,
            purpose,
            data_classifications
                .into_iter()
                .map(|classification| classification.into().compatibility_data_class())
                .collect(),
            decision,
        )
    }

    pub fn append_signed(
        &mut self,
        input: AuditAppendInput,
        signer: &Ed25519SigningKey,
    ) -> Result<&AuditEvent, AuditChainError> {
        self.try_append_legacy_data_classes(input, Some(signer))
    }

    pub fn events(&self) -> &[AuditEvent] {
        &self.events
    }

    pub fn is_multi_tenant_shards(&self) -> bool {
        self.scope == AuditChainScope::MultiTenantShard
    }

    pub fn merkle_root(&self) -> MerkleRoot {
        self.events
            .last()
            .map(|event| event.merkle_root.clone())
            .unwrap_or_else(MerkleRoot::genesis)
    }

    pub fn verify(&self) -> bool {
        verify_chain(self).is_ok()
    }

    pub fn verify_signed_with_keys(
        &self,
        trusted_keys: &Ed25519VerificationKeySet,
    ) -> Result<MerkleRoot, AuditChainError> {
        let root = verify_chain(self)?;
        for event in &self.events {
            event
                .ed25519_signature
                .as_ref()
                .ok_or(AuditChainError::MissingEd25519Signature {
                    sequence: event.sequence,
                })?
                .verify_with_trusted_keys(&event.signing_payload(), trusted_keys)?;
        }
        Ok(root)
    }
}

impl AuditEvent {
    fn signing_payload(&self) -> Vec<u8> {
        [
            "audit-chain-ed25519-v1".to_string(),
            format!("sequence={}", self.sequence),
            format!("tenant_shard={}", self.tenant_shard.as_str()),
            format!("hash={}", self.hash),
            format!("merkle_root={}", self.merkle_root.as_str()),
        ]
        .join("\n")
        .into_bytes()
    }
}

pub fn append<'a>(
    chain: &'a mut AuditChain,
    input: AuditAppendInput,
    signer: Option<&Ed25519SigningKey>,
) -> Result<&'a AuditEvent, AuditChainError> {
    chain.try_append_legacy_data_classes(input, signer)
}

pub fn verify_chain(chain: &AuditChain) -> Result<MerkleRoot, AuditChainError> {
    if chain.scope == AuditChainScope::MultiTenantShard {
        return verify_multi_tenant_shards(chain);
    }

    let mut previous = GENESIS_HASH.to_string();
    let mut expected_shard = None::<TenantShardId>;
    let mut prefix_hashes = Vec::<String>::new();
    for (index, event) in chain.events.iter().enumerate() {
        if event.sequence != index as u64 || event.previous_hash != previous {
            return Err(AuditChainError::InvalidChain);
        }
        if event.tenant_id.trim().is_empty() {
            return Err(AuditChainError::EmptyTenantId);
        }
        let actual_shard = TenantShardId::for_tenant(&event.tenant_id);
        if event.tenant_shard != actual_shard {
            return Err(AuditChainError::TenantShardMismatch {
                expected: actual_shard,
                actual: event.tenant_shard.clone(),
            });
        }
        if let Some(expected) = &expected_shard {
            if expected != &event.tenant_shard {
                return Err(AuditChainError::TenantShardMismatch {
                    expected: expected.clone(),
                    actual: event.tenant_shard.clone(),
                });
            }
        } else {
            expected_shard = Some(event.tenant_shard.clone());
        }
        if event.hash != event_hash(event) {
            return Err(AuditChainError::InvalidChain);
        }
        prefix_hashes.push(event.hash.clone());
        let expected_root = merkle_root(&prefix_hashes);
        if event.merkle_root != expected_root {
            return Err(AuditChainError::InvalidChain);
        }
        previous = event.hash.clone();
    }
    Ok(prefix_hashes
        .last()
        .map(|_| merkle_root(&prefix_hashes))
        .unwrap_or_else(MerkleRoot::genesis))
}

pub fn merkle_root_for_events(events: &[AuditEvent]) -> Result<MerkleRoot, AuditChainError> {
    verify_chain(&AuditChain {
        events: events.to_vec(),
        scope: AuditChainScope::SingleTenantShard,
    })
}

#[derive(Default)]
struct ShardVerifyState {
    previous_hash: Option<String>,
    prefix_hashes: Vec<String>,
}

fn verify_multi_tenant_shards(chain: &AuditChain) -> Result<MerkleRoot, AuditChainError> {
    use std::collections::BTreeMap;

    let mut shards = BTreeMap::<String, ShardVerifyState>::new();
    for event in &chain.events {
        if event.tenant_id.trim().is_empty() {
            return Err(AuditChainError::EmptyTenantId);
        }
        let actual_shard = TenantShardId::for_tenant(&event.tenant_id);
        if event.tenant_shard != actual_shard {
            return Err(AuditChainError::TenantShardMismatch {
                expected: actual_shard,
                actual: event.tenant_shard.clone(),
            });
        }
        let state = shards
            .entry(event.tenant_shard.as_str().to_string())
            .or_default();
        let expected_sequence = state.prefix_hashes.len() as u64;
        let expected_previous = state
            .previous_hash
            .clone()
            .unwrap_or_else(|| GENESIS_HASH.to_string());
        if event.sequence != expected_sequence || event.previous_hash != expected_previous {
            return Err(AuditChainError::InvalidChain);
        }
        if event.hash != event_hash(event) {
            return Err(AuditChainError::InvalidChain);
        }
        state.prefix_hashes.push(event.hash.clone());
        let expected_root = merkle_root(&state.prefix_hashes);
        if event.merkle_root != expected_root {
            return Err(AuditChainError::InvalidChain);
        }
        state.previous_hash = Some(event.hash.clone());
    }
    let shard_roots = shards
        .values()
        .filter_map(|state| {
            state
                .prefix_hashes
                .last()
                .map(|_| merkle_root(&state.prefix_hashes).value)
        })
        .collect::<Vec<_>>();
    Ok(if shard_roots.is_empty() {
        MerkleRoot::genesis()
    } else {
        merkle_root(&shard_roots)
    })
}

fn merkle_root_for_prefix<'a>(
    existing_hashes: impl Iterator<Item = &'a String>,
    next_hash: &str,
) -> MerkleRoot {
    let mut hashes = existing_hashes.cloned().collect::<Vec<_>>();
    hashes.push(next_hash.to_string());
    merkle_root(&hashes)
}

fn merkle_root(hashes: &[String]) -> MerkleRoot {
    if hashes.is_empty() {
        return MerkleRoot::genesis();
    }
    let mut level = hashes
        .iter()
        .map(|hash| digest_prefixed("merkle-leaf", [hash.as_str()]))
        .collect::<Vec<_>>();
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for chunk in level.chunks(2) {
            let left = chunk[0].as_str();
            let right = chunk.get(1).map(String::as_str).unwrap_or(left);
            next.push(digest_prefixed("merkle-node", [left, right]));
        }
        level = next;
    }
    MerkleRoot {
        value: format!("merkle-{}", level.remove(0)),
    }
}

fn event_hash(event: &AuditEvent) -> String {
    digest_prefixed(
        "audit-event-v2",
        vec![
            event.sequence.to_string(),
            event.tenant_shard.as_str().to_string(),
            event.tenant_id.clone(),
            event.surface.clone(),
            format!("{:?}", event.plane),
            format!("{:?}", event.purpose),
            format!("{:?}", event.data_classes),
            event.decision.clone(),
            event.previous_hash.clone(),
        ],
    )
}

fn digest_prefixed<I, S>(domain: &str, fields: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    fn feed(hasher: &mut Sha256, value: &str) {
        hasher.update((value.len() as u64).to_be_bytes());
        hasher.update(value.as_bytes());
    }
    let mut hasher = Sha256::new();
    feed(&mut hasher, domain);
    for field in fields {
        feed(&mut hasher, field.as_ref());
    }
    format!("sha256:{}", encode_hex(&hasher.finalize()))
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn decode_hex_array<const N: usize>(hex: &str) -> Result<[u8; N], AuditChainError> {
    if hex.len() != N * 2 {
        return Err(AuditChainError::InvalidHexLength {
            expected_bytes: N,
            actual_hex_len: hex.len(),
        });
    }
    let mut output = [0_u8; N];
    let bytes = hex.as_bytes();
    for (index, out) in output.iter_mut().enumerate() {
        let high = hex_value(bytes[index * 2])?;
        let low = hex_value(bytes[index * 2 + 1])?;
        *out = (high << 4) | low;
    }
    Ok(output)
}

fn hex_value(byte: u8) -> Result<u8, AuditChainError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(AuditChainError::InvalidHexDigit),
    }
}

#[cfg(test)]
mod tests {
    use data_boundary_kernel::{DataClassification, OperationalDataClass};

    use super::*;

    #[test]
    fn classified_append_preserves_legacy_audit_hash_payload() {
        let mut legacy = AuditChain::default();
        let legacy_event = legacy
            .append_legacy_data_classes(
                "ten_alpha",
                "foundry.evidence.emit",
                Plane::Audit,
                Purpose::CapabilityInvocation,
                vec![DataClass::InternalOnly, DataClass::Audit],
                "ALLOW",
            )
            .expect("test-side: append must succeed for valid inputs")
            .clone();

        let mut classified = AuditChain::default();
        let classified_event = classified
            .append_classifications(
                "ten_alpha",
                "foundry.evidence.emit",
                Plane::Audit,
                Purpose::CapabilityInvocation,
                [
                    DataClassification::from(DataClass::InternalOnly),
                    DataClassification::from(OperationalDataClass::Audit),
                ],
                "ALLOW",
            )
            .expect("test-side: append must succeed for valid inputs")
            .clone();

        assert_eq!(classified_event.data_classes, legacy_event.data_classes);
        assert_eq!(classified_event.hash, legacy_event.hash);
        assert_eq!(classified_event.merkle_root, legacy_event.merkle_root);
        assert_eq!(classified.events(), legacy.events());
        assert!(classified.verify());
    }
}
