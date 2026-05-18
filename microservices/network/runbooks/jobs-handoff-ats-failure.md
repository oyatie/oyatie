---
doc_class: Runbook
title: Jobs-handoff bridge to ATS degraded
microservice: network
severity: "Sev-2 (multi-tenant degradation; ATS pipelines stall)"
status: Accepted
owner_team: axis-network + axis-ats
date: 2026-05-17
last_drill_date: 2026-05-17
related_artifacts:
  - microservices/network/failure-modes.md (FM-19)
  - microservices/network/decisions/ADR-NET-0004-jobs-handoff-to-ats.md
  - microservices/network/backfill-replay.md (§"Jobs-Handoff Replay")
doc_status: published
---

# Runbook: Jobs-handoff bridge to ATS degraded (FM-19)

## Trigger

- `network_ats_bridge_queue_depth` > 50k sustained ≥ 5 min OR
- `network_ats_bridge_contract_mismatch_total` > 0 (contract-version drift detected) OR
- ATS µservice (Tier G) health-check failing for ≥ 5 min.

## Severity

Sev-2 default. Escalate to Sev-1 if ATS outage extends beyond 24h and tenant ATS pipelines impact hiring decisions.

## Immediate Mitigation (≤ 30 min)

| Step | Action | Time |
|---|---|---|
| 1 | Verify ATS µservice health: `kubectl -n ats get pods`; check ATS OpenSLO burn-rate | ≤ 2 min |
| 2 | Engage axis-ats on-call if ATS is the root cause | ≤ 5 min |
| 3 | Hold jobs-handoff events in Redis Streams `network:ats:bridge:queue:<tenant_id>` (worker continues batching; events are durable for 24h) | ≤ 5 min |
| 4 | Surface backlog UI to tenant-admins: "Jobs-pipeline handoff pending — ATS µservice degraded; queued events will replay automatically" | ≤ 5 min |
| 5 | Verify event contract-version compatibility: if ATS upgraded to v2 contract but network still on v1, coordinate with axis-ats for adapter | ≤ 30 min |
| 6 | Once ATS restored: invoke `runbooks` §"Jobs-Handoff Replay" via backfill-replay procedure; ATS POSTs `ATSResumeReady{from_event_id, contract_version}` | – |
| 7 | Drain queue at controlled rate (1k events/min per tenant; ATS rate-limit aware) | up to 30 min queue drain post-ATS-recovery |

## Contract-Version Mismatch Mitigation

If `network_ats_bridge_contract_mismatch_total` > 0:

| Step | Action |
|---|---|
| 1 | Inspect the rejected event payload: which contract version was emitted, which version did ATS expect |
| 2 | Engage ops-architecture council; this is typically a release-coordination bug |
| 3 | Emit `ContractMismatchDetected` event to ops-architecture topic for tracking |
| 4 | Coordinate with axis-ats: either ATS rolls forward to accept v1 + v2, or network rolls forward to emit v2 |
| 5 | Once aligned, replay queued events at the agreed version per `backfill-replay.md` §"Jobs-Handoff Replay" |
| 6 | Per ADR-NET-0004: future contract-version transitions require a 6mo dual-version window; investigate why this drift escaped CI lane `oya-gate validate jobs-handoff-contract` |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| ATS µservice outage | ATS OpenSLO burn red; ATS gateway health-check fails | engage axis-ats; await recovery; queue continues to buffer |
| ATS rate-limit refusing handoff events | `network_ats_bridge_event_send_failure_rate` includes 429 status | coordinate with axis-ats on per-µservice rate-budget |
| Contract-version drift (CI lane regression) | `network_ats_bridge_contract_mismatch_total` > 0 | ops-architecture engagement; CI lane `oya-gate validate jobs-handoff-contract` audit |
| Per-tenant ATS pipeline saturation | one tenant dominates queue | coordinate with gtm-customer-success on temporary budget raise; ATS µservice may need cell-shard for that tenant |
| Recruiter-stub-derived event tagging confusion (FM-15 cascade) | events flagged `recruiter_audit_pending=true` | engage `runbooks/recruiter-classifier-rollback.md`; do not replay recruiter-derived events until bias-audit cleared |

## Recovery Verification

- `network_ats_bridge_queue_depth` returns to < 5k sustained ≥ 30 min.
- `network_ats_bridge_contract_mismatch_total` rate at 0 for ≥ 24h.
- `network_ats_bridge_event_send_p95_seconds` ≤ 0.5.
- Per-tenant ATS confirmations received for replayed events (idempotency on `event_id` verified).
- No active alerts on jobs-handoff bridge.

## Tenant-Side Workaround (during extended outage)

If ATS outage extends beyond 4h, tenant-admins can:

1. Export queued job-applications via SDK `getQueuedAtsEvents()` → CSV.
2. Import into their ATS manually as a stopgap (acknowledging that re-import on automatic replay must be deduplicated by them).
3. Once ATS µservice restored, automatic replay backfills the canonical event log; tenant-admin must reconcile any manually-imported duplicates.

This workaround surfaces in tenant-admin onboarding doc.

## Postmortem Triggers

- Recurring queue backlog: review ATS-bridge worker sizing.
- ATS-side root cause: cross-µservice retrospective with axis-ats.
- Contract-version drift: review ADR-NET-0004 dual-version window discipline; possibly tighten CI lane.
- Tenant-impact > 4h: review BCDR posture in `multi-region.md`.

## References

- `microservices/network/failure-modes.md` FM-19.
- `microservices/network/decisions/ADR-NET-0004-jobs-handoff-to-ats.md`.
- `microservices/network/backfill-replay.md` §"Jobs-Handoff Replay".
- `microservices/network/multi-region.md` §"Cross-µservice Bridge Failure Modes".
