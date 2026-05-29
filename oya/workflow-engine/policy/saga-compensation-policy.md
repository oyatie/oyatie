---
microservice: workflow-engine
contract: saga-compensation-policy
authored: 2026-05-18
canonical_authority: ADR-0222
canonical_base: /docs/standards/saga-compensation-policy.md
related_specs:
  - /specs/saga-shape.json
related_adrs:
  - ADR-0035
  - ADR-0145
  - ADR-0222
status: microservice-overlay
---

# workflow-engine — saga + compensation policy

## Purpose

This policy is the workflow-engine's binding contract with the rest of
the portfolio: how it ingests saga definitions, how it executes them,
how it persists state, and how it reports failure.

The canonical-base policy lives at `/docs/standards/saga-compensation-policy.md`.
This overlay pins workflow-engine-specific behavior and is consumed by
the workflow-engine's own kernel + usecase + adapter crates.

## Saga ingestion

At startup the engine reads every `microservices/<axis>/specs/saga-*.json`
and validates each against `/specs/saga-shape.json`. Ingestion failures:

| Condition | Outcome |
| --- | --- |
| JSON-Schema validation fails | Reject saga; emit `SagaIngestionRejected` audit row; engine continues with previously-ingested set |
| `audit_class != ReadOnly` AND `compensation_action.kind == NoopWithEvidence` | Reject saga |
| `target_microservice` not in canonical 32-µservice catalog | Reject saga |
| Duplicate `saga_id` across packs | Reject (latest version wins; alert ops-sre-reliability) |

The engine exposes `GET /sagas/registered` to list ingested sagas.

## Execution shape

```
trigger_saga(saga_id, input)
  └→ persist saga_run (state = pending)
  └→ for each step in order:
        ├→ persist step_run (state = pending)
        ├→ apply retry policy
        ├→ on success: persist step_run (state = succeeded); advance
        ├→ on failure (non-retryable or exhausted retries):
        │     ├→ persist step_run (state = failed)
        │     ├→ trigger reverse compensation chain
        │     └→ end saga (state = compensated or compensation-failed)
        └→ on timeout: same as failure
  └→ on all steps succeeded: persist saga_run (state = succeeded)
```

## Compensation chain

```
for each step in reverse order (only previously-succeeded steps):
  ├→ persist compensation_run (state = pending)
  ├→ apply compensation retry policy
  ├→ on success: persist compensation_run (state = succeeded)
  └→ on failure: persist compensation_run (state = failed); page on-call
```

Compensation failure does NOT roll back. The on-call engineer's job is
to manually reconcile. The audit chain records both the forward attempt
and the failed compensation so the manual reconcile has a starting point.

## Idempotency keys

Per step attempt: `Idempotency-Key = sha256(saga_run_id || step_id ||
attempt_number)`. The engine sends this header on every step
invocation. Downstream µservices honor the canonical Idempotency-Key
invariant (ADR-0128 INV-IDEMPOTENCY).

## Retry policy

| Field | Default | Range |
| --- | --- | --- |
| `max_attempts` | 3 | 1..10 |
| `backoff_ms` | 1000 (first), 2× per attempt | ≥ 100 |
| `jitter` | true | bool |
| `total_budget_ms` | step `timeout_budget_ms` | — |

`total_budget_ms` is the hard ceiling; the engine MUST NOT exceed it
even if `max_attempts` would.

## Persistence

The engine persists saga state in its own Postgres (workflow-engine
µservice's data plane). Persistence is per-cell (the saga runs on the
cell where it was triggered).

State is also mirrored to the audit chain per row:

| Event | Audit class |
| --- | --- |
| Saga triggered | SagaTriggered |
| Step started | SagaForward |
| Step succeeded | SagaForwardSucceeded |
| Step failed | SagaForwardFailed |
| Compensation started | SagaCompensate |
| Compensation succeeded | SagaCompensateSucceeded |
| Compensation failed | SagaCompensateFailed |
| Saga ended | SagaEnded |

## Engine restart semantics

On restart the engine:

1. Reads pending saga_runs from Postgres.
2. For each pending saga_run, replays its persisted step_runs to
   reconstruct in-memory state.
3. Resumes execution from the last persisted step boundary.
4. The audit-chain rows are not re-emitted (the persisted state is
   authoritative; the audit chain is a mirror).

## On-call paging

| Condition | Severity |
| --- | --- |
| Saga in flight > 1 hour | SEV-3 (warning) |
| Compensation failed | SEV-2 |
| Saga state divergence (Postgres vs audit chain) | SEV-1 |
| Saga registration rejected | SEV-4 (informational) |

## Observability

- `oya_saga_in_flight{saga_id, axis}` — gauge.
- `oya_saga_completion_class{saga_id, class="succeeded|compensated|compensation-failed"}` — counter.
- `oya_saga_step_duration_ms{saga_id, step_id}` — histogram.

Dashboard:
`microservices/observability/dashboards/saga-execution.md` (planned).

## SLO

| SLI | Target |
| --- | --- |
| saga registration success rate | ≥ 99.9% |
| saga step p99 latency overhead (engine layer) | ≤ 50 ms |
| saga compensation success rate (given compensation triggered) | ≥ 99% |
| audit-chain mirror lag | ≤ 5 s p99 |

OpenSLO manifests live under
`microservices/workflow-engine/slos/saga-*.openslo.yaml` (catalogued
under registry/cell µservice).
