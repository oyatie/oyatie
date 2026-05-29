---
doc_class: ContractSpec
title: Backfill + Replay Contract (durable-execution-native)
microservice: workflow-engine
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-workflow
deciders: axis-workflow, council-architecture, ops-sre-reliability
related_adrs: [ADR-0035, ADR-0103, ADR-0131]
related_artifacts:
  - microservices/workflow-engine/PRD.md (FR-07 deterministic replay; AC-02 + AC-03)
  - microservices/workflow-engine/capacity-model.md
  - microservices/workflow-engine/policy/spec-integrity.md
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (workflow-engine µservice)

## Purpose

The workflow-engine has durable execution at Temporal parity — the engine is **itself replay-capable**. This document specs the contract for two distinct but related scenarios:
1. **Backfill** — computing run-history (or analytics) over historical event data when a new computation is registered. (Less common in workflow-engine than in metrics systems; primarily applies when ClickHouse analytics queries change schema.)
2. **Replay** — re-executing a workflow run's event log deterministically to either (a) reconstruct state for the debugger, (b) verify the deterministic-replay invariant after engine bug-fixes, or (c) test alternative scenarios against historical event sequences.

This is distinct from event-bus replay (which is a separate concept: re-delivering past events to a subscriber from offset N to M; covered by `runbooks/event-bus-replay.md` Path E). The replay contract here concerns **per-run** deterministic replay over the event log of a specific workflow run.

## Replay

### Contract — deterministic replay invariant

Per PRD AC-02 + AC-03: given the same initial state + the same event log, replay produces identical step sequence. This invariant is the engineering load-bearing contract.

The replay engine:
1. Reads the run's event log from Postgres / ClickHouse (per `EventLogReader` port).
2. Reconstructs the state at each event boundary by re-applying the state machine's transitions.
3. Emits step snapshots for each replayed step (via `replay-debugger-backend` REST stream).
4. Verifies the replayed step sequence matches the original (when target step range overlaps original).
5. Reports `identical_to_original: bool` on completion.

### Forbidden constructs (per `policy/spec-integrity.md`)

For replay to be deterministic, spec authors must NOT include in step bodies:
- System-time access (use engine-provided clock).
- Non-deterministic RNG (use engine-provided seeded RNG).
- Uncached external I/O (side effects must go through the side-effect ledger).
- Out-of-band state reads (state must flow through engine entities).

Engine refuses specs containing forbidden constructs at submit time.

### Side-effect ledger

Side-effecting steps (HTTP POST, DB writes, external API calls) are gated:
- During original execution: engine records the side-effect's outcome in the side-effect ledger keyed by `(run_id, step_index, idempotency_key)`.
- During replay: engine consults the ledger; if a record exists, replay uses the recorded outcome (no re-execution).
- Replay events are marked `replayed=true` on the bus; subscribers can refuse side-effecting actions on `replayed=true` events (defense in depth).

### Operator-initiated replay

Use cases:
- **Bug-fix verification**: after fixing the burn-rate math (or any engine domain logic), replay an affected run to verify the new logic produces the corrected sequence.
- **Operator debug**: tenant operator wants to step through a failed run to understand the failure point.
- **Compliance**: external auditor wants to reconstruct a run's state at a specific moment.

Procedure:
```bash
# Engineer or tenant operator
cargo run -p oya-dev-cli -- workflow-engine replay-run \
  --tenant <hash> --run-id <id> \
  --from-step 0 --to-step <last> \
  --reason "<rfc>"
```

For production-tier replay: 2-person rule required; audit-chain seal emitted with attribution.

### Constraints

- Replay does NOT mutate the original run record. A new replay session is created in `replay-debugger-backend`.
- Replay cannot exceed event-log retention (90d hot + 24mo cold; see `policy/data-residency.md`).
- Replay output never triggers retroactive promotion or side effects beyond the recorded ledger.

### Verification

- Integration test `tests/e2e/replay-deterministic.rs` — start a run; complete it; replay it; verify identical step sequence.
- `oya gate validate deterministic-replay` — LEAN lane that exercises replay against all canonical fixture runs in `capabilities/eval/`.
- Per-release replay-throughput benchmark: ≥ 1000 steps/s/worker on a single CPU (PRD AC-11).

## Backfill

### Contract

The workflow-engine's primary durable store is per-event-log-and-snapshot; backfill primarily applies when:
1. **ClickHouse analytics schema change**: a new aggregation column is added; backfill computes the new column for historical runs.
2. **Run-history rewrite for tenant DSR**: a DSR-driven retroactive scrub of a specific tenant's data subset.
3. **Spec-aware historical analysis**: a tenant wants run-history queryable under a new analytics dimension.

### Procedure

1. Tenant or operator triggers backfill via `cargo run -p oya-dev-cli -- workflow-engine backfill-analytics --tenant <hash> --metric <name> --window <duration>`.
2. Engine reads from Postgres (authoritative) + ClickHouse (analytics replica).
3. Computes the new column / dimension; writes back to ClickHouse.
4. Per-tenant rate-limited: 1 backfill per (tenant, metric) per day.

### Constraints

- Backfill does NOT mutate the original run state.
- Backfill never triggers retroactive event-bus emission.
- Backfill cost surfaced in `cost-budget.md` §"Cost-Optimisation Levers".

## Cost Model

| Operation | Frequency | Estimated cost per call |
|---|---|---|
| Replay a single run | per debugger session OR per release verification | ~$0.01 |
| Replay all runs of a tenant (bug-fix verification) | per engine domain release | ~$10 (full re-eval; bounded by ClickHouse query cost) |
| Backfill analytics column for a tenant | per schema change | ~$0.50 (single tenant, 90d window) |

## Limitations

- Replay quality is bounded by retention windows. Pre-2026-05 runs (if any) at lower-resolution ClickHouse may produce coarser snapshots.
- Determinism requires spec adherence to step-body discipline (per `policy/spec-integrity.md`). Specs that violate determinism contract refused at submit time; existing runs from pre-validation era may produce non-deterministic replays (flag `legacy_run=true`).

## References

- `microservices/workflow-engine/PRD.md` FR-07, AC-02, AC-03, AC-11.
- `microservices/workflow-engine/capacity-model.md`.
- `microservices/workflow-engine/cost-budget.md`.
- `microservices/workflow-engine/policy/spec-integrity.md` §"Forbidden Spec Constructs".
- `microservices/workflow-engine/runbooks/event-bus-replay.md` Path E (event-bus-level replay; distinct from per-run replay).
- ADR-0035 (workflow engine).
- ADR-0103 (workflow hexagonal).
- Temporal replay model — `docs.temporal.io/dev-guide/durability`.
