---
contract: saga-compensation-policy
authored: 2026-05-18
canonical_authority: ADR-0222
related_specs:
  - /specs/saga-shape.json
related_adrs:
  - ADR-0005
  - ADR-0035
  - ADR-0145
  - ADR-0222
status: canonical-base
overlay_consumers:
  - microservices/workflow-engine/policy/saga-compensation-policy.md
  - microservices/tenancy/specs/saga-onboard.json
  - registry/saga-shape/migration-backlog.tsv
authorities_cited:
  - AWS DistributedSagas re:Invent 2017
  - Microsoft Saga design pattern (Azure Architecture Center)
  - Temporal.io workflow + activity + compensation primitives
  - Stripe Engineering — Online migrations at scale (2017)
---

# Saga + compensating transaction policy

## Why this policy exists

Cross-microservice writes that span more than one transactional boundary
will, statistically, hit a partial-failure mode. Without a sanctioned
shape that requires the operation to declare its compensation at
definition time, the partial-failure rebuild is *archaeology* — the
on-call engineer reconstructs what *should* have rolled back. The
saga + compensation shape eliminates that archaeology by recording the
forward AND compensation at the same point: saga definition, then the
audit chain.

## Scope

Applies to every operation in the portfolio that:

- Writes to more than one microservice's persistent state.
- Mutates external state at any provider boundary (foundry capability,
  cloud-iac IaC apply, payment gateway, mail send, etc.).
- Spans a transactional boundary across cell boundaries (per ADR-0009).

Does NOT apply to:

- Read-only cross-microservice flows.
- Single-microservice writes (the µservice's own transaction is the
  shape).
- Cross-cutting carriers per ADR-0140 (audit-chain emission,
  observability emission, mesh-mTLS) which are explicitly exempt.

## Authoring a saga

### Step 1 — Declare the saga shape

```yaml
# microservices/<axis>/specs/saga-<name>.json
saga_id: tenant-suspend-saga
axis: workspace
owner_team: ops-compliance
version: v1.0.0
steps:
  - step_id: revoke_session_tokens
    target_microservice: tenancy
    forward_action:
      capability_ref: tenancy.session.revoke_for_tenant
      input_schema_ref: tenancy.session.RevokeForTenantInput
    compensation_action:
      kind: Custom
      capability_ref: tenancy.session.restore_for_tenant
      input_schema_ref: tenancy.session.RestoreForTenantInput
      evidence_class: WriteIdempotent
    idempotency_key_strategy: saga-step-attempt
    timeout_budget_ms: 10000
    retry_policy:
      max_attempts: 3
      backoff_ms: 1000
      jitter: true
    audit_class: WriteIdempotent

  - step_id: freeze_microservice_writes
    target_microservice: drive
    forward_action:
      capability_ref: drive.tenant.freeze_writes
      input_schema_ref: drive.tenant.FreezeWritesInput
    compensation_action:
      kind: Custom
      capability_ref: drive.tenant.unfreeze_writes
      input_schema_ref: drive.tenant.UnfreezeWritesInput
      evidence_class: WriteIdempotent
    idempotency_key_strategy: saga-step-attempt
    timeout_budget_ms: 30000
    retry_policy:
      max_attempts: 3
      backoff_ms: 2000
      jitter: true
    audit_class: WriteIdempotent

rollback_strategy: reverse-order-compensation
audit_chain_emit: true
```

### Step 2 — Register the saga

The workflow engine reads the spec at startup and admits it. Registration
fails if:

- Any step has `audit_class != ReadOnly` AND `compensation_action.kind ==
  NoopWithEvidence`.
- A target_microservice is not in the canonical 32-microservice catalog.
- The schema fails JSON-Schema validation.

### Step 3 — Trigger the saga

```rust
let saga_id = SagaId::from("tenant-suspend-saga");
let input = TenantSuspendInput { tenant_id, reason };
let saga_run = workflow_engine
    .trigger_saga(saga_id, input)
    .await?;
// saga_run.state() will reflect the progression
```

### Step 4 — Observe the saga

The workflow engine exposes:

- `GET /sagas/{saga_run_id}` — current state.
- `GET /sagas/{saga_run_id}/audit-rows` — audit-chain rows.
- Prometheus gauge `saga_in_flight{saga_id="..."}`.

## Compensation kinds

| Kind | Semantic | When to use |
| --- | --- | --- |
| `Cancel` | Undo the forward action; no side effect retained | Reservation-style steps (e.g. `reserve_cell`, `reserve_billing_credit`) |
| `Refund` | Reverse a monetary or counted effect | Billing or capability-cost steps |
| `Retry` | Re-attempt the forward action with idempotency key | Useful only when the forward action's failure was transient AND a re-attempt is the compensation (rare) |
| `NoopWithEvidence` | No compensation; emit evidence row only | ONLY when `audit_class == ReadOnly` |
| `Custom` | Caller-supplied capability ref | Default for non-trivial compensations |

## Compensation kinds the policy rejects

- "Best-effort compensate on error" — banned by D-6 of ADR-0222.
- Cross-microservice compensation (compensation that mutates a
  *different* microservice than the forward action) — banned by D-6.
- "Compensation will be added later" — every step MUST declare its
  compensation at definition time.

## Test matrix

Every saga MUST ship with:

1. **Happy-path test** — all steps succeed; assert final audit-row
   sequence.
2. **Step-K-fails test** — for each step K, simulate failure at step K
   and assert reverse-order compensation completes successfully.
3. **Compensation-fails test** — for at least one step, simulate
   compensation failure and assert the on-call page fires.
4. **Engine-restart test** — kill the workflow engine mid-saga;
   restart; assert saga resumes from last-persisted state.

Tests live in
`microservices/workflow-engine/tests/saga-<name>-test.rs` or
`microservices/<axis>/tests/saga-<name>-integration.rs`.

## Migration backlog

Existing cross-microservice flows that need re-shaping live in
`registry/saga-shape/migration-backlog.tsv`. The lane
`saga-shape` in the gate aggregator is advisory until the backlog
empties. Each backlog row is its own IP under the relevant
microservice's IP catalog.

## Worked example — tenant onboarding

The ADR-0175 tenant-lifecycle onboard saga is the canonical worked
example. Its full definition lives in
`microservices/tenancy/specs/saga-onboard.json` and its compensation
test matrix lives in `microservices/tenancy/tests/saga-onboard-test.rs`.

## Failure modes + on-call paging

| Mode | Engine behavior | Page severity |
| --- | --- | --- |
| Step succeeds | Advance | — |
| Step fails (retryable) | Apply retry; on exhaustion → reverse compensation chain | SEV-4 informational |
| Step fails (non-retryable) | Reverse compensation chain | SEV-4 informational |
| Step times out | Reverse compensation chain | SEV-3 |
| Compensation fails | Page on-call; manual intervention | SEV-2 |
| Engine crashes mid-saga | Resume from persisted state on restart | SEV-3 if duration > 15 min |
| Saga state divergence (audit chain says X; engine says Y) | Halt; page council-architecture | SEV-1 |

## Anti-patterns

- Synchronous chained RPC across µservices without saga registration
  — banned at mesh layer.
- "I'll add compensation when we have a bug" — banned by policy.
- Distributed transactions (XA / 2PC) — banned by ADR-0145.
- Saga without audit-chain emit — banned (`audit_chain_emit` defaults
  to `true` and cannot be `false`).
