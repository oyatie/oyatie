---
doc_class: Runbook
title: TM corruption — restore (Postgres PITR + Meilisearch reindex)
microservice: translate
severity: "Sev-1 (Postgres corruption) / Sev-2 (Meilisearch corruption)"
status: Accepted
owner_team: ops-sre-reliability + axis-translate + ops-database
date: 2026-05-17
related_artifacts:
  - microservices/translate/failure-modes.md (FM-10, FM-11, FM-12, FM-13)
  - microservices/translate/backfill-replay.md (Scenarios B + C)
  - microservices/translate/decisions/ADR-TRANSLATE-0002-translation-memory-and-leverage-model.md
doc_status: published
---

# Runbook: TM corruption — restore

## Trigger

Any of:

- FM-10 (Postgres TM table corruption; block-checksum mismatch).
- FM-11 (Meilisearch TM index corruption; lookup fails).
- FM-12 (TM leverage match latency persistent > 80 ms p99).
- FM-13 (cross-tenant TM leverage event; HARD; see also `sovereign-tenant-cross-region-leak-incident-p0.md` if pack also crossed).

## Severity

- Postgres corruption: **Sev-1** (ground-truth degraded; risk of permanent data loss without action).
- Meilisearch corruption: **Sev-2** (re-buildable from Postgres; degraded leverage path).
- Latency: **Sev-3** (scale + cache warm).
- Cross-tenant breach: **Sev-1 (P0)** — see incident-response.md for cross-tenant breach protocol; this runbook covers the restoration after halt.

## Postgres TM Restore (FM-10)

| Step | Action | Time budget |
|---|---|---|
| 1 | IC declares Sev-1; opens `#inc-translate-tm` | ≤ 5 min |
| 2 | Halt new TM writes for affected tenant(s): `cargo run -p oya-dev-cli -- translate halt-tm --tenant <t>` | ≤ 5 min |
| 3 | Identify corruption window via Postgres log + audit-chain `TmUpdated` events | ≤ 30 min |
| 4 | Snapshot current state (for forensics): `pg_dump tm_units --table=tm_units > tm_units-pre-restore-<ts>.sql` | ≤ 15 min |
| 5 | PITR to immediately-before corruption window (OCI Postgres PITR): `cargo run -p oya-dev-cli -- pg pitr --cluster oya-translate-postgres-<pack> --target-time <ts>` | ≤ 30 min |
| 6 | Verify TM count matches pre-corruption per-tenant: per-tenant assertion via `tests/integration/tm_count_assertion.rs` | ≤ 15 min |
| 7 | Re-emit `TmUpdated` events from PITR state forward where missing (replay via `cargo run -p oya-dev-cli -- translate replay-tm-events --from <ts> --to now`) | ≤ 30 min |
| 8 | Reindex Meilisearch from restored Postgres (next section) | ≤ 30 min |
| 9 | Resume TM writes | ≤ 5 min |
| 10 | Tenant notification per `incident-response.md` | ≤ 60 min |
| 11 | Postmortem within 5 business days | ≤ 5 d |

**Total RTO: ≤ 60 min per pack.**

## Meilisearch TM Index Reindex (FM-11)

| Step | Action | Time budget |
|---|---|---|
| 1 | Detect: `oya_translate_tm_leverage_error_rate > 1 %` | t = 0 |
| 2 | Identify affected tenant(s) + project(s) | ≤ 5 min |
| 3 | Halt new TM writes for affected: `cargo run -p oya-dev-cli -- translate halt-tm --tenant <t> --project <p>` | ≤ 5 min |
| 4 | Drop affected Meilisearch index: `meilisearch-cli delete tm-<t>-<p>` | ≤ 2 min |
| 5 | Replay TM units from Postgres into Meilisearch: `cargo run -p oya-dev-cli -- translate reindex-tm --tenant <t> --project <p>` | ≤ 20 min (per 100k units) |
| 6 | Verify count matches Postgres + leverage sample queries pass: `cargo run -p oya-dev-cli -- translate verify-tm-index --tenant <t> --project <p>` | ≤ 5 min |
| 7 | Resume TM writes | ≤ 2 min |
| 8 | Audit-chain emits `TmReindexed{tenant, project, count, duration_ms}` | – |

**Total RTO: ≤ 30 min per tenant + project.**

## Per-Tenant HMAC Key Rotation (planned)

Per quarter; or on emergency (suspected key compromise):

| Step | Action | Time budget |
|---|---|---|
| 1 | Pre-rotation: write new HMAC key version (N+1) to OpenBao at `openbao://<pack>/<tenant>/translate/tm-hash-key?version=N+1` | ≤ 5 min |
| 2 | Background job re-hashes all per-tenant TM units with new key (Postgres update; Meilisearch reindex) | per tenant size (~ 10 min – 60 min) |
| 3 | Atomic switch: TM-rest reads version=N+1 going forward; version=N retained until cutover confirmed | ≤ 1 min |
| 4 | Verify all per-tenant TM lookups succeed with new key | ≤ 5 min |
| 5 | Revoke version=N at OpenBao | ≤ 2 min |
| 6 | Audit-chain emits `TmHashKeyRotated{tenant, from_version, to_version}` | – |

If emergency rotation (compromise): emit immediately + run rotation; tenant DPA breach posture if exposure window suspected.

## TM Bulk Export Failure (FM-14)

If bulk-export of TM gets stuck:

| Step | Action |
|---|---|
| 1 | Check Meilisearch fetch + Postgres pull progress |
| 2 | If chunk-fault: identify failing chunk; retry-or-skip |
| 3 | If transient: retry whole export |
| 4 | If repeat-fail: file engineering ticket; tenant gets partial export with notice |

## Verification

After restore:

- `tm_units` count matches per-tenant pre-corruption count (or post-replay forward count where new writes occurred).
- `oya_translate_tm_leverage_error_rate < 0.1 %` rolling 30 m.
- Sample leverage queries return expected matches (test fixture).
- Audit-chain events for affected window emitted (`TmReindexed`, `TmUpdated`).
- `tests/integration/cross_tenant_tm_isolation.rs` re-run post-restore.

## Post-Incident

- Postmortem within 5 business days.
- If FM-10 root cause = hardware: extend Postgres replication class; consider RF-3 base storage.
- If FM-11 frequency: investigate Meilisearch deployment stability; consider upgrade to next LTS.
- If FM-13: separate P0 cross-tenant breach protocol per `incident-response.md`.

## References

- ADR-TRANSLATE-0002 (TM + leverage model).
- `microservices/translate/backfill-replay.md` Scenarios B + C.
- `microservices/translate/failure-modes.md`.
- Postgres PITR docs.
- Meilisearch index docs.
- OCI Postgres managed service docs.
