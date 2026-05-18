---
doc_class: Runbook
title: Event bus replay + outbox crash recovery + backpressure handling
microservice: workflow-engine
severity: "Sev-2 (operational; no data loss expected)"
status: Accepted
owner_team: axis-workflow + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/workflow-engine/failure-modes.md (FM-02 backpressure, FM-07 outbox crash, FM-12 event poisoning)
  - microservices/workflow-engine/PRD.md (FR-03, FR-04, FR-13)
  - microservices/workflow-engine/policy/spec-integrity.md
doc_status: published
---

# Runbook: Event bus replay + outbox crash recovery + backpressure

## Trigger (one of)

1. **Backpressure**: `oya_workflow_engine_event_bus_consumer_lag_seconds > 60` for one or more subscriptions sustained ≥ 2 min.
2. **Outbox crash**: outbox-relay worker absent; `oya_workflow_engine_outbox_lag_seconds > 30`.
3. **Poison event**: a subscriber crashes consistently on a specific event; consume failure rate > 3 within 1 min.
4. **Tenant-initiated replay**: tenant operator requests event-log replay from offset N to offset M for a specific subscription.

## Severity

- Backpressure single subscription: Sev-3.
- Backpressure cluster-wide (multiple subs lagging): Sev-2.
- Outbox crash + HA failover successful: Sev-3.
- Outbox crash + HA failed: Sev-1.
- Poison event quarantined: Sev-3.

## Pre-checks

1. Verify event-bus health: `kubectl -n workflow-engine get pods -l app=event-bus-worker`.
2. Verify outbox Postgres: SELECT count FROM outbox WHERE published_at IS NULL — should be bounded (< 10k).
3. Verify subscription registry in Valkey: `redis-cli SCAN 0 MATCH 'oya:workflow:sub:*' COUNT 100`.

## Recovery Path A — Backpressure on single slow subscription

| Step | Action |
|---|---|
| 1 | Identify the slow subscription: `cargo run -p oya-dev-cli -- workflow-engine inspect-subscription --tenant <hash> --sub-id <id>` reveals consume rate, lag, last-success timestamp. |
| 2 | Verify backpressure signal was sent: subscription state should show `flow_control: applied`. |
| 3 | If subscriber is genuinely slow but live: increase delivery batch window OR allow slow-subscriber quarantine policy to engage. |
| 4 | If subscriber is dead: cancel the subscription `cargo run -p oya-dev-cli -- workflow-engine drop-subscription --tenant <hash> --sub-id <id> --reason "<rfc>"`; notify tenant. |
| 5 | Re-subscribe after subscriber recovery via SDK; new subscription resumes from latest offset OR replay from specified offset. |

## Recovery Path B — Cluster-wide backpressure (many subscriptions lagging)

| Step | Action |
|---|---|
| 1 | Sev-2 declared; engage axis-workflow + ops-sre-reliability. |
| 2 | Check publisher rate: is one publisher emitting at unprecedented rate? `cargo run -p oya-dev-cli -- workflow-engine inspect-publisher --tenant <hash>` |
| 3 | Apply per-tenant publish rate limit (temporarily; recover normal afterwards): `cargo run -p oya-dev-cli -- workflow-engine apply-publish-cap --tenant <hash> --cap <events/s>` |
| 4 | HPA on event-bus-worker should scale up; verify replicas ramping. |
| 5 | Postgres outbox table grew; verify within bounds. |

## Recovery Path C — Outbox relay worker crash + HA failover

| Step | Action |
|---|---|
| 1 | HA leadership-election fails over to standby outbox-relay. New leader resumes from `last_published_offset` persisted in Postgres. |
| 2 | Verify new leader is active: `oya_workflow_engine_outbox_leader_alive == 1`. |
| 3 | If HA both pods crashed (rare): emergency manual leader-election via `kubectl rollout restart deployment/event-bus-worker`. |
| 4 | Verify outbox lag is recovering to baseline. |
| 5 | Postmortem: harden the crash mode that took out both pods. |

## Recovery Path D — Poison event quarantine

| Step | Action |
|---|---|
| 1 | Identify the poison event: `cargo run -p oya-dev-cli -- workflow-engine inspect-poison-queue --tenant <hash>` |
| 2 | Identify the subscriber that fails on this event. |
| 3 | Engage subscriber owner for fix (e.g., handle new event field). |
| 4 | Tenant decides: skip the poison event OR replay after subscriber fix. |
| 5 | Replay after fix: `cargo run -p oya-dev-cli -- workflow-engine replay-events --tenant <hash> --sub-id <id> --from-offset <N> --to-offset <M>` |
| 6 | Skip permanently: `cargo run -p oya-dev-cli -- workflow-engine skip-poison-event --tenant <hash> --sub-id <id> --event-id <eid> --reason "<rfc>"`. Audit-chain seal emitted. |

## Recovery Path E — Tenant-initiated replay (operational use case)

| Step | Action |
|---|---|
| 1 | Tenant operator authenticates + provides replay window. |
| 2 | Execute: `cargo run -p oya-dev-cli -- workflow-engine replay-events --tenant <hash> --sub-id <id> --from-offset <N> --to-offset <M>` |
| 3 | Engine emits replay-flagged events on the bus; subscribers receive with `replayed=true` label and apply idempotency. |
| 4 | Verify subscriber processed the replay window. |
| 5 | Audit-chain seal recorded. |

## Verification

- Subscription consumer lag returns to baseline (< 10s).
- Outbox lag returns to baseline (< 5s).
- Poison queue depth: 0 (or quarantined items have explicit operator decision).
- Tenant-facing dashboard reflects healthy event flow.

## Post-incident updates

- Postmortem within 5 business days.
- Action: SLO on subscription consumer lag with burn-rate alert.
- Action: subscriber idempotency contract revalidated.
- Action: if recurring backpressure pattern, fix root cause (publisher rate; subscriber capacity).

## References

- `microservices/workflow-engine/failure-modes.md` FM-02, FM-07, FM-12.
- `microservices/workflow-engine/PRD.md` FR-03, FR-04, FR-13.
- `microservices/workflow-engine/policy/spec-integrity.md`.
- Postgres outbox pattern — `microservices.io/patterns/data/transactional-outbox.html`.
