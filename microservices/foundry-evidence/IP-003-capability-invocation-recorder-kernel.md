---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-003-capability-invocation-recorder-kernel
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-evidence
acceptance_lanes: [cargo-clippy, cargo-doc, lean-port-location, lean-layer-correctness]
---

# IP-003: Capability-invocation-recorder kernel crate

## Intent

`oya-foundry-evidence-capability-invocation-recorder-kernel`: port traits + entity types + errors. Layer = `kernel` per ADR-0105 13-layer enum. No project-internal imports.

## ChangeSet boundary

Single Rust crate. Pure types + traits. No I/O.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-foundry-evidence-capability-invocation-recorder-kernel/Cargo.toml` | create | edition=2024; minimal deps (serde, thiserror, ulid, time) |
| `crates/oya-foundry-evidence-capability-invocation-recorder-kernel/src/lib.rs` | create | re-export modules |
| `crates/oya-foundry-evidence-capability-invocation-recorder-kernel/src/entities/invocation_envelope.rs` | create | `InvocationEnvelope` entity per OpenAPI schema |
| `crates/oya-foundry-evidence-capability-invocation-recorder-kernel/src/entities/record_invocation_receipt.rs` | create | `RecordInvocationReceipt` entity |
| `crates/oya-foundry-evidence-capability-invocation-recorder-kernel/src/entities/pack_id.rs` | create | `PackId` strong-typed ULID |
| `crates/oya-foundry-evidence-capability-invocation-recorder-kernel/src/ports/wal.rs` | create | `WalPort` trait: `append_envelope(envelope) -> Result<WalReceipt, WalError>` |
| `crates/oya-foundry-evidence-capability-invocation-recorder-kernel/src/ports/idempotency.rs` | create | `IdempotencyPort` trait: `dedup_check(idempotency_key, tenant_id) -> Result<DedupVerdict, IdempotencyError>` |
| `crates/oya-foundry-evidence-capability-invocation-recorder-kernel/src/ports/pack_builder_enqueue.rs` | create | `PackBuilderEnqueuePort` trait: `enqueue(envelope) -> Result<EnqueueReceipt, EnqueueError>` |
| `crates/oya-foundry-evidence-capability-invocation-recorder-kernel/src/errors.rs` | create | `RecorderError` enum with `#[non_exhaustive]` |
| `crates/oya-foundry-evidence-capability-invocation-recorder-kernel/Cargo.toml` workspace | edit | register in workspace |
| `Cargo.toml` (workspace) | edit | add crate |

## Acceptance Gates

```bash
cargo check -p oya-foundry-evidence-capability-invocation-recorder-kernel
cargo clippy -p oya-foundry-evidence-capability-invocation-recorder-kernel -- -D warnings
cargo doc -p oya-foundry-evidence-capability-invocation-recorder-kernel --no-deps
cargo run -p oya-dev-cli -- gate validate port-location --microservice foundry-evidence
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice foundry-evidence
```

## Halt Conditions

- Any project-internal import in this crate — block (kernel layer invariant).
- `PackId` not strong-typed — block.

## Next IP

[`IP-004-evidence-pack-builder-kernel.md`](IP-004-evidence-pack-builder-kernel.md)

## References

- ADR-0105 (13-layer enum).
- ADR-0056 (BNF v4.1 + clean architecture).
