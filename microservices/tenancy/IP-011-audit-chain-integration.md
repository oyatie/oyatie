---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-tenancy-substrate-stable
impl_plan_id: IP-011-audit-chain-integration
status: pending
owner: axis-tenancy + audit-chain
acceptance_lanes: [cargo-nextest, audit-chain-seal-latency-sli]
---

# IP-011: Audit-chain Ed25519 seal integration

## Intent

Every lifecycle event + DSR receipt + RLS policy install + JWT rotation + cell rebalance emits an audit-chain Ed25519 seal per Bominal ADR-0028. This IP wires tenancy crates to the audit-chain µservice; verifies seal latency SLO ≤ 1s p99.

## Concrete File Targets

| Path | Action |
|---|---|
| `oya-tenancy-tenant-lifecycle-adapter/src/audit_chain_sink.rs` | create |
| `oya-tenancy-isolation-policy-adapter/src/audit_chain_sink.rs` | create |
| `oya-tenancy-cell-assignment-adapter/src/audit_chain_sink.rs` | create |
| `oya-tenancy-dsr-cascade-adapter/src/audit_chain_sink.rs` | create |
| Shared utility crate (or shared in each adapter) | create |
| Catalog rows | update — audit-chain dependency declared |

## Code Shape

```rust
// shared audit_chain_sink pattern
pub async fn seal<E: AuditChainEnvelope>(client: &AuditChainClient, envelope: E) -> Result<SealId, ...> {
    let sealed_at = now();
    let signature = client.sign(&envelope).await?;
    let seal = AuditChainSeal { envelope_hash: blake3::hash(&envelope.canonical()), signature, sealed_at, ... };
    client.append(&seal).await?;
    Ok(seal.id)
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-tenancy-tenant-lifecycle-adapter --test audit_chain_seal_latency
```

## Test Plan

- `test_seal_latency_p99_under_1s` — emit 1000 envelopes; verify p99 latency ≤ 1s.
- `test_seal_idempotency` — same envelope sealed twice produces same id.
- Integration test against audit-chain test container.

## Next IP

[`IP-012-branch-protection-and-release-pointers.md`](IP-012-branch-protection-and-release-pointers.md)
