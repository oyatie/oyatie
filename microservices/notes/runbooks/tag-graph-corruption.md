---
doc_class: Runbook
title: Tag-graph corruption
microservice: notes
severity: "Sev-3"
status: Accepted
owner_team: axis-notes + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/notes/backfill-replay.md (Tag-Graph Rebuild)
  - microservices/notes/failure-modes.md (F-TG-01..F-TG-03)
doc_status: published
---

# Runbook: Tag-graph corruption

## When

Triggers:

1. `oya_notes_tag_graph_corruption_detected_total > 0 over 5m` (consistency-check worker disagrees with note_tag truth).
2. User reports: tag-search returns notes not actually tagged, or omits notes that are tagged.
3. Tag rename / merge job fails partway.
4. Postgres partial write detected (rare — usually masked by transaction).

## Severity

Sev-3. Tag-graph is denormalised from `note_tag`; truth is recoverable.

## Detection

The consistency-check worker (per-tenant, daily) verifies `tag_edge` matches the truth derived from `note_tag`. On disagreement, emits `oya_notes_tag_graph_corruption_detected_total` with `{tenant_id, tag_id_a, tag_id_b}` labels.

## Procedure

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Acknowledge alert; identify affected tenant + tag scope | axis-notes oncall | t+15m |
| 2 | Snapshot current `tag_edge` partition for forensics | oncall | t+30m |
| 3 | Run rebuild query (per `backfill-replay.md` Tag-Graph Rebuild) | oncall | t+45m |
| 4 | Compare pre-rebuild vs post-rebuild diff; classify cause | oncall | t+90m |
| 5 | Re-emit `oya_notes_tag_graph_corruption_detected_total` post-rebuild — should be 0 | oncall | t+120m |
| 6 | If corruption persists: escalate to axis-notes principal engineer | oncall | t+2h |
| 7 | Audit-chain entry `TagGraphRebuilt{tenant_id, rebuilt_at, source_event_id}` | server | post-rebuild |

## Rebuild Query

```sql
BEGIN;
DELETE FROM tag_edge WHERE tenant_id = $1;

INSERT INTO tag_edge (tenant_id, tag_a, tag_b, weight)
SELECT t1.tenant_id, LEAST(t1.tag_id, t2.tag_id), GREATEST(t1.tag_id, t2.tag_id), COUNT(*)
FROM note_tag t1
JOIN note_tag t2 ON t1.note_id = t2.note_id AND t1.tag_id < t2.tag_id
WHERE t1.tenant_id = $1
GROUP BY t1.tenant_id, t1.tag_id, t2.tag_id;

COMMIT;
```

Chunked at 50,000 rows per batch; per-batch checkpoint to allow resume.

## Common Causes

| Cause | Action |
|---|---|
| Tag rename race (F-TG-03) | tx-locked rename with idempotent UPSERT; re-run rename |
| Tag merge incomplete | resume merge job with explicit target tag_id |
| Postgres partial write | replay from `NoteTagged`/`NoteUntagged` Workflow events |
| Bug in tag-graph worker | revert deploy; rebuild |

## Tag Rename Procedure (Safe)

```sql
BEGIN;
SELECT FROM tag WHERE tenant_id = $1 AND tag_id = $2 FOR UPDATE;

UPDATE tag SET name = $3 WHERE tenant_id = $1 AND tag_id = $2;
-- no change to note_tag or tag_edge (tag_id unchanged)
INSERT INTO audit_chain_event (kind, tenant_id, payload) VALUES ('TagRenamed', $1, ...);

COMMIT;
```

## Tag Merge Procedure (Safe)

```sql
-- Merges $source_tag_id into $target_tag_id
BEGIN;
-- 1. Update note_tag to point to target
UPDATE note_tag SET tag_id = $target_tag_id
WHERE tenant_id = $1 AND tag_id = $source_tag_id
ON CONFLICT (tenant_id, note_id, tag_id) DO NOTHING;
DELETE FROM note_tag WHERE tenant_id = $1 AND tag_id = $source_tag_id;

-- 2. Delete source tag
DELETE FROM tag WHERE tenant_id = $1 AND tag_id = $source_tag_id;

-- 3. Rebuild tag_edge for affected tenant (chunked outside this tx)
COMMIT;
```

## Personal-Tier Note

Per ADR-NOTES-0001, Personal-tier tags are E2E-protected (client-side). This runbook covers **Professional-tier tag-graph only**. Personal-tier client-side tag-graph corruption is handled in-SDK with full local rebuild from plaintext.

## Failure Modes

| Failure | Recovery |
|---|---|
| Rebuild query times out on Enterprise tenant | chunk by note range; checkpoint per chunk |
| Postgres deadlock during rebuild | retry with exponential backoff |
| `note_tag` itself corrupt (rare) | replay from `NoteTagged`/`NoteUntagged` Workflow events |

## Metrics

- `oya_notes_tag_graph_corruption_detected_total` — should be 0.
- `oya_notes_tag_graph_rebuild_duration_seconds` — per-tenant.
- `oya_notes_tag_rename_failure_total` — rename safety proxy.

## References

- `microservices/notes/backfill-replay.md`.
- `microservices/notes/failure-modes.md` F-TG-01..F-TG-03.
- ADR-NOTES-0001 (Personal-tier client-side tags).
