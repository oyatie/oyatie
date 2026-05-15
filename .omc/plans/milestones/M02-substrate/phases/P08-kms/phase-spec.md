---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-substrate
phase: P08-kms
status: Proposed
acceptance_lanes: []
entry_gate: 'M01-P05 complete; P06-secrets merged (OpenBao adapter available; KMS
  depends

  on Secrets for key material storage references); oya-tenancy-kernel ships;

  cargo check exits 0.

  '
exit_gate: 'All KMS crates compile; envelope encryption (AES-256-GCM + DEK + KEK)

  round-trip test green; per-tenant DEK isolation verified; ML-DSA-87 signing

  key generation test passes; key rotation integration test green (re-encrypt

  envelope only, no ciphertext re-write); Cedar policy lints; grit done; ICM row emitted.

  '
depends_on:
- milestone: M01
  phase: P05-scaffold-locks
  reason: workspace scaffold prerequisite
- milestone: M02
  phase: P06-secrets
  reason: KMS key references stored as SecretRefs in OpenBao via Secrets substrate
owner_team: council-architecture
purpose: Auto-backfilled purpose for phase-spec.md
---
# P08-kms: KMS substrate — envelope encryption, per-tenant DEK, per-cell HSM, ML-DSA-87 post-quantum signing

## Purpose

This phase delivers the complete KMS substrate: envelope encryption for data-at-rest field encryption (CipherText property type per Bominal ADR-0111), per-tenant DEK (Data Encryption Key) isolation with KMS-backed KEK (Key Encryption Key), per-cell HSM in production, and ML-DSA-87 post-quantum signing per Bominal ADR-0111 / ADR-0028 combined mandate. The envelope encryption pattern (`AES-256-GCM(DEK, plaintext) || CBOR_envelope(KEK, DEK)`) decouples key rotation from ciphertext — rotating the KEK requires re-encrypting only the 128-byte envelope header, not all data rows. ML-DSA-87 (NIST FIPS 204) is the post-quantum replacement for RSA/ECDSA signing in audit segments and cross-tenant verifiable credentials. Without KMS the CipherText property type, PHI field encryption, and post-quantum audit signing are unavailable.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `kms` | `keys`, `envelope`, `signing` | `crates/oya-kms-{keys,envelope,signing}-{kernel,domain,application,adapter}/`, `crates/oya-kms-worker/`, `crates/oya-kms-rest/`, `crates/oya-kms-app/` | 3×4 + 1 worker + 1 rest + 1 app = 16 crates |

Naming justification:

```
NAME: oya-kms-keys-kernel
JUSTIFICATION:
- microservice = kms: Key Management System substrate; envelope encryption
  + ML-DSA-87 post-quantum signing
- bc-tokens = keys: the key lifecycle BC (create/rotate/revoke per-tenant DEK/KEK)
- layer = kernel: KmsKeyPort + KeyId, KeyVersion types; zero I/O
- exemptions claimed: none

NAME: oya-kms-signing-kernel
JUSTIFICATION:
- microservice = kms: same µservice
- bc-tokens = signing: ML-DSA-87 post-quantum signing BC; distinct from envelope
  (AES-GCM data encryption) and keys (lifecycle management)
- layer = kernel: PqSignerPort trait + MlDsa87Signature type
- exemptions claimed: none
```

### Out-of-scope

- TLS certificate management — handled by Kubernetes cert-manager; not in KMS scope.
- OpenBao vault storage — owned by P06-secrets; KMS stores key references there.
- FIDO2 signing keys — owned by P03-identity passkeys BC.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Full DDL + KmsKeyPort + EnvelopeEncryptionPort + PqSignerPort + OCI Vault adapter + ML-DSA-87 + rotation + load test | pending | `council-architecture` |

---

## Acceptance Gates

### Cargo / CI gates

```bash
cargo check --workspace --all-features               # exit 0
cargo build --workspace --all-features               # exit 0
cargo clippy --workspace --all-features -- -D warnings  # exit 0
cargo nextest run --workspace --all-features         # exit 0; 0 failures
cargo deny check                                     # exit 0
cargo doc --workspace --no-deps                      # exit 0; 0 warnings
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --phase P08-kms
oya gate validate lean-a2 --phase P08-kms
oya gate validate lean-a3 --phase P08-kms
oya gate validate lean-a4 --phase P08-kms
```

### Cryptographic correctness gates

```bash
# AES-256-GCM envelope encrypt → decrypt round-trip
cargo nextest run -p oya-kms-envelope-domain --test aes_gcm_round_trip  # exit 0
# GCM authentication tag tamper detection
cargo nextest run -p oya-kms-envelope-domain --test gcm_tag_tamper_rejected  # exit 0
# Per-tenant DEK isolation: DEK-A cannot decrypt DEK-B ciphertext
cargo nextest run -p oya-kms-envelope-application --test cross_tenant_dek_rejected  # exit 0
# Key rotation: re-encrypt envelope only; original plaintext recoverable
cargo nextest run -p oya-kms-keys-application --test key_rotation_envelope_only  # exit 0
# ML-DSA-87 sign → verify round-trip
cargo nextest run -p oya-kms-signing-domain --test ml_dsa87_round_trip  # exit 0
# ML-DSA-87 wrong-key verify rejected
cargo nextest run -p oya-kms-signing-domain --test ml_dsa87_wrong_key_rejected  # exit 0
```

### Load test gate

```bash
k6 run tests/load/smoke-kms-encrypt.js --env BASE_URL=http://localhost:8086
# Pass: p99 ≤200ms on envelope encrypt (DEK cache hit path); 0 errors
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? | Presentation-only? |
|---|---|---|---|---|
| `oya-kms-keys-kernel` | `kernel` | Yes — `KmsKeyPort` | N/A | No |
| `oya-kms-envelope-kernel` | `kernel` | Yes — `EnvelopeEncryptionPort` | N/A | No |
| `oya-kms-signing-kernel` | `kernel` | Yes — `PqSignerPort`, `Ed25519SignerPort` | N/A | No |
| `oya-kms-envelope-domain` | `domain` | N/A — pure encryption logic (no I/O) | N/A | No |
| `oya-kms-keys-adapter` | `adapter` | N/A | Yes — `OciVaultAdapter`, `InMemoryKmsAdapter` | No |
| `oya-kms-signing-adapter` | `adapter` | N/A | Yes — `MlDsa87Adapter`, `Ed25519Adapter` | No |
| `oya-kms-app` | `app` | N/A | Unrestricted inward | No |

### Port traits declared in kernel

```rust
// oya-kms-envelope-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

#[async_trait::async_trait]
pub trait EnvelopeEncryptionPort: Send + Sync + sealed::Sealed {
    /// Encrypt plaintext with tenant DEK under KMS-managed KEK.
    /// Returns: 12-byte nonce || AES-256-GCM ciphertext || 16-byte tag || CBOR envelope header.
    async fn encrypt(&self, tenant_id: TenantId, plaintext: &[u8])
        -> Result<CipherTextBlob, KmsError>;
    /// Decrypt. Returns zeroed plaintext. Rejects tampered GCM tags.
    async fn decrypt(&self, tenant_id: TenantId, blob: &CipherTextBlob)
        -> Result<ZeroizingBytes, KmsError>;
    /// Re-encrypt envelope headers for KEK rotation (no plaintext re-write).
    async fn rotate_kek(&self, tenant_id: TenantId, old_kek_id: KekId, new_kek_id: KekId)
        -> Result<RotationReport, KmsError>;
}

// oya-kms-keys-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait KmsKeyPort: Send + Sync + sealed::Sealed {
    async fn create_dek(&self, tenant_id: TenantId) -> Result<(DekId, ZeroizingBytes), KmsError>;
    async fn get_dek(&self, tenant_id: TenantId, dek_id: DekId)
        -> Result<ZeroizingBytes, KmsError>;
    async fn rotate_dek(&self, tenant_id: TenantId, dek_id: DekId)
        -> Result<DekId, KmsError>;
    async fn revoke_dek(&self, tenant_id: TenantId, dek_id: DekId) -> Result<(), KmsError>;
}

// oya-kms-signing-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait PqSignerPort: Send + Sync + sealed::Sealed {
    /// ML-DSA-87 (NIST FIPS 204) signing for post-quantum audit segments.
    async fn sign_ml_dsa87(&self, key_id: KeyId, message: &[u8])
        -> Result<MlDsa87Signature, KmsError>;
    async fn verify_ml_dsa87(&self, key_id: KeyId, message: &[u8],
        signature: &MlDsa87Signature) -> Result<bool, KmsError>;
    async fn generate_ml_dsa87_keypair(&self, tenant_id: TenantId)
        -> Result<(KeyId, MlDsa87PublicKey), KmsError>;
}

#[async_trait::async_trait]
pub trait Ed25519SignerPort: Send + Sync + sealed::Sealed {
    async fn sign(&self, key_id: KeyId, preimage: &[u8])
        -> Result<Ed25519Signature, KmsError>;
    async fn verify(&self, key_id: KeyId, preimage: &[u8], sig: &Ed25519Signature)
        -> Result<bool, KmsError>;
}
```

### CI lanes that must green before phase exit gate

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P08-kms` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P08-kms` | exit 0 |
| `port-location` | `oya gate validate port-location --phase P08-kms` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `keys` | `kms` | pending |
| `envelope` | `kms` | pending |
| `signing` | `kms` | pending |

---

## Grit Claim Symbols

```
crates/oya-kms-envelope-kernel/src/ports.rs::EnvelopeEncryptionPort
crates/oya-kms-keys-kernel/src/ports.rs::KmsKeyPort
crates/oya-kms-signing-kernel/src/ports.rs::PqSignerPort
crates/oya-kms-signing-adapter/src/ml_dsa87.rs::MlDsa87Adapter
crates/oya-kms-keys-adapter/src/oci_vault.rs::OciVaultAdapter
migrations/kms/V001__kms_init.sql::kms_schema
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P08-kms started; scope: 16 crates (keys/envelope/signing BCs); envelope encryption AES-256-GCM; ML-DSA-87 post-quantum per ADR-0111; per-tenant DEK isolation" \
  -i high \
  -k "M02,P08,phase-start,kms"

icm store \
  -t context-oyatie \
  -c "Phase P08-kms complete; envelope round-trip green; GCM tamper test green; cross-tenant DEK rejection tested; ML-DSA-87 sign/verify green; key rotation envelope-only verified; next: P09-search" \
  -i high \
  -k "M02,P08,phase-complete,kms"
```

---

## References

- Bominal ADRs inherited: ADR-0111 (CipherText property type + KMS envelope), ADR-0028 (Ed25519 audit signing)
- oyatie ADRs: ADR-0056 (BNF v4.1)
- depends_on: M01-P05, M02-P06-secrets
- unblocks: PHI-encrypted product fields (medical, hr PII), P04-audit-chain ML-DSA-87 upgrade path
