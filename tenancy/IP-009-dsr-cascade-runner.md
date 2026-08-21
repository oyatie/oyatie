---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-tenancy-substrate-stable
impl_plan_id: IP-009-dsr-cascade-runner
status: in-progress
owner: axis-tenancy + council-privacy
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-dsr-handler-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: DSR cascade runner + proof-of-erasure

## Intent

> **Delivery note (2026-08-20).** Implemented in tenancy/core/dsr-cascade as `tenancy-dsr-cascade`, collapsed into that ONE crate
> as a module tree rather than this plan's multi-crate fan-out: the capability is capped at 12 crates
> and `Cargo.lock` is a hub path owned by `integ/build`, so neither a new crate nor a new dependency
> was available to this lane. Landed: the cascade plan, idempotent fan-out, Merkle aggregation over an in-crate NIST-pinned SHA-256, and SLA tracking. Deferred and named as a gap in the crate's `lib.rs` header:
> the REST surface, the SLA timer worker, Postgres persistence, and a signature over the proof. The crate names in the tables below are this plan's original
> proposal, not what shipped.


`oya-tenancy-dsr-cascade-{kernel,domain,usecase,adapter,rest,worker,app}` crates: DSR ingestion; cross-µservice Workflow fan-out (`TenantDeletionRequested`); per-µservice receipt aggregation; Merkle root computation; proof-of-erasure certificate generation; per-pack legal-SLA timer.

## Concrete File Targets

| Path | Action |
|---|---|
| `oya-tenancy-dsr-cascade-kernel/` | create — DsrRequest, ErasureReceipt, ProofOfErasure entities + ports |
| `oya-tenancy-dsr-cascade-domain/` | create — Merkle aggregation; per-pack SLA enum |
| `oya-tenancy-dsr-cascade-usecase/` | create — submit / aggregate / complete orchestrators |
| `oya-tenancy-dsr-cascade-adapter/` | create — Workflow fan-out + audit-chain proof signer |
| `oya-tenancy-dsr-cascade-rest/` | create — `POST /dsr-requests`, `GET /dsr-requests/{id}`, `GET /dsr-requests/{id}/proof-of-erasure` |
| `oya-tenancy-dsr-cascade-worker/` | create — cascade orchestrator; SLA timer; missing-receipt escalation |
| Catalog rows | create — 7 entries |

## Code Shape

```rust
// domain/src/merkle.rs
pub fn compute_proof_of_erasure(receipts: &[ErasureReceipt]) -> ProofOfErasure {
    let leaves: Vec<[u8; 32]> = receipts.iter().map(|r| blake3::hash(&r.canonical_serialized()).into()).collect();
    let root = merkle_root(&leaves);
    ProofOfErasure {
        merkle_root: hex::encode(root),
        microservices_count: receipts.len() as i32,
        receipts: receipts.to_vec(),
        sealed_at: now(),
        ..
    }
}
```

```rust
// domain/src/per_pack_sla.rs
pub fn sla_deadline(pack: Pack, requested_at: DateTime<Utc>) -> DateTime<Utc> {
    match pack {
        Pack::Kr | Pack::Eu | Pack::In => requested_at + Duration::days(30),
        Pack::Br                       => requested_at + Duration::days(15),
        Pack::UsHc                     => requested_at + Duration::days(7),  // per BAA
        _                              => requested_at + Duration::days(30),
    }
}
```

```rust
// worker/src/cascade.rs
pub async fn run_cascade(deps: &Deps, dsr: &DsrRequest) -> anyhow::Result<()> {
    let expected_microservices = deps.microservice_registry.list_active().await?;
    deps.dsr_repo.set_receipts_expected(&dsr.dsr_id, expected_microservices.len() as i32).await?;
    deps.event_sink.emit(TenantDeletionRequestedEvent::from(dsr)).await?;
    loop {
        let received = deps.receipt_repo.count(&dsr.dsr_id).await?;
        if received == expected_microservices.len() as i32 {
            let receipts = deps.receipt_repo.list(&dsr.dsr_id).await?;
            let proof = compute_proof_of_erasure(&receipts);
            deps.audit_chain.seal_proof_of_erasure(&proof).await?;
            deps.event_sink.emit(TenantDeletionCompletedEvent::from(&proof)).await?;
            return Ok(());
        }
        let elapsed = now() - dsr.requested_at;
        let sla = sla_deadline(dsr.pack, dsr.requested_at);
        if (sla - now()) < Duration::days(7) {  // 7d before deadline
            deps.alerter.alert_dpo_sla_at_risk(&dsr.dsr_id, &expected_microservices, &received).await?;
        }
        tokio::time::sleep(Duration::minutes(5)).await;
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-tenancy-dsr-cascade-worker --test dsr_cascade_proof
cargo run -p oya-dev-cli -- gate validate dsr-handler-conformance
```

## Test Plan

- `test_merkle_root_deterministic` — same receipts → same root.
- `test_per_pack_sla_correct` — pack-eu 30d, pack-br 15d, pack-us-hc 7d.
- `test_cascade_aggregates_all_receipts` — N receipts → proof-of-erasure.
- `test_sla_at_risk_alert` — at 80% window, DPO alerted.
- `test_missing_receipt_halts_proof` — proof not emitted until all receipts in OR DPO 2-person-rule override.

## Halt Conditions

- Any µservice missing DSR handler at catalog registration time — refuse merge of that µservice.
- Proof-of-erasure emitted with `received_n < expected_n` AND no DPO override — refuse.

## Next IP

[`IP-010-tenancy-rest-and-sdk.md`](IP-010-tenancy-rest-and-sdk.md)
