---
doc_class: Runbook
template_id: TPL-RUNBOOK
title: Hard-Delete Tombstone Corruption
microservice: anonymous
severity: "Sev-1 (any tombstone-correctness regression is P0)"
status: Accepted
owner_team: axis-anonymous + ops-data + council-privacy
date: 2026-05-17
related_adrs: [ADR-ANON-0004]
related_artifacts:
  - microservices/anonymous/PRD.md I3
  - microservices/anonymous/failure-modes.md FM-11
  - microservices/anonymous/slos/hard-delete-propagation-correctness.openslo.yaml
doc_status: published
---

# Runbook: Hard-Delete Tombstone Corruption

## Trigger

| Signal | Notes |
|---|---|
| `anonymous_hard_delete_propagation_correctness` SLO breach (target = 100%, any breach is Sev-1) | regulatory exposure: GDPR Art. 17 right-to-erasure |
| Audit-chain tombstone Merkle proof verification fails | chain-of-custody compromised |
| Read path returns a record that has a tombstone | propagation incomplete |
| Tombstone with no matching record (orphan tombstone) | corruption |
| Tombstone count diverges from delete-event count | replay regression |

## Severity

- **Sev-1 always.** A tombstone-correctness regression is a privacy regression. There is no "Sev-2 tombstone corruption."

## Pre-checks

1. Confirm the regression signal: query Mimir `anonymous_hard_delete_propagation_correctness` and `anonymous_tombstone_orphan_total`.
2. Identify the regressed shard / cell / pack.
3. Capture the audit-chain Merkle root snapshot at the time of detection (do NOT advance the chain).
4. Pause the retention-policy worker on the affected shard: `cargo run -p oya-dev-cli -- anonymous retention pause-worker --shard <id>`

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Declare Sev-1; assemble IC, ops-data, council-privacy | ≤ 5 min |
| 2 | Pause retention worker on affected shard (above) | ≤ 1 min |
| 3 | Snapshot the Postgres state of affected shard: `pg_dump --snapshot --shard <id>` to `evidence/hard-delete-tombstone-<incident-id>/snapshot.sql.gz` | ≤ 10 min |
| 4 | Reconcile tombstones vs records: `cargo run -p oya-dev-cli -- anonymous retention reconcile --shard <id> --emit-report` | ≤ 30 min |
| 5 | Categorise the corruption: (a) orphan tombstone (record was never present); (b) missing tombstone (record was hard-deleted but no tombstone); (c) propagation-incomplete (tombstone exists but read path returns record); (d) audit-chain seal mismatch | ≤ 15 min |
| 6a (orphan tombstone) | Investigate replay: was a tombstone replayed without corresponding record? Audit-chain seal `OrphanTombstoneObserved`; do NOT remove the tombstone (defence: orphan tombstone is harmless) | – |
| 6b (missing tombstone) | This is a privacy regression. Identify the deleted records via audit-chain `HardDeleteJobCompleted` events; verify the records are deleted from every read path; if records still exist, force re-delete + tombstone | within 1h |
| 6c (propagation-incomplete) | Force propagation: `cargo run -p oya-dev-cli -- anonymous retention propagate --tombstone-id <id>`. Verify Postgres + Valkey + Meilisearch + audit-chain all reflect the deletion within p99 ≤ 5s | within 30 min |
| 6d (audit-chain seal mismatch) | Engage audit-chain ops; the Merkle root must be reconciled; if reconciliation fails, the audit-chain is compromised and a Sev-0 (cross-µservice) incident must be declared | within 2h |
| 7 | Verify post-fix: `cargo run -p oya-dev-cli -- anonymous retention verify --shard <id>` returns green | ≤ 30 min |
| 8 | Resume retention worker | ≤ 1 min |
| 9 | Notify affected users (if missing-tombstone category) per `incident-response.md` legal-notification template; report to relevant DPA (GDPR Art. 33 within 72h if applicable) | within 72h |
| 10 | Post-mortem within 5 business days | – |

## Regulatory escalation matrix

| Category | DPA notification required? | Tenant notification required? |
|---|---|---|
| Orphan tombstone (harmless) | No (no privacy regression) | No |
| Missing tombstone (deletion didn't propagate fully) | Yes if affected records contained PII (GDPR Art. 33; KR PIPA Art. 34; UK ICO; APPI; etc.) | Yes |
| Propagation-incomplete (transient; recovered within 5s) | No (within SLO bound) | No |
| Audit-chain seal mismatch | Yes (potential records integrity breach) | Yes |

## Cross-µservice coordination

- `audit-chain`: tombstone seals + Merkle proofs are owned by audit-chain; any seal mismatch escalates to audit-chain ops
- `tenancy`: tenant notification routes through tenancy's notification service
- `observability`: `anonymous_hard_delete_propagation_correctness` SLO tracked here

## References

- ADR-ANON-0004 (retention + deletion policy)
- GDPR Art. 17 (right to erasure); Art. 33 (breach notification)
- KR PIPA Art. 21 (deletion); Art. 34 (breach notification)
- UK Data Protection Act 2018 + UK GDPR Art. 33
- COPPA §312.5 (parental deletion)
- CCPA §1798.105 (right to delete)
- ADR-0028 (audit-chain Merkle / Ed25519)
