---
doc_class: BackfillReplay
title: Backfill + replay plan
microservice: translate
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-translate + ops-sre-reliability
related_adrs: [ADR-0028, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/translate/PRD.md
  - microservices/translate/failure-modes.md
  - microservices/translate/contracts/asyncapi/translate-events.yaml
review_cadence: annually + after every major schema/contract update
doc_status: published
---

# Backfill + Replay — translate µservice

## Goals

- Recover from audit-chain gaps when NATS / foundry-evidence was unreachable during translate emission (FM-82).
- Replay TM updates from a known-good Postgres snapshot.
- Rebuild Meilisearch indices from Postgres ground truth (FM-11).
- Bulk re-emit `EuAiActDisclosure` records on per-pack regulatory request (audit reconstruction).
- Replay bulk-translate jobs that failed mid-pipeline (FM-50/51/52).

## Sources of Truth

| Source | Authority | Retention |
|---|---|---|
| Postgres TM tables | Ground truth for TM units + termbase | Per pack retention table |
| Postgres bulk_jobs table | Ground truth for bulk-job state | 90 d |
| S3 bulk-job artifacts | Source XLIFF/TMX/TBX input | 30 d default; tenant-configurable |
| Audit-chain (foundry-evidence) | Ground truth for `TranslationCompleted` + `EngineRouted` + `TmUpdated` + `EuAiActDisclosure` | Per pack retention |
| Mimir SLI metrics | Operational ground truth (rolling) | per pack retention |

Postgres is the canonical store; Meilisearch is derived (always rebuildable from Postgres).

## Backfill Scenarios

### Scenario A — Audit-chain gap during NATS outage (FM-82)

If NATS unreachable, adapters buffer events in a local write-ahead log (per ADR-0028). On NATS recovery, drain WAL.

| Step | Action | Tool |
|---|---|---|
| 1 | Detect: `oya_translate_audit_emit_error_rate > 1 %` | observability alert |
| 2 | Verify WAL buffer is accruing | `kubectl exec <adapter-pod> -- ls /var/run/translate/wal/` |
| 3 | On NATS recovery, replay WAL | `cargo run -p oya-dev-cli -- translate replay-wal --adapter <name>` |
| 4 | Verify per-event Ed25519 signatures preserved | per-event verify step |
| 5 | Confirm zero gap: `oya_translate_audit_emit_total` vs `oya_translate_invocations_total` deltas | observability |

### Scenario B — Meilisearch index corruption (FM-11)

| Step | Action | Tool |
|---|---|---|
| 1 | Detect: `oya_translate_tm_leverage_error_rate > 1 %` | observability |
| 2 | Halt new TM writes for affected pack | `cargo run -p oya-dev-cli -- translate halt-tm --pack <pack>` |
| 3 | Drop affected Meilisearch index | `meilisearch-cli delete tm-<tenant>-<project>` |
| 4 | Replay TM units from Postgres into Meilisearch | `cargo run -p oya-dev-cli -- translate reindex-tm --pack <pack> --tenant <t> --project <p>` |
| 5 | Verify count matches Postgres | per-pack count assertion |
| 6 | Resume TM writes | per CLI |

RTO: ≤ 30 min per affected tenant + project (10–100k TM units typical).

### Scenario C — Postgres TM table PITR (FM-10)

| Step | Action | Tool |
|---|---|---|
| 1 | Identify corruption window (start, end) | Postgres log + audit-chain |
| 2 | PITR to immediately-before window | `pg_restore` per OCI Postgres PITR |
| 3 | Re-emit `TmUpdated` events from PITR state forward where missing | replay job |
| 4 | Reindex Meilisearch from restored Postgres (Scenario B steps 3–5) | per CLI |
| 5 | Verify TM count matches per-tenant pre-corruption count | per-tenant assertion |

RTO: ≤ 60 min per pack.

### Scenario D — EU AI Act disclosure record audit reconstruction

Per `EuAiActDisclosure` retention requirement (10y per Art. 12 + Art. 18), audit-chain stores all records. Reconstruction:

| Step | Action | Tool |
|---|---|---|
| 1 | Identify time window + tenant scope of audit request | audit-chain query |
| 2 | Query foundry-evidence for `oya.translate.eu-ai-act.disclosure` events in window | foundry-evidence query API |
| 3 | Verify Ed25519 envelope signatures | `cargo run -p oya-dev-cli -- evidence verify --topic oya.translate.eu-ai-act.disclosure --from <t> --to <t>` |
| 4 | Export as CSV/JSON for regulator | per audit-chain |

### Scenario E — Bulk-translate job replay (FM-50/51/52)

| Step | Action | Tool |
|---|---|---|
| 1 | Identify failed job(s) | `oya_translate_bulk_job_state{state="failed"}` |
| 2 | Determine retryability per error class | per-job state in Postgres |
| 3 | Re-enqueue retryable jobs | `cargo run -p oya-dev-cli -- translate bulk-retry --job-id <id>` |
| 4 | For non-retryable (e.g., malformed XLIFF), notify tenant + record state | per-job |

### Scenario F — Engine adapter response replay (rare; debugging)

If a per-call response is needed for postmortem (e.g., suspected response-shape anomaly FM-06):

| Step | Action | Tool |
|---|---|---|
| 1 | Look up `decision_id` in audit-chain | foundry-evidence query |
| 2 | Pull adapter call evidence with response hash | per audit-chain |
| 3 | If foundry-evidence stored full response (per `evidence_attach: true`), retrieve | per foundry-evidence |
| 4 | Compare against canonical schema | adapter test fixture |

## Replay Safety

- **Idempotency**: every event carries `(decision_id, request_hash)`; consumers deduplicate.
- **Per-call canonical**: replay is bytewise-identical to original emission (Ed25519 verifies).
- **Privacy**: replay does NOT re-call vendors; replay operates on already-emitted events + already-stored TM units. No re-cost.
- **DSR-respecting**: if a tenant has DSR-erased segments between original emission and replay, replay scope excludes those segments.

## Verification

- `tests/integration/replay/` exercises every scenario.
- Quarterly drill of Scenario B + Scenario E.
- Annual drill of Scenario A (chaos: NATS outage + WAL drain) + Scenario C (PITR).

## References

- ADR-0028 audit-chain (WAL buffer).
- ADR-0139 SLO-gated promotion (rollback patterns).
- `microservices/translate/failure-modes.md`.
- `microservices/translate/contracts/asyncapi/translate-events.yaml`.
- Postgres PITR docs.
- Meilisearch index-rebuild docs.
- OCI Object Storage replication docs.
