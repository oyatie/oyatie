---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-009-kernel-audit-tap
status: pending
owner: axis-intelligence + axis-audit-chain
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
---

# IP-009: Kernel — audit-tap port traits

## Intent

`oya-intelligence-audit-tap-kernel`: port traits for audit-tap emission + Ed25519 signing.

## Concrete file targets

| Path | Action |
|---|---|
| `.../oya-intelligence-audit-tap-kernel/Cargo.toml` | create |
| `.../oya-intelligence-audit-tap-kernel/src/lib.rs` | create |
| `.../oya-intelligence-audit-tap-kernel/src/audit_tap_port.rs` | create |
| `.../oya-intelligence-audit-tap-kernel/src/signer_port.rs` | create |
| `.../oya-intelligence-audit-tap-kernel/src/audit_tap_record.rs` | create |

## Code shape

```rust
pub struct AuditTapRecord {
    pub envelope_id: Ulid,
    pub tenant_id: TenantId,
    pub audience: Audience,
    pub modality: Modality,
    pub provider: Provider,
    pub model: ModelId,
    pub prompt_hash: Sha256Hash,
    pub output_hash: Option<Sha256Hash>,
    pub refusal: Option<RefusalDecision>,
    pub cost: CostRecord,
    pub emitted_at: SystemTime,
}

#[async_trait]
pub trait AuditTapPort: Send + Sync + 'static {
    /// Atomic commit: must complete (or fail) before dispatch returns to caller.
    async fn commit(&self, record: AuditTapRecord) -> Result<AuditTapReceipt, AuditTapError>;
}

#[async_trait]
pub trait SignerPort: Send + Sync + 'static {
    async fn sign(&self, bytes: &[u8]) -> Result<Ed25519Signature, SignerError>;
}
```

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-audit-tap-kernel
```

## Next IP

[`IP-010-usecase-dispatch-flow.md`](IP-010-usecase-dispatch-flow.md)

## References

- ADR-0263.
- `microservices/intelligence/threat-model.md` T-S-03.

## Wave 15 substance conversion — audit tap kernel

### §A Problem

The architecture requires every model call to emit audit evidence, but provider adapters need a kernel port rather
than direct audit-chain coupling.
This IP closes the audit-first seam that blocks unaudited dispatch.

### §B Approach

Define audit tap port traits and canonical call record structs in `oya-intelligence-audit-tap-kernel`.
Usecases depend on the port; the adapter/worker seals records later through IP-022.

### §C Deliverables

- `crates/oya-intelligence-audit-tap-kernel/src/call_record.rs`
- `audit_tap_port.rs`, `seal_receipt.rs`, and `audit_error.rs`
- tests for required tenant/provider/audience fields

### §D Implementation

1. Require tenant id, audience tag, envelope id, provider, model, and routing decision id.
2. Store prompt/completion hashes rather than raw content.
3. Represent refusal, success, timeout, and provider saturation terminal states.
4. Make call-record creation happen before provider execution.
5. Return typed errors that dispatch can fail closed on.
6. Keep Merkle and signing mechanics in IP-022.

### §E Acceptance

Nextest must prove required-field validation and that dispatch code cannot construct a success record without
tenant and provider metadata.

### §F Evidence

Local anchors: `slos/audit-emission-success.openslo.yaml`, `runbooks/audit-row-forgery-detected.md`, ADR-0263.

### §G Counterparts

CloudTrail, Azure Monitor, OpenAI usage records, and Anthropic organization logs provide vendor evidence; oyatie
closes the product gap by requiring per-call sealed records independent of provider.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-009-kernel-audit-tap.md` matched `cost, emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
