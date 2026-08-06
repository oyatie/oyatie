---
id: ADR-0035
status: Superseded
superseded_by: [ADR-700]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Workflow engine hybrid SM+DAG — product spine

# ADR-0035: Workflow engine — hybrid state-machine + DAG (not pure BPMN), per-tenant versioning, jurisdiction overlay, agent-authored steps

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `foundry`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0003, ADR-0007, ADR-0011, ADR-0028, ADR-0029, ADR-0033, ADR-0034, ADR-0049

---

## Context

Every axis needs workflow: SaaS tenant onboarding, Workspace meeting scheduling, Vertical-pack claim adjudication, Foundry agent task execution, Cloud DCIM workorder dispatch. The pack-of-19 foundation ADRs named workflow as a cross-cutting need but did not pin the engine architecture. Two industry mainstreams compete: (a) **pure BPMN** (Camunda / Activiti / jBPM) which is rich but heavyweight and notoriously hard to version per tenant; (b) **pure DAG** (Airflow / Dagster / Argo Workflows) which is lightweight but lacks first-class state-machine semantics for human-in-the-loop and saga compensation.

Neither matches Oyatie's requirements: per-tenant workflow definition versioning (one tenant on V12, another on V14), per-jurisdiction overlay (KR claim adjudication has different mandatory steps than US), saga compensation across cross-microservice calls, and agent-authored steps where a Foundry agent can synthesize a step at runtime within autonomy ceiling. This ADR pins a **hybrid state-machine + DAG** engine that gives us state-machine semantics where they belong (human approvals, sealed-step gates, saga compensation) and DAG semantics where they belong (parallel computation, fan-out / fan-in).

---

## Decision

We build `crates/oya-workflow-*` as the canonical workflow engine for the entire ecosystem. The engine is a **hybrid state-machine + DAG**: at the top level, every workflow is a state machine; within each state, computation can be expressed as a DAG. Per-tenant workflow definition versioning is first-class; per-jurisdiction overlays bind at runtime via the regional-pack architecture.

### Engine architecture

```rust
// crates/oya-workflow-kernel
pub struct WorkflowDefinition {
    pub workflow_id: WorkflowId,
    pub version: WorkflowVersion,                  // per-tenant pinnable
    pub vertical_id: Option<VerticalId>,           // ADR-0033 binding
    pub jurisdiction_overlay: Option<RegionId>,    // ADR-0049 binding
    pub state_machine: StateMachine,
    pub dag_per_state: BTreeMap<StateId, Dag>,
    pub sealed_steps: BTreeSet<StateId>,           // immutable / human-gated
    pub saga_compensations: BTreeMap<StateId, CompensationStep>,
    pub capability_bindings: Vec<CapabilityRef>,   // ADR-0011 binding
}

pub enum StateTransition {
    Automatic { guard: TransitionGuard },
    HumanApproval { approver: ApproverRef, sla: Duration },
    AgentAuthored { autonomy_ceiling: PersonaTier }, // ADR-0007
    External { event_match: EventMatcher },
    Timer { duration: Duration },
}
```

### Top-level state machine

Every workflow is a finite-state machine. States represent business-meaningful phases (e.g. healthcare claim: `Submitted` → `EligibilityChecked` → `MedicalNecessityReview` → `Adjudicated` → `Paid`). Transitions can be automatic, human-approval-gated, agent-authored, external-event-driven, or timer-driven.

### Per-state DAG

Within a state, computation is a DAG. A claim eligibility check might fan out to (a) member lookup, (b) coverage check, (c) provider network check, all in parallel, with results joined into the eligibility decision.

### Sealed steps

A step marked `sealed` is immutable post-execution: its inputs, outputs, and audit record cannot be retroactively edited. Sealed steps are mandatory for any step that emits a regulator-relevant event (claim submitted, drug administered, payment disbursed, agent action taken at proxy autonomy).

### Branching

State machines support conditional branching via Cedar policy guards on transitions. Branches can be tenant-scoped (different tenants take different paths through the same workflow definition).

### Saga + compensating actions

Cross-axis workflow steps emit saga semantics: each step that mutates external state declares a compensating action. Workflow failure triggers per-step compensation in reverse order. The audit chain (ADR-0003) records both the forward and the compensating action.

### Agent-authored steps

A workflow author can declare a step as `AgentAuthored { autonomy_ceiling: PersonaTier }`. At runtime, a Foundry agent at the specified persona-tier (or below) is invoked via the capability registry (ADR-0011) to synthesize the step's action. The autonomy ceiling is hard-enforced by the policy kernel (ADR-0007); the engine cannot exceed it even if the agent attempts to.

### Per-step audit emission

Every step emits to the audit chain (ADR-0003) with: workflow_id, version, tenant_id, state_from, state_to, transition_kind, input_hash, output_hash, executor_identity (human / agent / system), duration, outcome.

### Replay via state-vector restore

The engine persists a state-vector at every state transition. Replay (e.g. for incident investigation, regulator inquiry, or test) restores the state vector and re-runs from a chosen state. Sealed steps are not re-executed during replay; they are surfaced as evidence.

### Visual editor in Workflow Studio

Workflow Studio is the per-tenant visual editor (Workspace-app, per ADR-0029 — ships under `crates/oya-workspace-tasks-*` as a sub-surface, since workflow definitions are tasks-graph kin). Tenant authors design workflows in a visual canvas; the engine compiles to the kernel `WorkflowDefinition`.

### Export/import per regional pack templates

Workflow definitions can be exported as portable JSON/YAML and imported into another tenant. Regional-pack-bundled templates ship per-vertical (e.g. KR healthcare claim adjudication template, US fintech KYC template).

### Per-tenant versioning

A tenant can pin a specific workflow version; in-flight instances complete on their pinned version; new instances start on the active version. Version migration is explicit (admin choose-and-confirm), never automatic.

### Per-jurisdiction overlay

A workflow definition can declare a `jurisdiction_overlay` slot. At runtime, the regional pack injects per-region mandatory steps (e.g. KR 「의료법」 §21 mandatory consent step before any PHI-touching workflow). The overlay cannot be removed by tenant admin.

### Capability bindings

Every workflow step that calls an external system does so via a capability ref (ADR-0011). The engine never makes raw HTTP calls; it always goes through the capability registry. Per-capability autonomy ceiling, per-capability rate limit, per-capability audit emission all bind automatically.

### Anti-scope

The workflow engine does not own the audit chain (ADR-0003), does not own the agent runtime (ADR-0007), does not own the capability registry (ADR-0011), does not own the Cedar policy evaluator (ADR-0007), does not ship its own UI shell (Workspace owns the shell per ADR-0029).

---

## Consequences

### Positive

- Hybrid state-machine + DAG matches the actual computational shape of cross-microservice workflows; neither pure BPMN nor pure DAG forces the model.
- Per-tenant versioning + per-jurisdiction overlay let one definition serve many tenants in many regions without per-tenant forks.
- Agent-authored steps give Foundry agents a structured way into business workflows, governed by autonomy ceiling.
- Saga compensation across cross-microservice calls is first-class — the cohesion thesis depends on cross-microservice transactions being safe.

### Negative

- Hybrid model is more complex than either pure BPMN or pure DAG; the kernel team owns the complexity tax.
- Visual editor scope is real; we ship a basic editor at GA and improve.
- Replay mechanics require state-vector persistence per transition, which has storage cost (per ADR-0045 OLAP tier absorbs).
- Per-jurisdiction overlay authoring is a recurring cost.

### Operational

- Per-workflow SLO catalog: state-transition P95 latency, saga-compensation success rate, agent-authored-step audit completeness.
- Per-version migration runbook; version sunset cascade per ADR-0038.
- Per-jurisdiction overlay regulator review annual.
- Workflow lane: `oya-governance-workflow-cohesion` — every workflow step's external call must be a capability ref.

---

## Alternatives considered

### Alternative A — Pure BPMN (Camunda / Activiti / jBPM)

- **Pros:** rich modeling; large industry user base.
- **Cons:** heavyweight runtime; per-tenant versioning bolted on; saga compensation not first-class; agent-authored steps awkward; usually JVM-bound (we are Rust-first per stack policy).
- **Rejected because:** the model is right; the implementation lineage is wrong for our stack.

### Alternative B — Pure DAG (Airflow / Dagster / Argo Workflows)

- **Pros:** lightweight; cloud-native idioms.
- **Cons:** no state-machine semantics; human-in-the-loop steps awkward; saga compensation requires custom plumbing; per-tenant versioning is a per-org configuration headache.
- **Rejected because:** misses the human/regulator/saga half of our workflow surface.

### Alternative C — Temporal as durable-execution engine (gated per ADR-0035)

- **Pros:** mature durable execution; saga support.
- **Cons:** adds a heavy runtime; Temporal license posture is a moving target; we would still need a per-tenant versioning + overlay layer above it.
- **Rejected because:** if we need a layer above it anyway, we should own the engine.

### Alternative D — Per-vertical workflow engine

- **Pros:** vertical teams own their engine.
- **Cons:** N engines; per-engine drift; cohesion thesis violated.
- **Rejected because:** the cohesion moat applies to workflow as much as to identity.

---

## Open questions

1. **Q1.** Workflow Studio visual editor — Yrs CRDT collaborative or single-author? Default: single-author at GA; collaborative at W+12. → ADR-0029.
2. **Q2.** Per-step retry policy default — exponential backoff with jitter, max 5 attempts? Default: yes; per-step override allowed. → owner: `foundry`.
3. **Q3.** Workflow definition format — proprietary JSON or attempt CNCF Serverless Workflow / DMN compatibility? Default: proprietary at GA; Serverless Workflow import adapter at W+12. → ADR-0037.
4. **Q4.** Replay scope — full re-execution (excluding sealed steps) or evidence-only? Default: evidence-only at GA; full re-execution requires explicit admin approval. → owner: `foundry`.
5. **Q5.** Agent-authored step max latency budget? Default: 30s P95; SLA degradation alarms at 10s. → ADR-0007.

---

## References

- `docs/PRD.md` §7 (workflow engine), §11 (per-jurisdiction overlay)
- `docs/DESIGN.md` §4 (workflow), §10 (cross-microservice contracts)
- BPMN 2.0 spec; CNCF Serverless Workflow; DMN 1.4
- KR 「의료법」 §21 (consent before PHI processing); 「전자금융거래법」 §6 (KYC workflow mandatory steps)
- ADR-0001 (cohesion), ADR-0003 (audit), ADR-0007 (Cedar + persona tier), ADR-0011 (capability registry), ADR-0028 (cloud), ADR-0029 (workspace tasks), ADR-0033 (vertical pack), ADR-0034 (per-vertical override), ADR-0037 (API stability), ADR-0038 (DSR cascade), ADR-0042 (observability), ADR-0045 (database tier), ADR-0049 (residency)
