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

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: Audit-chain Ed25519 seal integration

## Intent

Every lifecycle event + DSR receipt + RLS policy install + JWT rotation + cell rebalance emits an audit-chain Ed25519 seal per Bominal ADR-0028. This IP wires tenancy crates to the audit-chain µservice; verifies seal latency SLO ≤ 1s p99.

## Concrete File Targets

| Path | Action |
|---|---|
| `tenancy-tenant-lifecycle-adapter/src/audit_chain_sink.rs` | create |
| `tenancy-isolation-policy-adapter/src/audit_chain_sink.rs` | create |
| `tenancy-cell-assignment-adapter/src/audit_chain_sink.rs` | create |
| `tenancy-dsr-cascade-adapter/src/audit_chain_sink.rs` | create |
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
cargo nextest run -p tenancy-tenant-lifecycle-adapter --test audit_chain_seal_latency
```

## Test Plan

- `test_seal_latency_p99_under_1s` — emit 1000 envelopes; verify p99 latency ≤ 1s.
- `test_seal_idempotency` — same envelope sealed twice produces same id.
- Integration test against audit-chain test container.


## DR posture (per ADR-0343)
- Manifest target source: `microservices/tenancy/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/tenancy/IP-011-audit-chain-integration.md` matched `p99, SLO`; anchors `microservices/tenancy/runbooks/dr-pair-promotion-drill.md, crates/tenancy-api/src/lib.rs`; type anchor `crates/tenancy-api/src/lib.rs::TenantCreateApiRequest`.

## Next IP

[`IP-012-branch-protection-and-release-pointers.md`](IP-012-branch-protection-and-release-pointers.md)
