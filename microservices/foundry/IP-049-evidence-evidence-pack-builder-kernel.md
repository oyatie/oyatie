---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-004-evidence-pack-builder-kernel
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-evidence
acceptance_lanes: [cargo-clippy, cargo-doc, lean-port-location, lean-layer-correctness]
---

# IP-004: Evidence-pack-builder kernel crate

## Intent

`oya-foundry-evidence-evidence-pack-builder-kernel`: port traits for SignalSource (runtime/eval/guardrails/supervisor) + AuditChainBridge + Postgres index writer + S3 blob staging. Layer = `kernel`.

## ChangeSet boundary

Single Rust crate. Pure types + traits.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/Cargo.toml` | create | edition=2024 |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/lib.rs` | create | re-export |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/entities/evidence_pack.rs` | create | `EvidencePack` entity (canonical schema) |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/entities/eval_verdict_at_invocation.rs` | create | join entity |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/entities/guardrail_decision.rs` | create | per-decision entity |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/entities/autonomy_tier_decision.rs` | create | T0..T3 + rationale_hash |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/ports/runtime_signal_source.rs` | create | `RuntimeSignalSourcePort` trait |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/ports/eval_signal_source.rs` | create | `EvalSignalSourcePort` trait |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/ports/guardrails_signal_source.rs` | create | `GuardrailsSignalSourcePort` trait |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/ports/supervisor_signal_source.rs` | create | `SupervisorSignalSourcePort` trait |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/ports/audit_chain_bridge.rs` | create | `AuditChainBridgePort` trait: `emit(pack) -> Result<AuditEventId, BridgeError>` |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/ports/evidence_index_writer.rs` | create | `EvidenceIndexWriterPort` trait |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/ports/dead_letter_store.rs` | create | `DeadLetterStorePort` trait |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/errors.rs` | create | `PackBuilderError` |
| `Cargo.toml` (workspace) | edit | register |

## Acceptance Gates

```bash
cargo check -p oya-foundry-evidence-evidence-pack-builder-kernel
cargo clippy -p oya-foundry-evidence-evidence-pack-builder-kernel -- -D warnings
cargo doc -p oya-foundry-evidence-evidence-pack-builder-kernel --no-deps
cargo run -p oya-dev-cli -- gate validate port-location --microservice foundry-evidence
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice foundry-evidence
```

## Halt Conditions

- Any project-internal import — block.
- `AuditChainBridgePort` exposes substrate-internal types (e.g., Merkle proofs) — block; substrate types only enter via SDK re-exports in adapter layer.

## Next IP

[`IP-005-evidence-pack-builder-domain.md`](IP-005-evidence-pack-builder-domain.md)

## References

- ADR-0105 + ADR-0056.
- ADR-0131 §"Substrate split" — kernel never depends on substrate internals.
