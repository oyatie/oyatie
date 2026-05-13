---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-substrate
phase: P08-kms
impl_plan_id: IP-P08-kms-substrate
status: pending
owner: council-architecture
blocked_by:
  - impl_plan: IP-P06-secrets-substrate
    reason: "KMS stores key references in OpenBao via Secrets substrate"
acceptance_lanes:
  - cargo-check
  - cargo-build
  - cargo-clippy
  - cargo-nextest
  - cargo-deny
  - lean-a1
  - lean-a2
  - lean-a3
  - lean-a4
---

# IP-P08-kms-substrate: Scaffold 16 KMS crates with envelope encryption, ML-DSA-87, per-tenant DEK, OCI Vault adapter

## Intent

Delivers the complete KMS substrate: 16 crates across 3 BCs (keys, envelope, signing), AES-256-GCM envelope encryption with CBOR envelope header, per-tenant DEK isolation, ML-DSA-87 (NIST FIPS 204) post-quantum signing, Ed25519 signing bridge, OCI Vault adapter for production, `InMemoryKmsAdapter` for tests, key rotation (envelope-only re-encrypt), full DDL, Cedar policy, load test p99≤200ms on cache-hit encrypt path.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-kms-envelope-kernel/Cargo.toml` | create | EnvelopeEncryptionPort trait; CipherTextBlob, ZeroizingBytes types |
| `crates/oya-kms-envelope-kernel/src/types.rs` | create | CipherTextBlob (12-byte nonce + AES-GCM ciphertext + 16-byte tag + CBOR header), ZeroizingBytes, KekId, DekId, RotationReport |
| `crates/oya-kms-envelope-kernel/src/ports.rs` | create | EnvelopeEncryptionPort sealed trait |
| `crates/oya-kms-keys-kernel/Cargo.toml` | create | KmsKeyPort trait |
| `crates/oya-kms-keys-kernel/src/ports.rs` | create | KmsKeyPort sealed trait |
| `crates/oya-kms-signing-kernel/Cargo.toml` | create | PqSignerPort + Ed25519SignerPort traits |
| `crates/oya-kms-signing-kernel/src/ports.rs` | create | PqSignerPort + Ed25519SignerPort sealed traits |
| `crates/oya-kms-signing-kernel/src/types.rs` | create | MlDsa87Signature, MlDsa87PublicKey, Ed25519Signature, KeyId |
| `crates/oya-kms-envelope-domain/src/aes_gcm.rs` | create | AES-256-GCM encrypt/decrypt (ring or aes-gcm crate); nonce generation; GCM tag verify |
| `crates/oya-kms-envelope-domain/src/cbor_header.rs` | create | CBOR envelope header: {kek_id, dek_ciphertext, algorithm}; ciborium crate |
| `crates/oya-kms-signing-domain/src/preimage.rs` | create | canonical signing preimage builder (mirrors audit-chain preimage format) |
| `crates/oya-kms-signing-domain/src/ml_dsa87.rs` | create | ML-DSA-87 key generation + sign + verify (pqcrypto-mldsa crate) |
| `crates/oya-kms-envelope-application/src/encrypt.rs` | create | EncryptUseCase: get/create DEK → AES-GCM → CBOR envelope |
| `crates/oya-kms-envelope-application/src/rotate_kek.rs` | create | RotateKekUseCase: re-encrypt envelopes only; no ciphertext re-write |
| `crates/oya-kms-keys-application/src/lifecycle.rs` | create | CreateDekUseCase, RotateDekUseCase, RevokeDekUseCase |
| `crates/oya-kms-signing-application/src/sign.rs` | create | SignUseCase: ML-DSA-87 or Ed25519 dispatch |
| `crates/oya-kms-keys-adapter/src/oci_vault.rs` | create | OciVaultAdapter: OCI Vault REST API; create/get/rotate keys |
| `crates/oya-kms-keys-adapter/src/in_memory.rs` | create | InMemoryKmsAdapter: static test KEK; ring crate; never outside #[cfg(test)] |
| `crates/oya-kms-signing-adapter/src/ml_dsa87.rs` | create | MlDsa87Adapter: pqcrypto-mldsa; key stored in OCI Vault |
| `crates/oya-kms-signing-adapter/src/ed25519.rs` | create | Ed25519Adapter: ring::signature::Ed25519; bridges to audit-chain ChainSigner |
| `crates/oya-kms-worker/src/key_rotation.rs` | create | KeyRotationWorker: monthly DEK rotation check |
| `crates/oya-kms-rest/src/routes.rs` | create | POST /kms/v1/encrypt, POST /kms/v1/decrypt, POST /kms/v1/sign, POST /kms/v1/keys/rotate |
| `crates/oya-kms-app/src/main.rs` | create | composition root |
| `migrations/kms/V001__kms_init.sql` | create | DDL |
| `contracts/kms/kms.proto` | create | Protobuf schema |
| `policy/kms/kms.cedar` | create | Cedar policy |
| `tests/load/smoke-kms-encrypt.js` | create | k6 smoke test |
| `Cargo.toml` | update | add all 16 KMS crates |

---

## Crate Naming

```
NAME: oya-kms-envelope-kernel
JUSTIFICATION:
- microservice = kms: Key Management System substrate; envelope encryption + ML-DSA-87
- bc-tokens = envelope: AES-256-GCM envelope encryption BC; distinct from keys (lifecycle) and signing
- layer = kernel: EnvelopeEncryptionPort trait + CipherTextBlob/ZeroizingBytes types
- exemptions claimed: none

NAME: oya-kms-signing-kernel
JUSTIFICATION:
- microservice = kms: same µservice
- bc-tokens = signing: ML-DSA-87 post-quantum signing BC; Bominal ADR-0111
- layer = kernel: PqSignerPort + Ed25519SignerPort traits + signature types
- exemptions claimed: none
```

---

## Code Shape

### `migrations/kms/V001__kms_init.sql`

```sql
CREATE SCHEMA IF NOT EXISTS kms;

-- DEK (Data Encryption Key) registry — key material never stored here
CREATE TABLE kms.keys (
    key_id          uuid    PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid    NOT NULL,
    key_type        text    NOT NULL CHECK (key_type IN ('dek','kek','signing_ed25519','signing_ml_dsa87')),
    algorithm       text    NOT NULL,    -- 'AES-256-GCM', 'Ed25519', 'ML-DSA-87'
    kms_key_ref     text    NOT NULL,    -- OCI Vault key OCID or OpenBao path; never the key bytes
    version         int     NOT NULL DEFAULT 1,
    status          text    NOT NULL DEFAULT 'active' CHECK (status IN ('active','rotating','revoked')),
    created_at      timestamptz NOT NULL DEFAULT now(),
    rotated_at      timestamptz NULL,
    revoked_at      timestamptz NULL
);
ALTER TABLE kms.keys ENABLE ROW LEVEL SECURITY;
ALTER TABLE kms.keys FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON kms.keys
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_kms_keys_active
    ON kms.keys (tenant_id, key_type, status)
    WHERE status = 'active';

-- Key versions (for rotation history)
CREATE TABLE kms.key_versions (
    version_id      uuid    PRIMARY KEY DEFAULT gen_random_uuid(),
    key_id          uuid    NOT NULL REFERENCES kms.keys(key_id),
    tenant_id       uuid    NOT NULL,
    version         int     NOT NULL,
    kms_key_ref     text    NOT NULL,
    activated_at    timestamptz NOT NULL DEFAULT now(),
    retired_at      timestamptz NULL
);
ALTER TABLE kms.key_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE kms.key_versions FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON kms.key_versions
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_kms_key_versions_unique
    ON kms.key_versions (key_id, version);
```

### `crates/oya-kms-envelope-domain/src/aes_gcm.rs`

```rust
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Raw DEK bytes — zeroed when dropped.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct RawDek([u8; 32]);

impl RawDek {
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self(key)
    }
    pub fn as_bytes(&self) -> &[u8; 32] { &self.0 }
}

/// Encrypt plaintext with DEK. Returns: 12-byte nonce ∥ ciphertext ∥ 16-byte GCM tag.
pub fn encrypt_aes256gcm(dek: &RawDek, plaintext: &[u8]) -> Result<Vec<u8>, KmsError> {
    let key = Key::<Aes256Gcm>::from_slice(dek.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|_| KmsError::EncryptionFailed)?;
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext); // includes 16-byte GCM tag
    Ok(result)
}

/// Decrypt. Verifies GCM authentication tag. Returns Err on tampered ciphertext.
pub fn decrypt_aes256gcm(dek: &RawDek, blob: &[u8]) -> Result<Vec<u8>, KmsError> {
    if blob.len() < 28 { return Err(KmsError::InvalidCipherText); } // 12 nonce + at least 1 byte + 16 tag
    let (nonce_bytes, ciphertext) = blob.split_at(12);
    let key = Key::<Aes256Gcm>::from_slice(dek.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, ciphertext)
        .map_err(|_| KmsError::DecryptionFailed) // GCM tag mismatch → tamper detected
}

#[derive(Debug, thiserror::Error)]
pub enum KmsError {
    #[error("AES-256-GCM encryption failed")]
    EncryptionFailed,
    #[error("AES-256-GCM decryption failed — possible tampering or wrong key")]
    DecryptionFailed,
    #[error("invalid cipher text blob: too short")]
    InvalidCipherText,
    #[error("DEK not found for tenant")]
    DekNotFound,
    #[error("KMS unavailable: {0}")]
    Unavailable(String),
}
```

### `crates/oya-kms-signing-domain/src/ml_dsa87.rs`

```rust
//! ML-DSA-87 (NIST FIPS 204 Dilithium-5 equivalent) signing.
//! Uses pqcrypto-mldsa crate which wraps the reference C implementation.

use pqcrypto_mldsa::mldsa87::{
    detached_sign, keypair, open_detached, PublicKey, SecretKey, DetachedSignature,
};

pub struct MlDsa87KeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,  // zeroed on drop via explicit impl
}

impl MlDsa87KeyPair {
    /// Generate a new ML-DSA-87 key pair. Store secret_key in KMS; return public_key.
    pub fn generate() -> Self {
        let (pk, sk) = keypair();
        Self {
            public_key: pk.as_bytes().to_vec(),
            secret_key: sk.as_bytes().to_vec(),
        }
    }
}

impl Drop for MlDsa87KeyPair {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.secret_key.zeroize();
    }
}

/// Sign message with ML-DSA-87 secret key bytes.
pub fn ml_dsa87_sign(secret_key_bytes: &[u8], message: &[u8]) -> Result<Vec<u8>, super::KmsSignError> {
    let sk = SecretKey::from_bytes(secret_key_bytes)
        .map_err(|_| super::KmsSignError::InvalidKey)?;
    let sig = detached_sign(message, &sk);
    Ok(sig.as_bytes().to_vec())
}

/// Verify ML-DSA-87 signature.
pub fn ml_dsa87_verify(public_key_bytes: &[u8], message: &[u8], sig_bytes: &[u8]) -> Result<bool, super::KmsSignError> {
    let pk = PublicKey::from_bytes(public_key_bytes)
        .map_err(|_| super::KmsSignError::InvalidKey)?;
    let sig = DetachedSignature::from_bytes(sig_bytes)
        .map_err(|_| super::KmsSignError::InvalidSignature)?;
    Ok(open_detached(message, &sig, &pk).is_ok())
}
```

### `contracts/kms/kms.proto`

```proto
syntax = "proto3";
package oyatie.kms.v1;

message DekCreated {
    string tenant_id   = 1;
    string key_id      = 2;
    string algorithm   = 3;   // "AES-256-GCM"
    int64  timestamp_ms = 4;
}

message KekRotated {
    string tenant_id      = 1;
    string old_kek_id     = 2;
    string new_kek_id     = 3;
    int32  envelopes_rotated = 4;
    int64  timestamp_ms   = 5;
}

message SigningKeyGenerated {
    string tenant_id   = 1;
    string key_id      = 2;
    string algorithm   = 3;   // "ML-DSA-87" | "Ed25519"
    bytes  public_key  = 4;
    int64  timestamp_ms = 5;
}
```

### `tests/load/smoke-kms-encrypt.js`

```javascript
import http from 'k6/http';
import { check } from 'k6';
import { b64encode } from 'k6/encoding';

export const options = {
  vus: 50, duration: '60s',
  thresholds: {
    http_req_duration: ['p(99)<200'],   // cache-hit DEK path ≤200ms
    http_req_failed: ['rate<0.001'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8086';
const TENANT_ID = __ENV.TENANT_ID || '00000000-0000-0000-0000-000000000001';

export default function () {
  const plaintext = b64encode('test-plaintext-value-for-load-test');
  const res = http.post(`${BASE_URL}/kms/v1/encrypt`, JSON.stringify({ plaintext }),
    { headers: { 'Content-Type': 'application/json', 'X-Tenant-Id': TENANT_ID } });
  check(res, { 'encrypt 200': (r) => r.status === 200 });
}
```

---

## Acceptance Gates

```bash
cargo check -p oya-kms-envelope-kernel --all-features     # exit 0
cargo check -p oya-kms-signing-adapter --all-features      # exit 0
cargo clippy --workspace --all-features -- -D warnings      # exit 0
cargo nextest run --workspace --all-features                # exit 0
psql $DATABASE_URL -f migrations/kms/V001__kms_init.sql    # exit 0
# AES-256-GCM round-trip
cargo nextest run -p oya-kms-envelope-domain --test aes_gcm_round_trip  # exit 0
# GCM tamper detection
cargo nextest run -p oya-kms-envelope-domain --test gcm_tag_tamper_rejected  # exit 0
# Cross-tenant DEK isolation
cargo nextest run -p oya-kms-envelope-application --test cross_tenant_dek_rejected  # exit 0
# Key rotation envelope-only
cargo nextest run -p oya-kms-keys-application --test key_rotation_envelope_only  # exit 0
# ML-DSA-87 round-trip
cargo nextest run -p oya-kms-signing-domain --test ml_dsa87_round_trip  # exit 0
# Load test
k6 run tests/load/smoke-kms-encrypt.js --env BASE_URL=http://localhost:8086
```

---

## Test Plan

### Unit tests

| Test name | What it verifies |
|---|---|
| `test_aes_gcm_encrypt_decrypt_round_trip` | Encrypt → decrypt → same plaintext |
| `test_aes_gcm_gcm_tag_tamper_rejected` | Flip 1 bit in ciphertext → DecryptionFailed |
| `test_aes_gcm_wrong_key_rejected` | Different DEK → DecryptionFailed |
| `test_cbor_envelope_round_trip` | Serialize + deserialize envelope header |
| `test_cross_tenant_dek_cannot_decrypt` | DEK from tenant A cannot decrypt tenant B ciphertext |
| `test_kek_rotation_envelope_only` | After rotate_kek: old KEK rejected, new KEK decrypts |
| `test_ml_dsa87_sign_verify_round_trip` | ML-DSA-87 sign → verify passes |
| `test_ml_dsa87_wrong_key_rejected` | Wrong public key → verify returns false |
| `test_ml_dsa87_tampered_sig_rejected` | Flipped bit in signature → verify returns false |
| `test_in_memory_kms_adapter_cfg_test_only` | InMemoryKmsAdapter not exported in non-test builds |

### Integration tests

| Test name | What it verifies |
|---|---|
| `integration_envelope_encrypt_decrypt_full` | Full envelope stack: generate DEK → encrypt → store CBOR → decrypt |
| `integration_key_rotation_10_rows` | 10 rows encrypted → rotate KEK → all 10 decrypt under new KEK |

---

## Clean Architecture Compliance

| Crate | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-kms-envelope-kernel` | `kernel` | `zeroize` (external) | all project layers |
| `oya-kms-envelope-domain` | `domain` | `envelope-kernel`; `aes-gcm`, `ciborium` (external) | `adapter`, presentation |
| `oya-kms-signing-domain` | `domain` | `signing-kernel`; `pqcrypto-mldsa` (external) | `adapter`, presentation |
| `oya-kms-keys-adapter` | `adapter` | `keys-application`, `keys-kernel`; `reqwest` (external) | presentation |
| `oya-kms-app` | `app` | all | none |

---

## Load Test

```bash
k6 run tests/load/smoke-kms-encrypt.js --env BASE_URL=http://localhost:8086
# Pass: p99 ≤200ms (DEK cache hit); 0 errors at 50 VUs/60s
```

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent m02-wave-a-executor \
  --intent "IP-P08-kms: 16 crates + AES-256-GCM + ML-DSA-87 + OCI Vault + key rotation" \
  --ttl 7200 \
  crates/oya-kms-envelope-kernel/src/ports.rs::EnvelopeEncryptionPort \
  crates/oya-kms-signing-kernel/src/ports.rs::PqSignerPort \
  crates/oya-kms-signing-domain/src/ml_dsa87.rs::ml_dsa87_sign \
  migrations/kms/V001__kms_init.sql::kms_schema
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-P08-kms merged; 16 crates; AES-256-GCM envelope; ML-DSA-87 FIPS 204; per-tenant DEK; OCI Vault adapter; key rotation envelope-only; next: P09-search/impl-plan" \
  -i high \
  -k "M02,P08,IP-P08,kms"
```

---

## Next IP Pointer

`phases/P09-search/impl-plan.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- Bominal ADR-0111 (CipherText property type + KMS envelope)
- `pqcrypto-mldsa` crate: https://crates.io/crates/pqcrypto-mldsa
- `aes-gcm` crate: https://crates.io/crates/aes-gcm
