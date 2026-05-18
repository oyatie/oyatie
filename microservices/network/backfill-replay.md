---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: network
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-network
deciders: axis-network, council-architecture, ops-sre-reliability
related_adrs: [ADR-0028, ADR-0126, ADR-0131]
related_artifacts:
  - microservices/network/PRD.md
  - microservices/network/capacity-model.md
  - microservices/network/contracts/asyncapi/network-events.yaml
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (network µservice)

## Purpose

Specify how `network` handles these scenarios:

1. **Backfill** — search index (multi-index) + feed cache + connection-graph degree-cache rebuild from canonical Postgres profile + post + connection-graph + endorsement-chain + jobs stores (e.g., after a Meilisearch corruption, Redis flush, or new ranking version shipped).
2. **Replay** — re-fanout of historical events to a newly subscribed downstream consumer (audit-chain, workflow-engine, mail action-card processor, ontology, ATS µservice), or to replay missed events for a tenant onboarded mid-stream.
3. **Endorsement-chain replay** — bounded re-derivation of the endorsement-chain from per-endorser Ed25519 signatures + audit-chain seals (ADR-NET-0005); used to verify integrity after corruption or to re-emit downstream after a consumer onboards.
4. **Jobs-handoff replay** — bounded re-emission of jobs-handoff events to ATS µservice after extended ATS outage (ADR-NET-0004).

Note: `network` does NOT federate in P01; there is no federation replay surface (deferred per ADR-NET follow-up).

## Backfill (search index rebuild — multi-index)

### Contract

Trigger sources:
- Operator-invoked: `cargo run -p oya-dev-cli -- network backfill-search --tenant <t> --index <people|content|skills|jobs|companies|events> --from <iso> --to <iso>`.
- Auto: corruption-detector emits `SearchIndexCorruptionDetected{index}` event; worker picks up + backfills the affected index + partition.

Procedure:

1. Acquire backfill lease in Redis (per tenant per index per partition; lease TTL = 1h).
2. Snapshot canonical Postgres rows in `(tenant_id, context_kind=Professional)` partition, ordered by `created_at`:
   - `people` index: `network_profiles` (handle, display_name, headline, skills array, current-role).
   - `content` index: `network_posts` (body, hashtags, mentions).
   - `skills` index: `network_skills_taxonomy` (pack-localised).
   - `jobs` index: `network_job_postings` (title, location, level, skills-required).
   - `companies` index: `network_pages` (name, industry, location).
   - `events` index: `network_events` (title, time, location, capacity).
3. Stream rows in batches of 1000 → Meilisearch `addDocuments` (idempotent on document primary key).
4. After bulk, emit `SearchIndexBackfilled{index, partition}` event with tuple `(tenant_id, partition, row_count, completed_at, signature)`.
5. Per-pack retention: backfill window bounded by retention floor.
6. PII redaction (per `policy/data-residency.md` + pack-overlay) applied at document-emission time.

### Constraints

- Backfill is read-only against Postgres; no row mutation.
- Cedar policy + redaction applied at document-emission time.
- Per-tenant rate limit: 1 active backfill per index per partition; cluster cap = 6 concurrent backfills cluster-wide.
- Cost: roughly $0.20 per 1M docs backfilled (PG read + Meilisearch index + S3 audit log; multi-index higher than social).

### Verification

- Integration test: corrupt Meilisearch index → backfill → search returns identical hits to Postgres `SELECT`.
- Idempotency: re-running same backfill produces no duplicate documents.
- Per-index health-check after backfill: facet counts match canonical PG counts.

## Backfill (feed cache rebuild)

### Contract

Trigger sources:
- Operator-invoked: `cargo run -p oya-dev-cli -- network rebuild-feed-cache --tenant <t> --user-ref <u>`.
- Auto: Redis cache eviction triggers per-user lazy rebuild on next feed-render.

Procedure:

1. Identify scope: per-user or per-tenant.
2. For each affected user, query Postgres for posts from connected accounts + followed Pages within feed window (default 7 days hot).
3. Rank using current ranking heuristic (P01) / model (P03).
4. Write feed slice to Redis cache with TTL.
5. Emit `FeedCacheRebuilt` event.

### Constraints

- Per-user rebuild bounded by Cedar (cannot pre-populate feed with posts user cannot read).
- Rebuild duration ≤ 12s per Professional user p95 (Professional graphs trend larger than Personal).

## Backfill (connection-graph degree-of-separation cache)

### Contract

Trigger:
- Operator-invoked: `cargo run -p oya-dev-cli -- network rebuild-degree-cache --tenant <t>`.
- Auto: after FM-05 connection-graph corruption recovery; Redis cache cleared.

Procedure:

1. For each user in tenant, run a BFS over `connection_edges` table out to depth 3 (1st / 2nd / 3rd-degree).
2. Materialise the count per degree into Redis (`degree_count:<user_ref>:1deg`, `:2deg`, `:3deg`).
3. Emit `DegreeCacheRebuilt` event.

### Constraints

- BFS bounded at depth 3 (UI never surfaces beyond 3rd-degree).
- Per-user cap: 10s rebuild p95 for users with > 1000 1st-degree connections.
- For tenants with > 100k accounts, rebuild runs in batches over off-peak hours.

## Replay (event fanout)

### Contract

Triggers:
- New µservice consumer onboards (e.g., new Workflow event consumer) and needs catch-up.
- Tenant onboarded mid-stream needs historical events for audit-chain seal rebuild.
- Bug-fix in event payload schema; re-emit corrected version.

Procedure:

1. Operator invokes: `cargo run -p oya-dev-cli -- network replay-events --tenant <t> --from <iso> --to <iso> --consumer <id>`.
2. CLI requires 2-person rule + ops-security approval (replay-events can re-trigger side-effects on consumers; must be audit-trail-bounded).
3. Worker scans `network_posts` + `network_connection_events` + `network_endorsement_events` + `network_jobs_events` + `network_audit_events` tables in `[from, to]` window; emits each as a Workflow event with `replay=true` label, `original_event_ts=<...>`.
4. Consumers MUST honour the `replay` label (idempotent processing on `(event_id, tenant_id)` tuple); failure to do so is the consumer's bug.
5. Audit-chain seal: replay emits sealed `EventReplayed` records per batch (one per 1000 events).

### Constraints

- Replay does NOT mutate the original event records; it appends fresh copies with `replay=true`.
- Replay window bounded by retention floor.
- Workflow consumers MUST be idempotent; replay-unsafe consumers are declared via `consumer_metadata.idempotent: false` and refuse the replay.
- Personal-tier never bleeds into replay (compile-time invariant; `network` is Professional-tier-only).
- Recruiter-stub events (when activated) require additional Cedar entitlement on the replay-target consumer (audit-grade replay only).

### Verification

- Integration test: synthetic consumer with idempotency tracking; verify replay of 10k events produces no duplicate side-effects.
- Audit-chain integrity: replay event seals link to original; chain reconstructable end-to-end.

## Endorsement-chain Replay

### Contract

Triggers:
- FM-14 endorsement-chain integrity compromise detected; replay re-derives the chain.
- New audit-chain consumer onboards and needs the full historical endorsement-chain.
- Independent integrity verification at audit cadence.

Procedure:

1. Operator invokes: `cargo run -p oya-dev-cli -- network replay-endorsement-chain --tenant <t> --from <iso> --to <iso>`.
2. Worker fetches per-endorser Ed25519 signatures from audit-chain authoritative replay.
3. For each endorsement record, verify signature; flag verification-failure as `signature_invalid` (do NOT auto-suppress; tenant-visible).
4. Re-derive Merkle tree from verified signatures; check against last sealed Merkle root.
5. Emit `EndorsementChainReplayed{tenant, partition, root_hash, verified_count, invalid_count}` event.

### Constraints

- Replay never mutates the canonical endorsement records; verification is read-only.
- Per-endorser Ed25519 public key fetched from per-user KMS at audit time; key rotation respected.
- Endorsement-chain integrity compromise (FM-14) triggers Sev-1 incident workflow.
- DSR cascade (GDPR Art. 17 erasure) revokes endorsement records BEFORE replay; replay must respect tombstones.

### Verification

- Integration test: synthetic endorsement-chain with 10k entries; replay produces identical Merkle root; injected forgery is caught.
- Drill: quarterly endorsement-chain integrity verification drill per `incident-response.md` Drills table.

## Jobs-Handoff Replay (to ATS µservice)

### Contract

Triggers:
- ATS µservice (Tier G) restored after extended outage; queued events flush.
- ATS contract-version upgrade; replay emit at new version.

Procedure:

1. ATS µservice POSTs `ATSResumeReady{from_event_id, contract_version}` to the network jobs-handoff worker.
2. Worker scans `network_jobs_events` from `from_event_id` forward; re-emits each at `contract_version` (down-converting where the new version is a superset).
3. Per-batch audit-chain seal.

### Constraints

- ATS must honour idempotency on `event_id`.
- Personal-tier never in jobs-handoff replay (Professional-only by µservice scope).
- Recruiter-stub-derived event tags travel with the replay; ATS must filter per its tenant Cedar policy.
- Per-tenant rate limit during resume (1k events/min).

### Verification

- Integration test: synthetic ATS endpoint requests resume; receives only Professional-tier events; signatures valid; contract version honoured.

## Cost Model

| Operation | Frequency | Estimated cost per call |
|---|---|---|
| Backfill people index (per 1M profiles) | per-corruption | ~$0.15 |
| Backfill content index (per 1M posts) | per-corruption | ~$0.20 |
| Backfill all 6 indexes (per 1M docs) | per-corruption | ~$0.85 |
| Rebuild feed cache (per 1M users) | per-Redis-flush | ~$0.70 |
| Rebuild degree-cache (per 1M users) | per-corruption | ~$2.00 (BFS-heavy) |
| Replay events (per 10k events × 1 consumer) | per-onboard | ~$0.05 |
| Replay events (per 1M events × all consumers) | per-bugfix-replay | ~$5.00 |
| Endorsement-chain replay (per 100k endorsements) | per-integrity-failure | ~$0.40 |
| Jobs-handoff replay (per 10k events) | per-ATS-recovery | ~$0.10 |

Cost surfaced in `cost-budget.md` §"Cost-Optimisation Levers".

## Limitations

- Backfill quality bounded by retention floor; cannot recover deleted + retention-purged content.
- Replay quality bounded by audit-chain seal availability; events older than the seal-archival horizon (24mo cold-tier) cannot be replayed.
- Endorsement-chain replay quality bounded by KMS public-key availability for revoked/rotated keys (key escrow keeps prior public keys 7y).
- Federation replay is NOT supported in P01 (network does not federate).
- Minor-account data is NEVER replayed via any external path; Cedar `minor_protect_reader` entitlement gated.

## References

- `microservices/network/PRD.md`.
- `microservices/network/capacity-model.md`.
- `microservices/network/cost-budget.md`.
- `microservices/network/contracts/asyncapi/network-events.yaml`.
- `microservices/social/backfill-replay.md` (sibling reference).
- ADR-0028 audit-chain.
- ADR-NET-0004 (jobs-handoff to ATS).
- ADR-NET-0005 (endorsement-chain integrity).
- ADR-0126 (Connect dissolution).
- ADR-0131 (per-microservice flat layout).
- RFC 8032 (Ed25519); RFC 6962 (Certificate Transparency Merkle-tree pattern; reference).
