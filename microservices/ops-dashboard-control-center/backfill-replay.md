---
doc_class: Backfill-Replay
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0276
  - ADR-0248
companion_docs:
  - microservices/ops-dashboard-control-center/multi-region.md
  - microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md
  - microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md
planned_enforcement_ref: oya-governance-microservice-doc-suite
---

# Backfill and Replay — ops-dashboard-control-center

## §1 Backfill scenarios

| Scenario | Trigger | Procedure |
|---|---|---|
| Cell outage — missed audit events | Cell unreachable; outbox accumulated events | On cell recovery, outbox drains automatically via Kafka consumer; audit events backfilled in order |
| Manual handoff fallback record | ops-dashboard unavailable during on-call handoff | `POST /ops/v1/oncall/handoffs/backfill` with `source: MANUAL_FALLBACK`; backdated HLC timestamp |
| Cross-region audit log backfill | Replication lag exceeded SLO; remote cell has stale log | Kafka MirrorMaker 2 replication catches up automatically; staleness bounded by SLO ≤5s lag |
| Forensic evidence backfill | Regulator requests historical period | Evidence export via `runbooks/forensic-investigation-handoff.md §step-1`; ClickHouse cold-tier covers 7yr |

## §2 Replay capability

Audit chain replay for post-incident analysis:
- `GET /ops/v1/audit/replay?from=<ISO8601>&to=<ISO8601>&principal=<id>` — ordered replay of audit events.
- Events are immutable (append-only ClickHouse); replay is read-only.
- Merkle chain verification included in replay response — tamper detection on replay.
- Per-tenant scope enforced on replay (auditor principals scoped to `scoped_tenants`).

## §3 Portability

Per ADR-0276: per-tenant backup export as signed JSONL zstd archive.

Export command:
```
POST /ops/v1/audit/evidence-export
Body: { "tenant_ids": [...], "time_range": {...}, "export_format": "signed-jsonl-zstd" }
```

Archive contents:
- `events.jsonl.zstd` — audit events in chronological order.
- `manifest.json` — SHA-256 hash + event count + time range + seal ref.
- `chain.sig` — cosign keyless OIDC signature over manifest SHA-256.

Restoration test: quarterly per `compliance.md §key-rotation-cadence`. Verifiable offline: `cosign verify-blob --cert chain.sig events.jsonl.zstd`.

## §4 Retention

| Data class | Retention | Storage tier |
|---|---|---|
| AUDIT events | 7yr | ClickHouse: hot (0-90d), warm (90d-1yr), cold (1yr-7yr) |
| Session recordings (T3) | 1yr | OpenBao-encrypted block storage → cold tier |
| Session recordings (T2) | 90d | OpenBao-encrypted block storage → purged |
| Cluster health signals | 1yr | ClickHouse warm tier |
| Incident records | 7yr | Postgres → ClickHouse archive |
