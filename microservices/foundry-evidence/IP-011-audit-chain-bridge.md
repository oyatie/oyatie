---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-011-audit-chain-bridge
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-evidence + axis-audit-chain
acceptance_lanes: [cargo-clippy, lean-cross-microservice-import-forbidden, audit-chain-bridge-availability-drill]
---

# IP-011: audit-chain bridge (the only cross-µservice consumer)

## Intent

Implement `oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge` as the SOLE adapter that consumes `oya-audit-chain-emission-sdk`. Encapsulates retry policy, back-off, dead-letter integration, SPIFFE identity binding. Mirror exists for regulator-export bundle-seal path.

## ChangeSet boundary

1 adapter crate (logically; physically already created in IP-006 but this IP delivers the substantive bridge implementation including retry + back-off + dead-letter contract). Plus the regulator-export bundle-seal bridge subset.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge/src/lib.rs` | edit | implement `AuditChainBridgePort` |
| `crates/oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge/src/retry.rs` | create | exponential back-off with jitter; bounded retry budget |
| `crates/oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge/src/dead_letter.rs` | create | enqueue to dead-letter on persistent failure; emit `foundry.evidence.pack.assembly_failed.v1` on retry exhaustion |
| `crates/oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge/src/spiffe.rs` | create | SPIFFE workload-identity binding |
| `crates/oya-foundry-evidence-regulator-export-adapter/src/audit_chain_bundle_seal_bridge.rs` | create | bundle-seal path that asks substrate to mint a bundle root from the per-pack tree |
| `crates/oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge/tests/availability_drill.rs` | create | drill: substrate down → bridge holds; substrate recovers → bridge drains |

## Acceptance Gates

```bash
cargo check -p oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge
cargo nextest run -p oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge --test availability_drill
oya gate validate cross-microservice-import-forbidden --microservice foundry-evidence
oya gate validate audit-chain-bridge-availability-drill --microservice foundry-evidence
```

## Halt Conditions

- Any crate other than this bridge or the bundle-seal bridge subset imports `oya-audit-chain-*` — block.
- Retry budget unbounded — block (DOS risk on the substrate).
- Dead-letter contract missing emit on permanent failure — block (silent loss is forbidden).

## Next IP

[`IP-012-sdk-cross-microservice.md`](IP-012-sdk-cross-microservice.md)

## References

- ADR-0131 §"Substrate split".
- `microservices/audit-chain/contracts/openapi/audit-emit.yaml`.
- `microservices/audit-chain/IP-014-cross-microservice-emission-adapter.md` (substrate-side pattern).
