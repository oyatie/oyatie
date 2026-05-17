---
doc_class: ImplementationPlan
impl_plan_id: IP-013-retention-cascade
status: pending
owner: council-privacy + axis-audit-chain
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, layer-correctness, retention-conformance]
---

# IP-013: retention-cascade BC (6 crates)

## Intent

Full retention-cascade BC: kernel + domain + usecase + api + adapter + worker. Per-pack retention sweep (daily) + DSR cascade event consumption + soft-delete with chain-preserving Merkle proof of redaction + hard-delete after grace.

## Crates introduced (6)

- `oya-audit-chain-retention-cascade-kernel`
- `oya-audit-chain-retention-cascade-domain`
- `oya-audit-chain-retention-cascade-usecase`
- `oya-audit-chain-retention-cascade-api`
- `oya-audit-chain-retention-cascade-adapter`
- `oya-audit-chain-retention-cascade-worker`

## Policy artifact

| Path | Action |
|---|---|
| `microservices/audit-chain/policy/retention-matrix.yaml` | create — per-pack × per-data-class retention windows; per `policy/data-residency.md` §"Retention" matrix |

## Code Shape

```rust
// retention-cascade-worker/src/main.rs (daily sweep loop)
loop {
    for tenant_partition in tenant_partitions(&pack).await? {
        for data_class in DATA_CLASSES {
            let retention_window = retention_matrix.lookup(&pack, &data_class, &tenant_partition.tenant_scope);
            let candidate_events = pg.scan_old_events(&tenant_partition, &data_class, retention_window).await?;
            for event in candidate_events {
                if event.in_grace_window() {
                    // already soft-deleted; check hard-delete eligibility
                    if event.grace_expired() {
                        hard_delete_payload(&event).await?;  // emit RetentionApplied{mode=hard_delete}
                    }
                } else {
                    soft_delete(&event).await?;  // emit RetentionApplied{mode=soft_delete}
                }
            }
        }
    }
    tokio::time::sleep(Duration::from_hours(24)).await;
}
```

DSR cascade consumer:

```rust
// consume DataSubjectRequestRaised event
async fn handle_dsr(event: DataSubjectRequestRaised) -> Result<()> {
    let affected = pg.scan_events_for_subject(&event.tenant_partition, &event.subject_hash).await?;
    for affected_event in affected {
        if statutory_retention_locked(&affected_event, &event.request_type) {
            mark_for_retention_expiry_redaction(&affected_event).await?;
        } else {
            soft_delete(&affected_event).await?;
        }
    }
    send_dsr_receipt(&event.dsr_id).await?;
    Ok(())
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-audit-chain-retention-cascade-worker --features integration
cargo run -p oya-dev-cli -- gate validate retention-conformance --microservice audit-chain
```

## End-to-end drill

```bash
# DSR cascade drill
cargo nextest run --test dsr_cascade_drill -p oya-audit-chain-retention-cascade-usecase
# Inject DSR; verify all target events soft-deleted within 5 min; Merkle proof of redaction preserved
```

## References

- Bominal ADR-0028 §"Right-to-erasure with chain preservation".
- `microservices/audit-chain/policy/data-residency.md` §"DSR Cascade".
- `microservices/audit-chain/runbooks/retention-cascade.md`.
- GDPR Art. 17 + recital 65.
