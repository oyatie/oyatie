---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-network-foundation
impl_plan_id: IP-007-endorsement-engine-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-network + axis-audit-chain
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-endorsement-chain-integrity, oya-governance-port-location]
---

# IP-007: endorsement-engine BC end-to-end (per-endorser Ed25519 + Merkle-style chain via audit-chain)

## Intent

Author the full `endorsement-engine` BC per ADR-NET-0005:

- Skill endorsements (per-skill 1-click) with per-endorser Ed25519 signature.
- Long-form recommendations with attribution + relationship metadata.
- Per-tenant Merkle tree sealed to audit-chain at batch boundary.
- Revocation tombstone semantics (body-wiped; chain intact).
- KMS-bound Ed25519 keypair per endorser (signing via KMS Sign API; private key never exfiltrates).
- Per-tenant advisory lock on Merkle position assignment per ADR-NET-0001.

## Code Shape

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait EndorsementRepository: Send + Sync {
    async fn add(&self, endorsement: EndorsementNew) -> Result<Endorsement, EndorsementError>;
    async fn revoke(&self, tenant_id: &TenantId, endorsement_id: &EndorsementId, reason: &str) -> Result<(), EndorsementError>;
    async fn verify_chain(&self, tenant_id: &TenantId, partition: &Partition) -> Result<ChainVerifyResult, EndorsementError>;
}

#[async_trait]
pub trait EndorsementSignatureSigner: Send + Sync {
    async fn sign(&self, body: &[u8], endorser_key_ref: &KmsKeyRef) -> Result<Ed25519Signature, SigningError>;
    async fn verify(&self, body: &[u8], signature: &Ed25519Signature, endorser_public_key: &Ed25519PublicKey) -> Result<(), SigningError>;
}

#[async_trait]
pub trait MerkleChainSealer: Send + Sync {
    async fn seal_root(&self, tenant_id: &TenantId, partition: &Partition, root_hash: &Hash, position: u64) -> Result<AuditChainSealId, SealError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-network-endorsement-engine-kernel
cargo nextest run -p oya-network-endorsement-engine-adapter-postgres
cargo run -p oya-dev-cli -- gate validate endorsement-chain-integrity --microservice network
cargo run -p oya-dev-cli -- gate validate port-location --microservice network
```

## Test Plan

- Synthetic 10k-endorsement chain: replay produces matching Merkle root.
- Injected forgery: signature verification fails; chain verify catches mismatch.
- Revocation: aggregated counts respect revocation (revoked endorsements do not count).
- GDPR Art. 17 erasure: body wiped; chain link intact; verifiability preserved.
- KMS audit log shows no unauthorised key access; rotation respected.
- Per-tenant advisory lock under concurrent endorsement burst: Merkle positions assigned without collision.

## Halt Conditions

- Chain verify-failure rate > 0 — fix; never ship.
- KMS rotation breaks historical verification — investigate KMS public-key retention 7y.

## Next IP

[`IP-008-skill-assessments-and-profile-verification-bcs.md`](IP-008-skill-assessments-and-profile-verification-bcs.md)

## References

- ADR-NET-0005 (endorsement-chain integrity).
- Bominal ADR-0028 (audit-chain).
- RFC 8032 (Ed25519); RFC 6962 (Merkle tree pattern).
- eIDAS 910/2014 (AdES alignment).
