---
doc_class: BackfillReplay
template_id: TPL-BACKFILL-REPLAY
microservice: sites
status: Accepted
date: 2026-05-17
owner_team: axis-sites + ops-sre-reliability
related_adrs: [ADR-0117, ADR-0131, ADR-SITES-0001, ADR-SITES-0003]
doc_status: published
---

# Backfill + Replay — sites µservice

## Purpose

Define backfill (cold-start population) + replay (event-stream
re-processing) procedures for sites' persisted state. Aligned with
DR (`multi-region.md`) and chaos-testing (`failure-modes.md`).

## When backfill / replay is engaged

| Trigger | Action |
|---|---|
| New cell brought online (per-pack expansion) | Backfill from peer cell snapshot |
| Cross-region failover post-promotion | Replay Loro CRDT log + audit-chain from snapshot anchor |
| Search-index reindex (Meilisearch corruption / version bump) | Replay from Postgres source-of-truth |
| Site-content migration from external CMS (WordPress, Squarespace etc.) | Backfill via importer (subsequent-to-M04-completion) |
| → Sites Strangler Phase 5 (legacy cutover) | One-time backfill of legacy `oya-sites-*` data |
| Schema change in cms-collection requiring re-rendering | Replay published artifacts via cdn-delivery-worker |

## Source-of-truth ordering

Per Bominal ADR-0028 + ADR-0117, the canonical source-of-truth ordering for sites state is:

1. **Postgres** (site, page, page-version, block-list, cms-collection, redirects, domain-binding, cert metadata, audit-chain pointer).
2. **Audit-chain seals** (immutable Merkle log; can verify Postgres integrity).
3. **Loro CRDT log** (per-page edit history; can reconstruct page block-tree).
4. **S3** (published-artifact HTML + assets; derived from Postgres + block-tree).
5. **Meilisearch index** (derived from Postgres + cms-collection).
6. **Valkey cache** (derived; rebuildable on demand).

Any inconsistency is resolved by re-rendering from Postgres + Loro CRDT log; S3 + Meilisearch + Valkey are treated as derived.

## Backfill procedures

### Backfill: new cell from peer snapshot

```bash
# Step 1: snapshot Postgres at peer cell
cargo run -p oya-dev-cli -- vcs snapshot --microservice sites --cell peer-cell-a --output s3://oya-cellgen/sites-snap-2026-05-17.tgz

# Step 2: restore to new cell
cargo run -p oya-dev-cli -- vcs restore --microservice sites --cell new-cell-b --input s3://oya-cellgen/sites-snap-2026-05-17.tgz

# Step 3: reattach Loro CRDT log
cargo run -p oya-dev-cli -- vcs crdt-rebuild --microservice sites --cell new-cell-b

# Step 4: reindex search
cargo run -p oya-dev-cli -- vcs search-reindex --microservice sites --cell new-cell-b

# Step 5: re-publish artifacts (for sites flagged as live)
cargo run -p oya-dev-cli -- vcs publish-replay --microservice sites --cell new-cell-b --scope live
```

### Backfill: legacy → Sites Strangler Phase 5

```bash
# Step 1: dump legacy data
cargo run -p oya-dev-cli -- vcs legacy-dump --legacy oya-sites --output s3://oya-migrate/sites-2026-05-17.jsonl

# Step 2: import to new µservice
cargo run -p oya-dev-cli -- vcs legacy-import --microservice sites --input s3://oya-migrate/sites-2026-05-17.jsonl \
  --redirect-map microservices/sites/specs/legacy-url-redirect-map.json

# Step 3: verify URL signature stability
cargo nextest run -p oya-sites-url-routing-domain -- redirect_signature_stability
```

## Replay procedures

### Replay: Loro CRDT log → block-tree reconstruction

```bash
# Per-page replay from CRDT log + Postgres journal
cargo run -p oya-dev-cli -- vcs crdt-replay --microservice sites --site-id <site_id> --page-id <page_id>

# Verify deterministic convergence
cargo nextest run -p oya-sites-block-adapter-loro -- crdt_converge
```

### Replay: Audit-chain → Postgres integrity check

```bash
cargo run -p oya-dev-cli -- audit-chain verify --microservice sites --range 2026-01-01..2026-05-17
```

### Replay: Published artifacts (CMS-collection schema change)

```bash
cargo run -p oya-dev-cli -- vcs publish-replay --microservice sites --scope cms-collection --collection-id <collection_id>
```

### Replay: Search index from Postgres

```bash
cargo run -p oya-dev-cli -- vcs search-reindex --microservice sites --tenant-id <tenant_id>
```

## Idempotency + determinism

- All replay operations are idempotent: replaying the same Loro log
  twice produces the same block-tree (CRDT commutativity guarantee
  per Loro 1.x).
- Audit-chain seals are append-only; replay does not duplicate.
- Published-artifact reconciliation: S3 object-key includes
  `version-hash`; re-publish overwrites only if hash changes.
- Meilisearch reindex is full-rebuild (Meilisearch index is
  per-tenant; rebuild from scratch is the operation).

## Latency expectations

| Operation | Scale | Expected duration |
|---|---|---|
| Postgres snapshot (50k sites, 5M pages) | per cell | 30 min |
| Cell restore from snapshot | per cell | 1h |
| Loro CRDT log rebuild | per page | 5s |
| Loro CRDT log rebuild (full cell) | per cell | 4h |
| Audit-chain verify | per cell, 1mo range | 10 min |
| Publish-artifact replay | per tenant, 1k pages | 30 min |
| Search reindex | per tenant, 5k pages | 10 min |
| Search reindex (full cell) | per cell | 4h |
| Legacy import | per tenant, 100 sites | 1h |

## Verification post-replay

- LEAN `oya-check-postgres-vs-audit-chain-consistency` — verifies that
  audit-chain Merkle root matches Postgres event count + content
  hashes.
- LEAN `oya-check-s3-vs-postgres-version-hash` — verifies that
  every Postgres `Page.version_hash` has matching S3 object-key.
- LEAN `oya-check-search-index-vs-postgres-page-count` — verifies that
  Meilisearch index size matches per-tenant published-page count.

## References

- ADR-0117, ADR-0131, ADR-SITES-0001, ADR-SITES-0003.
- Bominal ADR-0028 (audit-chain Merkle + Ed25519).
- `multi-region.md`, `failure-modes.md`, `incident-response.md`.
- Loro CRDT documentation — `loro.dev/docs`.
- Google SRE Workbook ch. 26 (data integrity).
