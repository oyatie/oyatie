---
doc_class: BackfillReplay
template_id: TPL-BACKFILL-REPLAY
microservice: community
status: Accepted
date: 2026-05-17
owner_team: axis-community + ops-sre
related_adrs: [ADR-0056, ADR-0105, ADR-0135, ADR-0131]
doc_status: published
---

# Backfill + Replay: community µservice

## Scenarios

### S1 — Search index rebuild (clean / corruption / schema migration)

- Source-of-truth: Postgres `community.posts` + `community.kb_articles`.
- Target: per-tenant Elasticsearch index `community-<tenant_id_short>-<bc>-v<N+1>`.
- Worker: `oya-community-search-index-worker` running reindex job.
- Throughput: 10⁴ docs/min per worker (parallelism = 4 per tenant).
- SLO: 10⁷ docs reindex completes in ≤ 60 min.
- Verification: document count + sample-query parity check.

### S2 — Foundry-guardrails classifier replay

- When classifier model updates: replay last 7 d of `PostCreated` events through the new model.
- Source-of-truth: NATS JetStream retained stream + Postgres snapshot.
- Idempotency: per `(post_id, model_version)` deduplication.
- Throughput: 10³ events/min.
- Output: revised `moderation_state` if model verdict differs.

### S3 — Audit-chain seal replay

- When audit-chain µservice resumes after outage: drain bridge queue.
- Bridge stores unsealed events for up to 24 h.
- Worker: `oya-community-audit-bridge-worker` flushes to audit-chain.
- SLO: full drain within 30 min of audit-chain restoration.

### S4 — Vote tally rebuild (after divergence detection)

- Source-of-truth: Postgres `community.votes`.
- Target: Redis tally counter.
- Worker: `oya-community-voting-engine-worker` reconcile job.
- Pause vote writes during rebuild; resume after verification.
- SLO: per-tenant rebuild in ≤ 5 min for 10⁶ votes.

### S5 — KB attachment integrity scan

- Walk Postgres `community.kb_attachments`; verify S3 object exists + sha256 matches.
- Schedule: weekly.
- Output: drift report; restore from cross-region replica if missing.

### S6 — DSR cascade replay (rare)

- When DSR cascade fails partway: resume from last completed step.
- Per-step idempotency key.
- Resume window: 30 d (regulatory deadline buffer).

## Replay Capabilities

| Capability | CLI invocation |
|---|---|
| Search reindex (per tenant) | `cargo run -p oya-community-search-index-cli -- reindex --tenant <T> --from-source postgres --to-index v<N+1>` |
| Search reindex (cluster-wide, staggered) | `cargo run -p oya-community-search-index-cli -- reindex-cluster --token-bucket 4 --staggered` |
| Classifier replay | `cargo run -p oya-community-moderation-queue-cli -- replay-classifier --since <ts> --model <v>` |
| Audit-bridge drain | `cargo run -p oya-community-post-store-cli -- audit-bridge-drain` |
| Vote tally rebuild | `cargo run -p oya-community-voting-engine-cli -- rebuild-tally --tenant <T>` |
| KB attachment scan | `cargo run -p oya-community-kb-article-store-cli -- attachment-scan --tenant <T>` |
| DSR cascade resume | `cargo run -p oya-community-post-store-cli -- dsr-resume --case <id>` |

## Throughput Targets

| Operation | Worker QPS | Tenant cap |
|---|---|---|
| Search reindex | 10 k docs/min/worker | 4 workers/tenant |
| Classifier replay | 1 k events/min | unbounded |
| Audit-bridge drain | 5 k events/min | unbounded |
| Vote tally rebuild | 100 k votes/min | 1 per tenant at a time |
| Attachment scan | 1 k objects/min | unbounded |

## Failure Modes

- Replay storm: token-bucket cluster-wide; tenant-staggered.
- Source-of-truth gap: Postgres WAL replay first if needed.
- Partial replay: per-step idempotency; resumable from last checkpoint.

## Audit

Every replay job emits a `BackfillStarted` + `BackfillCompleted` event sealed by audit-chain.
