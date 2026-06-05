---
doc_class: ContractSpec
title: Backfill + Replay Contract (foundry-supervisor)
microservice: foundry-supervisor
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry-control-plane
deciders: axis-foundry-control-plane, council-architecture, ops-sre-reliability, ops-security
related_adrs: [ADR-0028, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/intelligence-supervisor/PRD.md
  - microservices/intelligence-supervisor/capacity-model.md
  - microservices/intelligence-supervisor/runbooks/supervision-bus-replay.md
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (foundry-supervisor µservice)

## Purpose

Specify how the supervisor handles:
1. **Backfill** — when a new tenant onboards or capability YAML history needs reconstruction; replay the audit-chain over a historical window.
2. **Replay** — re-emit existing supervision events for downstream consumers (foundry-evidence audit-chain catch-up, observability dashboard reconstruction, post-incident analysis).

## Backfill

### Contract

When a new tenant is onboarded (`TenantRegistered` event arrives):
1. Supervisor materializes the tenant's Postgres + OpenBao + Kubernetes namespace state from the canonical tenant DPA + entitlement set.
2. Supervisor emits a synthetic `TenantBackfillCompleted` event with the backfill window `(start_ts, end_ts)` — both `now()` for new tenant.
3. No historical deployment events are fabricated; backfill only seeds the starting fleet state.

When a tenant migrates from an external control plane (e.g., AWS Bedrock Agents):
1. Tenant exports their capability + deployment history via the migration tool.
2. Supervisor admit-loop replays each historical capability admit + deployment phase in chronological order, tagging events with `backfilled=true` label.
3. Supervisor emits `MigrationBackfillCompleted` event when done.

### Constraints

- Backfilled events never trigger live behaviour:
  - No kill-switch engagement is replayed (kill-switch is live-state-only).
  - No live rollouts kicked off.
  - foundry-runtime workers ignore backfilled events.
- Backfill is rate-limited: 1 backfill operation per tenant per 24 h.
- Cost is bounded by `O(N_historical_events × event_emission_cost)`.

### Verification

- Integration test: backfill a synthetic tenant; verify `backfilled=true` events emitted; verify foundry-runtime ignored them.
- Idempotency: re-running the same backfill produces the same event chain.

## Replay

### Contract

Replay re-emits existing events for downstream consumer catch-up. Triggers:

- foundry-evidence audit-chain Merkle break: replay the window to reseal.
- observability dashboard reconstruction: replay supervision events for missing dashboards.
- Post-incident analysis: replay events with alternate analysis lenses (e.g., "what if the autonomy threshold had been tighter?").
- supervision-bus lag remediation (FM-08): replay missed events to foundry-evidence.

### Procedure

1. Operator invokes: `Intelligence control-plane operation: supervisor replay-events --window <start_ts,end_ts> --consumer <foundry-evidence|observability|...> --reason "<rfc>"`.
2. The control-plane operation requires 2-person rule + ops-security approval for cross-tenant replays.
3. Supervisor reads events from supervision-event-bus (Valkey Streams (Redis wire-compat)) over the window.
4. Re-emits with `replayed=true` label; signatures unchanged (originals preserved).
5. Audit-chain seal: the replay-window itself is sealed, distinguishing replay from originals.

### Constraints

- Replayed events have `replayed=true` AND `original_event_id` label.
- Replay does NOT trigger live kill-switch engagement, deployment phase advance, or rollback.
- Replay cannot exceed event-bus retention (Valkey Streams (Redis wire-compat): 7 d at-rest + 6 mo in audit-chain).
- Replay output never re-applies state mutations (no double-deployment, no double-kill-switch).

### Verification

- Integration test: emit synthetic events; replay; verify identical event chain.
- Audit-chain integrity: replay event sealed; original event remains sealed.

## Cost Model

| Operation | Frequency | Estimated cost per call |
|---|---|---|
| Tenant onboarding backfill | per-new-tenant | ~$0.001 (one event emission + Postgres + Kubernetes namespace + OpenBao seeding) |
| Migration backfill | per-migration | varies; bounded by N_historical_events × $0.0001 |
| Replay for foundry-evidence catch-up | per-incident | varies; bounded by N_events × $0.0001 |
| Replay for post-incident analysis | per-analysis | bounded |

Cost surfaced in `cost-budget.md`.

## Limitations

- Backfill quality bounded by source-system fidelity (e.g., if migrating from Bedrock, only the metadata Bedrock exports is replayable).
- Replay assumes determinism in event ordering; re-played events carry `evaluator_version` to surface any semantic drift.
- Replay cannot reconstruct events beyond audit-chain retention (6 mo to 6 y depending on pack).

## References

- `microservices/intelligence-supervisor/PRD.md` Open Question 5 (replay window scope).
- `microservices/intelligence-supervisor/capacity-model.md`.
- `microservices/intelligence-supervisor/cost-budget.md`.
- `microservices/intelligence-supervisor/contracts/asyncapi/foundry-supervisor-events.yaml`.
- `microservices/intelligence-supervisor/runbooks/supervision-bus-replay.md`.
- ADR-0028, ADR-0139, ADR-0131.
- Valkey Streams (Redis wire-compat) — `redis.io/docs/data-types/streams/`.
