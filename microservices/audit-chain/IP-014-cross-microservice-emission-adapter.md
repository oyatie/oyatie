---
doc_class: ImplementationPlan
impl_plan_id: IP-014-cross-microservice-emission-adapter
status: pending
owner: axis-audit-chain + axis-tenancy + axis-observability
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a2]
---

# IP-014: Cross-µservice emission SDK adoption

## Intent

Standardise `AuditEmitter` SDK consumption across every other oyatie µservice. Demonstrate integration via two reference adopters:
1. `tenancy` µservice emits `TenantOnboarded`, `TenantOffboarded`, `DataSubjectRequestRaised` via the SDK.
2. `observability` µservice emits `EligibilityChanged`, `PromotionExecuted`, `RollbackExecuted` via the SDK.

## ChangeSet boundary

This IP does NOT introduce new audit-chain crates. It introduces:
- `oya-audit-chain-emission-sdk` adoption in `tenancy` and `observability` µservices' Cargo.toml (workspace-internal dep).
- Conventions doc at `docs/standards/audit-chain-emission.md` (cross-cutting; Slice D).
- Migration template for every other µservice to follow when migrating to per-microservice-flat-layout.

## Concrete File Targets

| Path | Action |
|---|---|
| `crates/oya-tenancy-*/Cargo.toml` | update — add `oya-audit-chain-emission-sdk` dep |
| Tenancy emission integration | update — wire AuditEmitter into TenantOnboarded handler etc. |
| `crates/oya-observability-slo-engine-*/Cargo.toml` | update — same |
| Observability emission integration | update — wire AuditEmitter into EligibilityChanged emitter |
| `docs/standards/audit-chain-emission.md` | create — cross-µservice convention; what to emit; payload-class declaration; SPIFFE-binding setup; idempotency-key construction |

## Cross-product check

This is the rare cross-product change. Per `feedback_workflow_objectgraph_adapter_layer.md`:
- `tenancy` + `observability` import `oya-audit-chain-emission-sdk` (a Workflow-aligned cross-µservice contract; the SDK is the adapter).
- They do NOT import any other audit-chain crate at any layer.
- LEAN-A2 verifies.

## Acceptance Gates

```bash
cargo nextest run --workspace
cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice tenancy
cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice observability
cargo run -p oya-dev-cli -- gate validate audit-chain-emission-adoption
```

## References

- Bominal ADR-0003 §"SDK contract".
- `microservices/audit-chain/sdk-plan.md`.
- `docs/standards/audit-chain-emission.md` (this IP's deliverable).
