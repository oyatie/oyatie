---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-022-audit-tap-merkle-seal
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
related_adrs: [ADR-0263, ADR-0255, ADR-0028]
---

# IP-022: Audit-tap — Merkle seal + Ed25519 signing

## Intent

Every call record emitted by `oya-intelligence-audit-tap-usecase` MUST be Ed25519-signed by
the per-µservice signing key (held in OpenBao sidecar, ADR-0296) and Merkle-sealed into the
audit-chain per ADR-0028. This IP wires the signing + sealing path.

## Concrete file targets

| Path | Action |
|---|---|
| `crates/oya-intelligence-audit-tap-usecase/src/signer.rs` | create |
| `crates/oya-intelligence-audit-tap-usecase/src/merkle_sealer.rs` | create |
| `crates/oya-intelligence-audit-tap-adapter/src/openbao_signing_key.rs` | create |
| `crates/oya-intelligence-audit-tap-worker/src/seal_worker.rs` | create |

## Code shape

```rust
pub struct AuditTapSigner {
    key_handle: Arc<dyn SigningKeyPort>,  // OpenBao sidecar via ADR-0296
}

impl AuditTapSigner {
    pub async fn sign(&self, record: &CallRecord) -> Result<SignedCallRecord, AuditError>;
}

pub struct MerkleSealer {
    chain_client: Arc<dyn AuditChainPort>,
}

impl MerkleSealer {
    pub async fn seal(&self, signed: SignedCallRecord) -> Result<SealReceipt, AuditError>;
}
```

## Key implementation notes

- Signing key rotated by OpenBao; sidecar exposes current signing key via Unix socket.
- Merkle-sealed: batch of records every 5 s → Merkle root → appended to audit-chain.
- In-µservice forgery: impossible — signing key never leaves sidecar; audit-chain is append-only.
- Failure mode: if seal fails → dispatch returns `AuditTapEmitFailed`; no provider call proceeds.

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-audit-tap-usecase -- signer merkle_sealer
cargo run -p oya-dev-cli -- gate validate audit-tap-atomicity --microservice intelligence
cargo run -p oya-dev-cli -- gate validate audit-chain-seal-integrity --microservice intelligence
```

## References

- ADR-0263 (audit-tap emission contract).
- ADR-0028 (audit-chain seal).
- ADR-0296 (sidecar credential handle — signing key isolation).
- `microservices/intelligence/runbooks/audit-row-forgery-detected.md`.

## Wave 15 substance conversion — signed Merkle evidence

### §A Problem

An audit record that is merely written to a local table can be forged, dropped, or reordered.
This IP closes the integrity gap by signing each intelligence call record and sealing batches into the audit-chain.

### §B Approach

Use the OpenBao sidecar for the per-service signing key and an audit-chain client for Merkle append.
Dispatch fails closed if signing or sealing cannot complete for high-risk calls.

### §C Deliverables

- `crates/oya-intelligence-audit-tap-usecase/src/signer.rs`
- `merkle_sealer.rs`, `openbao_signing_key.rs`, and `seal_worker.rs`
- tests for signature validation, batch root stability, and seal failure

### §D Implementation

1. Canonicalize `CallRecord` bytes before signing.
2. Request Ed25519 signatures through the sidecar; never load keys in process memory.
3. Batch signed records into five-second Merkle roots.
4. Append roots to audit-chain and store returned seal receipts.
5. Return `AuditTapEmitFailed` before provider call when audit cannot proceed.
6. Link failure response to `runbooks/audit-row-forgery-detected.md`.

### §E Acceptance

The audit atomicity and seal integrity gates must prove tamper detection, missing-seal refusal, and sidecar-key
isolation.

### §F Evidence

Local anchors: `runbooks/audit-row-forgery-detected.md`, `slos/audit-emission-success.openslo.yaml`, ADR-0263,
ADR-0028, and ADR-0296.

### §G Counterparts

CloudTrail, Azure Monitor, OpenAI usage records, and Anthropic organization logs provide provider-side evidence;
oyatie closes the stronger evidence gap with per-call Ed25519 signatures and Merkle sealing.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-022-audit-tap-merkle-seal.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
