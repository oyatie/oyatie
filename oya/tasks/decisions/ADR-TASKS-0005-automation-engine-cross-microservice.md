---
id: ADR-TASKS-0005
status: Accepted
date: 2026-05-17
microservice: tasks
deciders: axis-tasks, council-architecture, axis-workflow-engine, axis-workflow-studio
owner: axis-tasks + axis-workflow-engine
supersedes: []
superseded_by: []
related:
  - ADR-0056
  - ADR-0105
  - ADR-0135
  - ADR-0131
  - ADR-0132
related_artifacts:
  - microservices/tasks/PRD.md (Tenant Outcome 5 bidirectional task ↔ workflow; §"Integration via Workflow + Ontology")
  - microservices/tasks/IP-009-state-workflow-engine-cross-link.md
  - microservices/tasks/contracts/asyncapi/tasks-events.yaml (tasks.task.state.v1)
  - microservices/tasks/contracts/proto/tasks.proto (StreamStateTransitions)
purpose: |
  Decide how the tasks µservice integrates with the workflow-engine —
  closes PRD-tasks "Tenant Outcome 5 — bidirectional task ↔ workflow"
  + the cross-product-isolation invariant from feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md.
---

# ADR-TASKS-0005: Automation engine cross-µservice — bidirectional workflow-engine bridge via canonical event bus + gRPC; no in-µservice durable-execution engine

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

PRD-tasks declares **Tenant Outcome 5**: bidirectional task ↔ workflow
integration. The user expectation:

- A task state change should be able to trigger a workflow (e.g.,
  "when a task transitions to Done, kick off a billing workflow").
- A workflow should be able to create / mutate tasks (e.g.,
  "every Monday at 9am, create a 'weekly retro' task in the engineering
  project").

The naïve approach is to embed a Temporal-class durable-execution
engine inside the tasks µservice itself, run the automations there.
This violates the cross-product rule per
`feedback_workflow_objectgraph_adapter_layer.md`: all inter-product
flows go through Workflow (orchestration) + Ontology (info); products
never call each other directly.

Three integration patterns compete:

1. **Embedded mini-workflow engine** inside tasks. Pros: zero
   inter-µservice latency. Cons: duplicates the durable-execution
   engine that the workflow-engine µservice already owns; cross-product
   refusal lane fails.
2. **Direct gRPC calls between tasks and workflow-engine** (both
   directions). Pros: clean strong-typed contract. Cons: tight coupling
   — a workflow-engine outage halts task state transitions; cross-
   product-direct-call refusal applies.
3. **Asynchronous via canonical event bus** in BOTH directions. Pros:
   loose coupling; outage tolerance; cross-product refusal honoured;
   audit-chain trivially traces every event. Cons: eventual consistency
   between the task state change and the workflow trigger (typically
   < 100ms via NATS).
4. **Asymmetric pattern**: tasks → workflow via async events; workflow
   → tasks via public gRPC (workflow-engine becomes a regular gRPC
   client of the tasks public API). Pros: cleanest separation —
   workflow-engine has no special privileges; combines the strengths
   of (3) and (2).

## Decision

The tasks µservice ships **the asymmetric pattern (option 4)**:

- **task state change → workflow trigger**: tasks publishes
  `TaskStateTransitioned` on `tasks.task.state.v1` AsyncAPI channel.
  The workflow-engine subscribes via the canonical NATS bus; tasks
  has NO direct dependency on workflow-engine. No durable-execution
  engine duplication.
- **workflow creates / mutates tasks**: the workflow-engine acts as
  a regular gRPC client of `oya.tasks.v1.TaskStore` and
  `oya.tasks.v1.ProjectList`. The workflow-engine's caller identity is
  a per-tenant service principal; Cedar policies authorise its actions
  exactly as any other client.
- **bidirectional traceability**: emitted state-transition events
  carry an optional `via_workflow_id` field when the transition was
  driven by a workflow-engine call (correlated via the gRPC
  `workflow_id` request header).

Cross-product isolation lane (`oya-governance-cross-product-isolation`)
enforces:

- No `oya-workflow-engine-*` crate imported by any `oya-tasks-*` crate.
- No `oya-tasks-*` crate imported by any `oya-workflow-engine-*` crate.

## Alternatives Considered

### Alternative 1 — Embedded mini-workflow engine inside tasks

- Pros:
  - Zero inter-µservice latency.
- Cons:
  - Duplicates durable-execution engine; cross-product refusal lane
    fails; ops-burden of two scheduler engines.
- Rejected because: bundle anti-pattern + cross-product violation.

### Alternative 2 — Direct gRPC both directions

- Pros:
  - Strong-typed contract; low latency.
- Cons:
  - Outage coupling: workflow-engine down → task state transitions
    halt (if synchronously waiting for trigger ack). Or: tasks-down →
    workflow-engine's outbound task-create call fails.
- Rejected because: outage-coupling unacceptable for AC-13 availability
  budget (99.95% task-read; 99.9% task-write).

### Alternative 3 — Both directions via async events

- Pros:
  - Maximum decoupling.
- Cons:
  - Asymmetric — workflow-engine's create-task call has natural REST/
    gRPC shape; forcing it into a bus message is awkward and adds
    response-correlation complexity (a sync response over an async
    bus = bad pattern).
- Rejected because: workflow → tasks is genuinely a client-server call;
  forcing it into events adds complexity without benefit.

## Consequences

### Consequence 1 — IP-009 + IP-013 + AsyncAPI ship together

IP-009 wires the state-transition emit path; IP-013 wires the public
gRPC peer surface; the AsyncAPI catalog enumerates the
`tasks.task.state.v1` channel. All three must land coherently to
make T-Outcome-5 work end-to-end.

### Consequence 2 — `via_workflow_id` is observability paid_scale

When a state transition was driven by workflow-engine, the emitted
event carries the workflow ID. This enables: (a) "show me all tasks
this workflow touched" queries; (b) audit-chain attribution; (c)
fairness analysis (workflow-driven auto-transitions vs human-driven).

### Consequence 3 — workflow-engine has no special privileges

The workflow-engine speaks the same public REST + gRPC contract as
any tenant integration. There is no internal-only API surface
between tasks and workflow-engine. This keeps the contract stable
and prevents quiet over-fitting of one to the other.

## References

- ADR-0132 (no-grouping policy + cross-product refusal).
- ADR-TASKS-0001 (data model); ADR-TASKS-0006 (auto-assign + EU AI Act).
- `feedback_workflow_objectgraph_adapter_layer.md` (adapter rule).
- Temporal — `temporal.io` (durable-execution comparison).
- NATS — `nats.io` (canonical event bus).
- PRD-tasks Tenant Outcome 5 + §"Integration via Workflow + Ontology".
