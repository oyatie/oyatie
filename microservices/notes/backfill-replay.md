---
doc_class: BackfillReplay
title: notes µservice — Backfill + Replay Plan
microservice: notes
status: Accepted
date: 2026-05-17
owner_team: axis-notes + ops-sre-reliability
doc_status: published
---

# Backfill + Replay — notes µservice

## Scenarios Requiring Backfill / Replay

| Scenario | Source-of-truth | Target stores | Cadence |
|---|---|---|---|
| Search index rebuild after Meilisearch loss | Postgres notes table | Meilisearch (Professional tier only) | on-demand + monthly compaction |
| Tag-graph adjacency rebuild | Postgres notes + note_tag join | Postgres tag_edge | on-demand |
| Backlink adjacency rebuild | Postgres notes (parse `[[wikilinks]]`) | Postgres backlink table | on-demand |
| Daily-note timeline restore | Postgres daily_note + note | (reconstruct timeline) | on-demand |
| Ontology Note-entity rewrite | Postgres notes | ontology µservice | on schema migration |
| Audit-chain replay verification | source events | (verification only) | quarterly |

## Backfill Pipeline

```
Postgres → (read replica) → backfill-worker → batch update → target store
                                              → emit audit-chain "backfill_segment_sealed"
```

Pattern:
1. Backfill-worker reads from Postgres read-replica in stable chunks (10k rows).
2. Per-row computation (tokenise body for search, parse wikilinks for backlinks, etc.).
3. Batched write to target.
4. Per-chunk Ed25519 seal emitted to audit-chain.
5. Backfill job state machine: `pending → running → paused → resumed → completed | failed`.

## Personal-Tier Constraint

Personal-tier ciphertext **CANNOT be re-indexed server-side**. Backfill is bounded to:
- Per-note metadata (id, created_at, tenant_id, user_id, context_kind).
- No body content.
- No tags (Personal-tier tags are also E2E-protected per DCI-03).

For Personal-tier search-index rebuild (client-side encrypted index in IndexedDB):
- Client SDK rebuilds locally on demand using the user's local plaintext cache + key material.
- Server has no role.

## Search Index Rebuild (Professional Only)

### Trigger
- Meilisearch shard loss.
- Quarterly compaction.
- Schema migration.

### Procedure
1. Mark target index `building` (reads still served from old replica).
2. Backfill-worker streams from Postgres in `(tenant_id, created_at)` order.
3. Per-batch `add_documents` call to Meilisearch.
4. On completion, swap pointer; verify count; teardown old index.
5. Audit-chain seal `MeilisearchIndexRebuilt` per tenant.

### Throughput
- 5,000 notes/sec sustained on M-target cell.
- 100GB index built in ~6h on XL cell.

### Idempotency
Backfill rows include source `note_id`; re-runs produce identical Meilisearch state.

## Tag-Graph Rebuild

```sql
TRUNCATE tag_edge;
INSERT INTO tag_edge (tenant_id, tag_a, tag_b, weight)
SELECT t1.tenant_id, t1.tag_id, t2.tag_id, COUNT(*)
FROM note_tag t1
JOIN note_tag t2 ON t1.note_id = t2.note_id AND t1.tag_id < t2.tag_id
WHERE t1.tenant_id = $1
GROUP BY t1.tenant_id, t1.tag_id, t2.tag_id;
```

Run per-tenant; chunked at 50k tags.

## Backlink Rebuild

Per-note: parse Markdown body via `pulldown-cmark`-based wikilink extractor; insert `(tenant_id, from_note_id, to_note_id, kind)` triplets. Idempotent: `INSERT ... ON CONFLICT DO NOTHING`.

## Replay From Workflow Event Log

Workflow events are append-only per ADR-0028. Replay scenarios:

| Use case | Procedure |
|---|---|
| Reconstruct denormalised tag-graph from `NoteTagged`/`NoteUntagged` events | replay from event log; idempotent via event_id |
| Reconstruct backlink-graph | replay `BacklinkResolved`/`BacklinkBroken` |
| Reconstruct version-history pointer | replay `NoteEdited` |

Replay tooling: `oya-dev-cli replay --microservice notes --from <ulid> --to <ulid> --bc <name>`.

## Audit-Chain Replay Verification

Quarterly:
1. Pick random week of audit-chain segments.
2. Re-derive Merkle root from source rows.
3. Compare to sealed Merkle root.
4. Verify Ed25519 signature against published public key.
5. Discrepancy → Sev-1 (audit-chain integrity broken).

## Personal-Tier Audit-Chain Replay

Personal-tier emits audit-chain ONLY on sharing events. Replay verifies:
- Every `ShareLinkCreated` has a matching `ShareLinkRevoked` or active TTL.
- Every `ShareLinkAccessed` references a still-valid (or recently-valid) share-link.

## Throughput + RTO

| Job | Throughput | RTO |
|---|---|---|
| Full Meilisearch rebuild (Business tier) | 30 min | 60 min |
| Full Meilisearch rebuild (Enterprise tier) | 6 h | 12 h |
| Tag-graph rebuild (Business tier) | 5 min | 10 min |
| Backlink rebuild (Business tier) | 20 min | 40 min |
| Audit-chain quarterly verification | 24 h | n/a (verification only) |

## References

- ADR-0028 (audit-chain Merkle + Ed25519, inherited).
- ADR-0139 (SLO-gated promotion).
- `runbooks/tag-graph-corruption.md`.
- `runbooks/attachment-loss-recovery.md`.
