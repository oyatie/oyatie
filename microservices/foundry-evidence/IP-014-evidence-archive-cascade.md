---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-014-evidence-archive-cascade
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-evidence + axis-audit-chain + council-privacy
acceptance_lanes: [archive-cascade-drill, retention-cascade-on-cadence]
---

# IP-014: Evidence archive cascade (hot → warm → cold)

## Intent

Hot→warm→cold archival cascade interlocked with audit-chain substrate retention cascade. Postgres partition pruning + WORM lifecycle policy + substrate `RetentionApplied` consumer.

## ChangeSet boundary

1 new worker crate + Postgres partition DDL + S3 lifecycle Helm values.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-foundry-evidence-archive-cascade-worker/Cargo.toml` | create | edition=2024 |
| `crates/oya-foundry-evidence-archive-cascade-worker/src/lib.rs` | create | daily cascade runner |
| `crates/oya-foundry-evidence-archive-cascade-worker/src/substrate_observer.rs` | create | consume audit-chain `RetentionApplied` to keep Postgres index in sync |
| `microservices/foundry-evidence/iac/helm/postgres/templates/partition-cascade-cronjob.yaml` | create | daily kubernetes CronJob invoking cascade |
| `microservices/foundry-evidence/iac/helm/evidence-blob-store/values.yaml` | edit | S3 lifecycle rules (hot→IA at 90d; IA→Glacier Deep at 1y) |
| `crates/oya-foundry-evidence-archive-cascade-worker/tests/cascade_run_drill.rs` | create | end-to-end drill against test-pack |

## Acceptance Gates

```bash
cargo check -p oya-foundry-evidence-archive-cascade-worker
cargo nextest run -p oya-foundry-evidence-archive-cascade-worker --test cascade_run_drill
helm lint microservices/foundry-evidence/iac/helm/postgres
oya gate validate archive-cascade-drill --microservice foundry-evidence
oya gate validate retention-cascade-on-cadence --microservice foundry-evidence
```

## Halt Conditions

- Cascade runs outside the substrate-interlocked window — block.
- Cold-tier metadata loss (cold_index row missing for an archived blob) — block.
- Cascade attempts to delete a pack with active regulator engagement — block.

## Next IP

[`IP-015-self-observability-slo-wiring.md`](IP-015-self-observability-slo-wiring.md)

## References

- `runbooks/evidence-archive-migration.md`.
- `policy/data-residency.md`.
- `microservices/audit-chain/IP-013-retention-cascade.md`.
