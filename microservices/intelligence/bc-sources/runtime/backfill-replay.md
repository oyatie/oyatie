---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: foundry-runtime
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry-runtime
deciders: axis-foundry-runtime, council-architecture, ops-sre-reliability
related_adrs: [ADR-0022, ADR-0024, ADR-0025, ADR-0131]
related_artifacts:
  - microservices/intelligence-runtime/PRD.md
  - microservices/intelligence-runtime/capacity-model.md
  - /specs/agent-operating-contract.json
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (foundry-runtime µservice)

## Purpose

Specify how the runtime handles two scenarios:
1. **Backfill** — historical invocation lifecycle records need re-emission to a downstream (e.g., foundry-evidence rebuilds its index after retention-extension; observability replays for capacity planning).
2. **Replay** — an existing invocation needs re-execution (e.g., bug-fix in capability descriptor; manual override reconciliation; post-incident validation that the corrected descriptor would have produced different outcome).

## Backfill

### Contract

When a downstream consumer (foundry-evidence; observability; tenant invocation history) needs to re-receive historical lifecycle events, the runtime re-emits them through the same AsyncAPI bus topics:

1. Trigger: `cargo run -p oya-dev-cli -- foundry-runtime backfill --from <ISO8601> --to <ISO8601> --tenant <id> --reason "<rfc>"`.
2. Authority: 2-person rule + OpenBao JIT elevation (backfill can flood downstream + carries audit semantics; controlled access).
3. The orchestrator-worker reads invocation_lifecycle Postgres rows in the time window matching the tenant scope.
4. For each row, re-emits the original event (`InvocationStarted` / `Completed` / `Failed` / `Cancelled`) with `backfilled=true` label so downstream can distinguish backfill from live.
5. Backfilled events carry the original signature (validated; no re-signing).

### Constraints

- Backfill does NOT change historical lifecycle records; they remain immutable in Postgres + audit-chain.
- Backfilled events are NOT consumed by workflow-engine for re-execution of waiting workflow steps; the `backfilled=true` label is a hard filter at workflow-engine.
- Cost: backfill is computed once per request. Cost = `O(events_in_window)`; rate-limited at orchestrator-worker to ≤1k events/sec to avoid downstream overload.
- Per-tenant rate limit: max 1 backfill per tenant per hour.

### Verification

- Integration test: emit synthetic lifecycle records; backfill window; verify all events re-emit with `backfilled=true`; verify downstream foundry-evidence accepts.
- Idempotency: re-running same backfill window emits the same events with same signatures (deterministic).

## Replay

### Contract

Replay re-executes the runtime portion of an invocation (capability dispatch + provider call + guardrails + session updates). Triggers:

- Bug-fix in capability descriptor: tenant operator triggers replay against a list of recent invocations to validate the fix.
- Manual override reconciliation: after a manual override (per `runbooks/autonomy-violation-quarantine.md` Path B), replay reconciles the "what would have happened" trace.
- Post-incident analysis: replay against alternate descriptor versions to test "would the corrected descriptor have produced different result?"

### Procedure

1. Operator invokes: `cargo run -p oya-dev-cli -- foundry-runtime replay --invocation-id <id> --reason "<rfc>"`.
2. CLI requires 2-person rule + ops-security approval (replay invokes real providers; cost-bound + audit-bound).
3. Runtime re-executes the invocation against current descriptor (NOT the descriptor at original invocation time, unless `--descriptor-version <orig>` is passed; that variant lets analysts test "the fixed version").
4. Emits `InvocationStarted{replayed=true, original_invocation_id=<orig>}` + downstream lifecycle events.
5. Audit-chain seal: replay is itself sealed; original invocation remains sealed; chain is reconstructable.

### Constraints

- Replay does NOT mutate the original invocation record; it appends a new invocation_id with `replayed=true` label.
- Replay cannot exceed Postgres retention (90d hot + cold restore from object-storage WAL beyond).
- Replay output never triggers retro-active workflow continuation (no "we now declare yesterday's failed invocation was actually fine; resume the workflow"); workflow-engine ignores `replayed=true` events.
- Per-tenant rate limit: max 100 replays per tenant per hour.
- Cost: replay carries full provider-call cost + guardrail cost + session ops cost; cost-budgeted via tenant's monthly cap.

### Verification

- Integration test: run an invocation; record outcome; replay with same descriptor; verify identical outcome (modulo provider non-determinism — declared in replay output).
- Replay against modified descriptor: verify outcome differs as expected.
- Audit-chain integrity: replay event sealed; original event remains sealed; chain reconstructable.

## Cost Model

| Operation | Frequency | Estimated cost per call |
|---|---|---|
| Backfill (1h window, 1 tenant) | per-trigger | ~$0.10 runtime overhead (downstream cost owned by consumer) |
| Replay (single invocation) | per-trigger | ~$0.0005 runtime overhead + provider cost (owned by foundry-providers) + guardrail cost |
| Replay (100-invocation batch for descriptor validation) | per-PR | ~$0.05 runtime overhead + provider × 100 |

Cost surfaced in `cost-budget.md` §"Cost-Optimisation Levers".

## Limitations

- **Replay non-determinism**: LLM provider outputs are non-deterministic at temperature > 0; replay carries `evaluator_version` + `provider_version` + `seed` (when available) so analysts can distinguish bug-fix-induced delta from provider-noise delta.
- **Backfill data freshness**: invocation_lifecycle records older than 90d require Postgres cold-restore from WAL archive (object-storage); backfill latency in that case is ≤5min per 10k events.
- **Replay session-state divergence**: session-state at replay time is current; the original session-state at invocation time may have changed. The replay output explicitly flags session_state_at_original_time_unavailable when applicable (rare; only when session evicted between original + replay).

## References

- `microservices/intelligence-runtime/PRD.md` Open Question 5.
- `microservices/intelligence-runtime/capacity-model.md`; `cost-budget.md`.
- `microservices/intelligence-runtime/contracts/asyncapi/foundry-runtime-events.yaml`.
- ADR-0022; ADR-0024; ADR-0025; `/specs/agent-operating-contract.json`.
- Google SRE Workbook ch. 4 (replay-driven incident analysis).
