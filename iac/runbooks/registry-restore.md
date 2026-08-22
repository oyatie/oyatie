---
doc_class: Runbook
title: Registry restore (iac-state-index Postgres)
microservice: cloud-iac
severity: "Sev-1 (registry corruption) / Sev-2 (replica lag) / Sev-2 (worker outage)"
status: Accepted
owner_team: axis-cloud-iac + cloud-secrets
date: 2026-05-17
related_artifacts:
  - microservices/cloud-iac/failure-modes.md (FM-03, FM-12)
  - microservices/cloud-iac/incident-response.md
  - microservices/cloud-iac/multi-region.md
doc_status: published
---

# Runbook: Registry restore (iac-state-index Postgres)

## Trigger

ONE of:
1. **Postgres unavailable**: `pg_isready` fails OR cluster-wide write failures.
2. **Replica lag > 60s**: failover risk; may need promote.
3. **Worker outage** (FM-12): registry-worker pods unhealthy; index writes blocked.

## Severity

- Replica lag transient: Sev-3.
- Replica lag persistent > 30min: Sev-2.
- Primary down + failover available: Sev-2.
- Primary + replica down (full registry outage): Sev-1.

## Pre-checks

1. Verify Postgres pod state: `kubectl -n cloud-iac get pods -l app=iac-state-index-pg`.
2. Verify replica lag: `psql -c "SELECT pg_last_wal_receive_lsn(), pg_last_wal_replay_lsn(), now() - pg_last_xact_replay_timestamp() as lag"`.
3. Verify recent backups: list S3 archive bucket for latest WAL + base-backup.
4. Verify worker pod state: `kubectl -n cloud-iac get pods -l app=iac-registry-worker`.

## Recovery Path A — Replica lag transient

| Step | Action |
|---|---|
| 1 | Verify replication slot is alive: `SELECT * FROM pg_replication_slots` |
| 2 | If replica is catching up: monitor; lag should decrease |
| 3 | If lag stuck: investigate network between primary and replica |
| 4 | If lag > 60s and replica is not catching up: prepare promote (Recovery Path B) |

## Recovery Path B — Promote replica to primary

| Step | Action |
|---|---|
| 1 | Declare Sev-2; engage axis-cloud-iac on-call + cloud-secrets |
| 2 | Verify primary is truly unavailable (not split-brain risk) |
| 3 | Promote replica: `pg_ctl promote -D <data-dir>` on the replica |
| 4 | Update connection strings: applier + validator + renderer + registry workers re-point to new primary (via ConfigMap update; HPA-triggered re-roll) |
| 5 | Verify writes resume: `INSERT INTO health_check VALUES (now())` works |
| 6 | Provision new replica from base-backup; resume streaming replication |
| 7 | Postmortem: was the primary failure recoverable, or is the underlying hardware/OCI failure permanent? |

## Recovery Path C — Full restore from S3 archive (PITR)

| Step | Action |
|---|---|
| 1 | Declare Sev-1; engage axis-cloud-iac + cloud-secrets + ExecSponsor |
| 2 | Stop all cloud-iac workers (renderer / validator / applier / rollback / registry); HPA scales to 0 |
| 3 | Identify last-good base-backup: `aws s3 ls s3://cloud-iac-pg-backups/<pack>/base/` |
| 4 | Restore base-backup to new Postgres instance |
| 5 | Replay WAL: `recovery_target_time = '<timestamp>'` per PITR procedure |
| 6 | Verify restore: critical tables (apply_state_index, chart_record, provenance) populated |
| 7 | Re-point workers to restored Postgres |
| 8 | Resume operations; verify apply path end-to-end with a synthetic apply |
| 9 | If any data loss between last WAL replay and incident: investigate which µservices have stale apply-state; force re-derivation via fresh apply |
| 10 | Notify tenants per `incident-response.md` template |

## Recovery Path D — Registry-worker outage (FM-12)

| Step | Action |
|---|---|
| 1 | Inspect worker logs: `kubectl -n cloud-iac logs -l app=iac-registry-worker --tail=200` |
| 2 | Common causes: Postgres connection-pool exhaustion; OOM; OpenBao token-renewal failure |
| 3 | If pool exhausted: increase `max_connections` (with memory bump); restart workers |
| 4 | If OOM: vertical-scale worker pods |
| 5 | If OpenBao: rotate worker service-account token; restart |
| 6 | Verify writes resume: `cloud_iac_registry_writes_total` rate > 0 |

## Verification

After recovery:
- Postgres `pg_isready` returns success on primary + replica.
- Replica lag < 30s.
- Apply path end-to-end works (synthetic apply succeeds).
- iac-state-index row count matches expected (cross-reference against audit-chain seal log).
- Self-SLO dashboard green: `https://grafana-<pack>.oyatie.dev/d/cloud-iac-registry/health`.

## Post-incident updates

- Postmortem within 5 business days (Sev-2+).
- If PITR was used: audit data loss between WAL replay and incident; document re-derivation completeness.
- Verify backup cadence + retention configured per `policy/data-residency.md`.
- Review Postgres capacity per `capacity-model.md`.

## References

- `microservices/cloud-iac/failure-modes.md` FM-03 + FM-12.
- `microservices/cloud-iac/incident-response.md`.
- `microservices/cloud-iac/multi-region.md` §"DR Failover".
- Postgres PITR docs — `postgresql.org/docs/current/continuous-archiving.html`.
- WAL-G — `github.com/wal-g/wal-g` (used for S3 archival; verify version against capacity-model).
