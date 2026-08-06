---
id: ADR-0222
status: Superseded
date: 2026-05-18
owners:
  - council-architecture
  - ops-sre-reliability
  - axis-workspace
supersedes: []
superseded_by: [ADR-0704]
related:
  - ADR-0005-eventing-backbone-outbox-pattern.md
  - ADR-0035-workflow-engine-state-machine-and-dag-hybrid.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0128-hyperscaler-architecture-invariants.md
doc_class: Architecture-Decision-Record
purpose: >
  Make sagas the only sanctioned shape for cross-microservice writes.
  Every cross-µservice mutation declares (forward_action,
  compensation_action, idempotency_key). The workflow engine is the
  coordinator. Two-phase commit remains banned (per ADR-0145). The
  audit chain records both forward and compensating actions.
enforcement_status: advisory-until-cross-flow-recatalogued
enforced_by: oya gate validate saga-shape
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0222: Saga + compensating-transaction portfolio policy

## Status

Accepted — 2026-05-18. Enforcement is advisory until every existing
cross-microservice flow has been re-shaped to the saga form catalogued
in `registry/saga-shape/migration-backlog.tsv`. Promotion to strict
follows the deferred-gate pattern (see ADR-0145 promotion mechanic).

## Context

PR-143 Fix-M's portfolio audit identified a structural gap: ADR-0035
(workflow engine state-machine + DAG hybrid) sketches saga semantics as
an internal workflow-engine feature, and ADR-0145 (inter-microservice
communication reform) bans two-phase commits, but no portfolio-wide
ADR establishes the rule *every cross-µservice write goes through a
saga*. As a result:

- Some µservices invent ad-hoc cross-µservice transactions
  (synchronous chained RPCs with no compensation path).
- Some teams implement "best-effort compensate on error" without
  declaring the compensation action up-front, leaving partial-failure
  states uninvestigated.
- The audit chain (ADR-0003) records the forward calls but cannot
  reconstruct which compensation should have fired because that
  information was never declared at the saga-definition layer.

The hyperscaler industry consensus (AWS DistributedSagas pattern,
Microsoft Saga pattern guidance, Uber Cadence, Temporal.io) is the same:
sagas + compensation are the canonical shape for cross-service
mutations. The Oyatie portfolio must adopt the same shape and pin it as
a portfolio-wide invariant, not an opt-in workflow-engine feature.

### Why now

PR-143 lands the workflow-engine saga coordinator (per ADR-0035) for
the workspace + tenancy axes; the foundation is now strong enough to
make the saga shape portfolio-mandatory. The audit chain (ADR-0003)
already records arbitrary events, so binding compensation to the chain
requires no new substrate.

### What changes

| Surface | Before this ADR | After this ADR |
| --- | --- | --- |
| Cross-µservice write definition | Free-form; depends on the team | Saga step block in workflow-engine schema |
| Compensation declaration | Ad hoc; sometimes missing | Mandatory per step; declared at saga registration |
| Audit chain rows | Forward call only | Forward + compensation invocation rows |
| Two-phase commit | Banned (ADR-0145) | Banned (ADR-0145, reaffirmed) |
| Workflow engine role | Optional coordinator | Mandatory coordinator for cross-µservice writes |

## Decision

### D-1. Saga shape

Every cross-µservice write is a saga consisting of an ordered list of
steps. Each step declares:

```rust
pub struct SagaStep {
    pub step_id: StepId,                       // unique within the saga
    pub target_microservice: MicroserviceId,
    pub forward_action: ActionRef,             // capability + input
    pub compensation_action: CompensationRef,  // Cancel | Refund | Retry | Noop-with-evidence
    pub idempotency_key_strategy: IdempotencyKeyStrategy,
    pub timeout_budget_ms: u32,
    pub retry_policy: RetryPolicy,
    pub audit_class: AuditClass,
}
```

`CompensationRef::NoopWithEvidence` is allowed only when the forward
action is provably side-effect-free (read-only, query-only). Any step
with `audit_class != ReadOnly` MUST declare a non-noop compensation.

### D-2. Workflow engine is the coordinator

The workflow engine (per ADR-0035) is the only sanctioned coordinator.
µservices MUST NOT chain cross-µservice writes outside the engine; the
mesh layer (Istio per ADR-0148) rejects direct cross-µservice POST/PUT
calls between business µservices unless the request carries a valid
`oya-saga-coordinator-token` issued by the engine. Read-only GETs are
unaffected.

### D-3. Idempotency + retry

Each saga step carries an idempotency key derived from
(saga_id, step_id, attempt_number). Receivers MUST honor the
Idempotency-Key invariant (ADR-0128 INV-IDEMPOTENCY) and dedupe.
Retry policy declared per step honours the per-hop latency budget
(`docs/standards/cross-microservice-latency-budget.md`); the engine
enforces deadline propagation.

### D-4. Audit chain shape

Every forward action emits an audit row of class `SagaForward`. Every
compensation emits `SagaCompensate`. The compensation row references
the forward row by event_id. `oya-check-audit-chain-seal-coverage`
(existing) extends to recognise both row classes.

### D-5. Failure modes

| Mode | Engine behavior |
| --- | --- |
| Step succeeds | Advance to next step |
| Step fails (retryable) | Apply retry policy; on exhaustion, treat as `failed` |
| Step fails (non-retryable) | Trigger reverse compensation chain |
| Step times out | Treat as failed; trigger compensation |
| Compensation fails | Page on-call (per ADR-0040 metric-gated rollback alert); manual intervention |
| Engine crashes mid-saga | On restart, replay the saga log (per ADR-0024 eval replay generalization); resume from the last persisted state |

### D-6. Bans

The following patterns are banned:

- Distributed transactions (XA / 2PC) across µservices — already banned
  by ADR-0145, reaffirmed here.
- "Best-effort compensate on error" without an up-front compensation
  declaration.
- Direct cross-µservice writes outside the saga coordinator (except
  the explicit cross-cutting carriers exemption in ADR-0140).
- Compensation that mutates a different µservice than the forward
  action (compensation is bound to its forward target).

## Alternatives considered

### Alt-1. Per-axis saga coordinators

Let each axis (workspace, cloud, foundry) implement its own saga
coordinator. **Rejected.** Multiplies the audit-chain integration
surface; defeats the portfolio-wide audit-row class invariant; each
re-implementation re-introduces partial-failure bugs the workflow
engine has already solved.

### Alt-2. Workflow engine optional; default = synchronous chained RPC

Keep cross-µservice writes as direct RPC chains and treat the workflow
engine as an opt-in for "complicated" flows. **Rejected.** The
Stripe + Uber + AWS distributed-sagas evidence is unambiguous: every
non-trivial cross-service mutation hits a partial-failure mode
eventually, and "we'll add compensation when we hit a bug" loses every
time. The cost of saga-shaping every flow is small; the cost of
deferring is unbounded.

### Alt-3. Event-driven choreography only (no central coordinator)

Adopt pure event-driven choreography (each µservice publishes events;
peers subscribe). **Rejected.** Choreography optimizes for write
throughput at the cost of *observability* — there's no central place
to see the saga state, the compensation chain is implicit, and
debugging a stuck saga becomes archaeology. Orchestration via the
workflow engine keeps the saga state explicit and inspectable, which
the audit-chain + observability dashboards require.

## Consequences

### C-1. Positive

- **Partial-failure recovery is provable.** Every cross-µservice
  write has a declared compensation; the audit chain records it.
- **Saga state is inspectable.** The workflow engine exposes a
  `GET /sagas/{id}` route; the observability dashboard plots
  in-flight saga counts per axis.
- **Cross-axis flows can be re-shaped uniformly** (e.g. the tenant
  lifecycle from ADR-0175).
- **Hyperscaler-grade.** Matches AWS DistributedSagas + Temporal +
  Microsoft Saga guidance.
- **Two-phase commit is structurally impossible** for cross-µservice
  writes because the saga shape is the only sanctioned shape.

### C-2. Negative

- **Cost of re-shaping existing flows.** ~24 cross-µservice flows
  catalogued in `registry/saga-shape/migration-backlog.tsv` need
  re-shaping. Mitigation: catalog is ordered by risk, and the gate
  is advisory until the migration is complete.
- **Workflow engine becomes a hard runtime dependency for cross-axis
  writes.** Mitigation: engine has its own SLO (`microservices/workflow-engine/slos/`)
  and per-cell-deployment per ADR-0009.
- **Compensation actions can themselves fail.** Mitigation: page on-call;
  the workflow engine's saga-state-machine retries the compensation
  per its declared policy.

### C-3. Sustainability

- The saga shape is testable in isolation per step (kernel-tier unit
  test) plus per saga (integration test). The portfolio test pattern
  (per ADR-0083 Tier 1) makes this cheap.
- Adding a new µservice to a saga is a saga-definition change, not a
  cross-µservice contract change.

## Implementation surface

- `microservices/workflow-engine/policy/saga-compensation-policy.md`
  — normative policy doc; companion to this ADR.
- `specs/saga-shape.json` — machine-readable schema; the workflow
  engine validates registered sagas against the schema.
- `crates/oya-check-saga-shape/` — kernel-tier validator that walks
  registered saga definitions and reports violations.
- Lane `saga-shape` added to `AGGREGATED_VALIDATE_LANES` (advisory).

## References

- AWS — *Distributed Sagas: A Protocol for Coordinating Microservices*
  (re:Invent 2017 + AWS Builders Library).
- Microsoft — *Saga design pattern* (Azure Architecture Center).
- Temporal — *Workflow + activity + compensation primitives* (public
  docs, 2024).
- Uber Cadence — *Cadence workflow patterns* (open source repo).
- Stripe Engineering — *Online migrations at scale* (2017 blog) — the
  saga pattern at billing.
- ADR-0035 (this portfolio) — workflow engine hybrid state-machine
  + DAG.
- ADR-0145 (this portfolio) — inter-microservice communication reform.
