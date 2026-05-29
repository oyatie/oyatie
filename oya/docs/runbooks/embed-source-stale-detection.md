---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: docs
runbook_id: RB-embed-source-stale-detection
status: Accepted
date: 2026-05-17
owner_team: axis-docs + ops-sre-reliability
severity_applicable: [Sev-2, Sev-3]
related_failure_modes: [FM-08, FM-14, FM-17]
doc_status: published
---

# Runbook — Embed-source stale detection (workflow-studio / sheets / slides cross-µservice)

## When this runbook fires

- `oya_docs_embed_source_staleness_seconds > 600` (10 min) on a (embedding_doc, embed_ref) tuple.
- `oya_docs_cross_pack_embed_timeout_rate > 5%`.
- Embed-resolver returns "stale snapshot" banner repeatedly for a tenant.
- `oya_docs_embed_loop_detected_total > 0` (FM-17 recursive embed loop).
- Source-side ACL revocation didn't propagate to docs cache.

## Severity

- Single tenant + transient: Sev-3.
- Tenant business impact (stale data drives decision): Sev-2.
- Embed loop / ACL passthrough breach: Sev-2 (escalate to Sev-1 if cross-tenant disclosure suspected).

## Symptoms

- Embedded workflow-studio canvas shows yesterday's snapshot, not current.
- Embedded sheets cell shows pre-edit value.
- Cross-pack embed times out → fall-back to stale snapshot.
- Recursive embed loop refusal: `EmbedLoopDetected` audit event surfaces.

## Probable causes

1. Workflow event (`WorkflowStudioDefinitionPublished` / `SheetsCellChanged`) lost between source µservice and embed-resolver.
2. Cross-pack mesh partition.
3. Source-side ACL revoked but cache not invalidated.
4. Tenant created circular embed (Doc-A embeds Doc-B embeds Doc-A).
5. Embed-resolver worker queue saturation.

## Triage (within 15 min)

1. Acknowledge page; classify severity.
2. Identify affected (embedding_doc, embed_ref):
   ```bash
   oya docs embed list --status stale --pack <pack>
   ```
3. Check Workflow event subscription:
   ```bash
   oya docs embed subscription-health --source workflow-studio --pack <pack>
   ```
4. Check cross-pack mesh:
   ```bash
   oya mesh health --pair pack-<a>:pack-<b>
   ```
5. Check loop detection: `oya_docs_embed_loop_detected_total`.

## Section A — Stale embed (Workflow event lost or source µservice down)

| Step | Action |
|---|---|
| 1 | Force re-fetch: `oya docs embed refresh --document <d> --embed <e>`. |
| 2 | If source µservice returns 200: cache updated; alert clears. |
| 3 | If source µservice returns 5xx / timeout: surface "source unavailable" banner; retain prior cached snapshot. |
| 4 | Investigate source µservice health; engage source µservice's on-call. |
| 5 | If Workflow event subscription lag: replay missing events from outbox table. |

## Section B — Cross-pack mesh partition

| Step | Action |
|---|---|
| 1 | Verify mesh health between embedding pack and source pack. |
| 2 | If partition confirmed: cross-pack embeds gracefully degrade to stale snapshot. |
| 3 | Engage cloud-k8s on-call to restore mesh. |
| 4 | When mesh recovers, embeds auto-refresh on next access. |

## Section C — Source-side ACL revoked

| Step | Action |
|---|---|
| 1 | Identify the affected (source, embed_ref) by `EmbedAccessRevoked` event. |
| 2 | Cache invalidation should be automatic; if not, force: `oya docs embed invalidate --document <d> --embed <e>`. |
| 3 | Replace cached snapshot with redacted placeholder. |
| 4 | Notify embedding-doc tenant: "Embedded content access revoked by source tenant." |

## Section D — Recursive embed loop (FM-17)

| Step | Action |
|---|---|
| 1 | Embed-resolver enforces depth bound 3; loops refused at resolver. |
| 2 | Verify `EmbedLoopDetected` audit event surfaced. |
| 3 | Tenant UX surfaces "embed-loop detected" to author of the cyclic doc. |
| 4 | If loop persists: tenant must break the cycle by deleting one of the embeds. |

## Section E — Embed-resolver queue saturation

| Step | Action |
|---|---|
| 1 | Check queue depth: `oya_docs_embed_resolver_queue_depth_seconds`. |
| 2 | Scale embed-resolver-worker pool: `kubectl scale deployment/oya-docs-embed-resolver-worker -n docs --replicas=50`. |
| 3 | Verify single-flight coalescing is active (per-resolver-pod metric `coalesce_hit_ratio`). |
| 4 | If recurring: tune TTL jitter + per-(source,ref) coalescing window. |

## Recovery validation

| Metric | Target | After mitigation |
|---|---|---|
| `oya_docs_embed_source_staleness_seconds` p99 | < 60s | within 15 min |
| `oya_docs_cross_pack_embed_timeout_rate` | < 1% | within 15 min |
| `oya_docs_embed_loop_detected_total` rate | 0 | should be 0 |
| `oya_docs_embed_resolver_queue_depth_seconds` | < 60s | within 30 min |
| Tenant smoke-test (open doc with embed) | succeeds | yes |

## Post-incident review

- Was Workflow event subscription reliable?
- Did cross-pack mesh meet SLO?
- Update threat-model.md T-I-03 mitigation if a new ACL-passthrough vector discovered.
- If recurring loops: surface UX guidance to council-design-system.

## Drills

- Bi-annual cross-pack mesh-partition drill.
- Quarterly embed-storm simulation.

## References

- `failure-modes.md` FM-08, FM-14, FM-17.
- `threat-model.md` T-I-03, T-D-03, T-D-04.
- ADR-DOCS-0004 (per-block ACL; embed-resolver source-side ACL passthrough).
- `policy/data-residency.md` Invariant DR-04 (cross-µservice embed cross-pack).
- `dashboards/collab-health.json`.
- `microservices/workflow-studio/runbooks/*.md` — sibling source µservice runbooks.
