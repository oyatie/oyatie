---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02b-substrate
phase: P12-workflow-engine
status: Proposed
acceptance_lanes: []
entry_gate: 'M02/P02-ontology complete; oya-ontology-kernel ships with ObjectStore
  +

  LinkStore + ActionStore + OntologyFunction port traits; cargo check clean

  on workspace; grit done called on all P02 symbols; ICM phase-handoff row emitted.

  '
exit_gate: 'All P12 impl-plan acceptance gates green; 9 BCs registered in

  docs/standards/bounded-contexts.md; all crates pass cargo check/build/clippy/

  nextest/deny; oya gate validate lean-a1/lean-a2/lean-a3/lean-a4 exit 0;

  grit done called on all P12 symbols; ICM phase-complete row emitted.

  '
depends_on:
- milestone: M02
  phase: P02-ontology
  reason: Workflow engine adapter/glue layer calls through Ontology ports for all
    cross-product data mutations (action steps call ObjectStore/ActionStore); WorkflowBridgePort
    trait uses OntologyFunction for entity reads.
owner_team: council-architecture
purpose: "Delivers the oyatie Workflow engine substrate: the action/orchestration adapter layer that makes cross-product and intra-product integration coherent across the flat microservice catalog."
---
# P12-workflow-engine: Workflow Engine Substrate — State-Machine + DAG Adapter/Glue Layer

## Purpose

Delivers the oyatie Workflow engine substrate: the action/orchestration adapter layer
that makes cross-product and intra-product integration coherent across the flat
microservice catalog. Per [[feedback-workflow-objectgraph-adapter-layer]], Workflow is
THE load-bearing integration plane — products publish typed events; Workflow routes
them via state-machine + DAG hybrid; consuming products subscribe. No direct
cross-product imports are permitted; all inter-product action flow goes through this
layer.

The engine model follows Bominal ADR-0148 (state-machine + DAG hybrid, not BPMN) and
ADR-0103 (hexagonal migration) translated to oyatie BNF v4.1. The Workflow Studio
visual editor ships in M03 separately (per [[feedback-workflow-studio-scope]]); this
phase ships the engine substrate, kernel ports, adapter implementations, worker binary,
and gRPC/REST API surface that M03 will build the editor on top of.

Nine bounded contexts ship: workflow-engine (core run/step execution), workflow-transitions
(state audit trail), workflow-approvals (multi-stage 전자결재 chains), workflow-sla
(SLA timer enforcement), workflow-automations (trigger-bound automation bindings),
workflow-triggers (cron/webhook/event/ontology/manual/api trigger types),
workflow-integrations (connector action framework).

Advances Master Plan principles: flat-catalog integration coherence (all products
orchestratable without cross-product imports); hyperscaler-grade horizontal scale
(stateless worker pool + sharded state per tenant_id); Cedar policy gate on every
automation step execution.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `workflow` | `engine` | `crates/oya-workflow-engine-kernel/` | `oya-workflow-engine-kernel` |
| `workflow` | `engine` | `crates/oya-workflow-engine-domain/` | `oya-workflow-engine-domain` |
| `workflow` | `engine` | `crates/oya-workflow-engine-application/` | `oya-workflow-engine-application` |
| `workflow` | `engine` | `crates/oya-workflow-engine-adapter/` | `oya-workflow-engine-adapter` |
| `workflow` | `engine` | `crates/oya-workflow-engine-worker/` | `oya-workflow-engine-worker` |
| `workflow` | `engine` | `crates/oya-workflow-engine-grpc/` | `oya-workflow-engine-grpc` |
| `workflow` | `engine` | `crates/oya-workflow-engine-rest/` | `oya-workflow-engine-rest` |
| `workflow` | `engine` | `crates/oya-workflow-engine-app/` | `oya-workflow-engine-app` |
| `workflow` | `transitions` | `crates/oya-workflow-transitions-kernel/` | `oya-workflow-transitions-kernel` |
| `workflow` | `transitions` | `crates/oya-workflow-transitions-adapter/` | `oya-workflow-transitions-adapter` |
| `workflow` | `approvals` | `crates/oya-workflow-approvals-kernel/` | `oya-workflow-approvals-kernel` |
| `workflow` | `approvals` | `crates/oya-workflow-approvals-domain/` | `oya-workflow-approvals-domain` |
| `workflow` | `approvals` | `crates/oya-workflow-approvals-application/` | `oya-workflow-approvals-application` |
| `workflow` | `approvals` | `crates/oya-workflow-approvals-adapter/` | `oya-workflow-approvals-adapter` |
| `workflow` | `sla` | `crates/oya-workflow-sla-kernel/` | `oya-workflow-sla-kernel` |
| `workflow` | `sla` | `crates/oya-workflow-sla-application/` | `oya-workflow-sla-application` |
| `workflow` | `sla` | `crates/oya-workflow-sla-adapter/` | `oya-workflow-sla-adapter` |
| `workflow` | `automations` | `crates/oya-workflow-automations-kernel/` | `oya-workflow-automations-kernel` |
| `workflow` | `automations` | `crates/oya-workflow-automations-application/` | `oya-workflow-automations-application` |
| `workflow` | `automations` | `crates/oya-workflow-automations-adapter/` | `oya-workflow-automations-adapter` |
| `workflow` | `triggers` | `crates/oya-workflow-triggers-kernel/` | `oya-workflow-triggers-kernel` |
| `workflow` | `triggers` | `crates/oya-workflow-triggers-application/` | `oya-workflow-triggers-application` |
| `workflow` | `triggers` | `crates/oya-workflow-triggers-adapter/` | `oya-workflow-triggers-adapter` |
| `workflow` | `integrations` | `crates/oya-workflow-integrations-kernel/` | `oya-workflow-integrations-kernel` |
| `workflow` | `integrations` | `crates/oya-workflow-integrations-application/` | `oya-workflow-integrations-application` |
| `workflow` | `integrations` | `crates/oya-workflow-integrations-adapter/` | `oya-workflow-integrations-adapter` |
| `workflow` | all | `contracts/workflow.openapi.yaml` | — |
| `workflow` | all | `contracts/workflow.proto` | — |
| `workflow` | all | `migrations/workflow/` | — |

Naming justification for new crates:

```
NAME: oya-workflow-engine-kernel
JUSTIFICATION:
- microservice = workflow: the workflow engine µservice; registered in
  [workspace.metadata.oya.microservices]; ADR-0056 v4.1 flat BNF; override #1
  (feedback_workflow_is_shared): workflow is shared, not corporate-owned
- bc-tokens = engine: distinct from transitions/approvals/sla/automations/triggers/
  integrations BCs within the same µservice; required by BC-optionality rule when
  multiple BCs at same layer
- layer = kernel: pure types (WorkflowDefinition, RunId, StepKind, TriggerSource,
  WorkflowEvent) + port trait declarations (WorkflowStateStore, TransitionEngine,
  EventBus, AutomationRunner); ZERO business logic; ADR-0056 §"Layer semantics"
- exemptions claimed: none
```

```
NAME: oya-workflow-engine-worker
JUSTIFICATION:
- microservice = workflow, bc-tokens = engine: same as above
- layer = worker: long-running background worker that executes the DAG step
  dispatch loop; consumes from outbox/event-bus; stateless; horizontally scalable;
  ADR-0056 §"Layer semantics" — worker = long-running background workers
- exemptions claimed: none
```

```
NAME: oya-workflow-approvals-kernel
JUSTIFICATION:
- microservice = workflow: same µservice
- bc-tokens = approvals: 전자결재 approval chain BC; separate from engine BC;
  multi-stage approval decisions with Ed25519 signatures per ADR-0028 lineage
- layer = kernel: port traits (ApprovalDecisionStore, ApproverResolver) + types
  (ApprovalChain, ApprovalDecision, ApprovalSignature)
- exemptions claimed: none
```

### Out-of-scope

- Workflow Studio visual editor — deferred to M03/P-studio; editor needs M02 engine running
- Definition versioning + jurisdiction overlays — deferred to M03 (Bominal ADR-0149 pattern)
- WASM plugin sandbox for user-extensible step logic — deferred to M03 (Bominal ADR-0161)
- Pre-publish validator chain — deferred to M03 (Bominal ADR-0160)

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`IP-001-workflow-engine-kernel-scaffold.md`](IP-001-workflow-engine-kernel-scaffold.md) | Scaffold oya-workflow-engine-kernel with all port traits and types; full DDL migrations | pending | `council-architecture` |
| [`IP-002-workflow-engine-adapter.md`](IP-002-workflow-engine-adapter.md) | Postgres adapter implementing WorkflowStateStore + TransitionEngine; outbox publisher | pending | `council-architecture` |
| [`IP-003-workflow-approvals.md`](IP-003-workflow-approvals.md) | Approvals BC: ApprovalDecisionStore port + Postgres adapter + Ed25519 signature store | pending | `council-architecture` |
| [`IP-004-workflow-sla-automations-triggers.md`](IP-004-workflow-sla-automations-triggers.md) | SLA timer, automations binding, triggers (cron/webhook/event/ontology) BCs | pending | `council-architecture` |
| [`IP-005-workflow-integrations-connector.md`](IP-005-workflow-integrations-connector.md) | Integrations BC: WorkflowBridgePort connecting to Ontology ActionStore | pending | `council-architecture` |
| [`IP-006-workflow-worker-grpc-rest.md`](IP-006-workflow-worker-grpc-rest.md) | Worker binary + gRPC service + REST API + OpenAPI/Protobuf contracts | pending | `council-architecture` |
| [`IP-007-workflow-load-tests.md`](IP-007-workflow-load-tests.md) | k6 + vegeta load tests meeting Performance Targets; Cedar policy gate tests | pending | `council-architecture` |

---

## Acceptance Gates

### Cargo / CI gates (exit 0 required)

```bash
cargo check --workspace --all-features               # exit 0
cargo build --workspace --all-features               # exit 0
cargo clippy --workspace --all-features -- -D warnings  # exit 0; 0 warnings
cargo nextest run --workspace --all-features         # exit 0; 0 failures
cargo deny check                                     # exit 0
cargo doc --workspace --no-deps                      # exit 0; 0 warnings
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --phase P12-workflow-engine   # LEAN-A1: layer ordering
oya gate validate lean-a2 --phase P12-workflow-engine   # LEAN-A2: cross-product refusal
oya gate validate lean-a3 --phase P12-workflow-engine   # LEAN-A3: BC boundary
oya gate validate lean-a4 --phase P12-workflow-engine   # LEAN-A4: naming conformance
```

### Workflow + Ontology integration gates

```bash
oya gate validate workflow-event-registry --phase P12-workflow-engine
oya gate validate ontology-type-registry --phase P12-workflow-engine
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? | Presentation-only? |
|---|---|---|---|---|
| `oya-workflow-engine-kernel` | `kernel` | Yes — WorkflowStateStore, TransitionEngine, EventBus, AutomationRunner, WorkflowBridgePort | N/A | No |
| `oya-workflow-engine-domain` | `domain` | N/A — calls through ports | N/A | No |
| `oya-workflow-engine-application` | `application` | N/A | N/A | No |
| `oya-workflow-engine-adapter` | `adapter` | N/A | Yes — PgWorkflowStateStore, KafkaEventBus | No |
| `oya-workflow-engine-worker` | `worker` | N/A | No direct adapter import | Yes |
| `oya-workflow-engine-grpc` | `grpc` | N/A | No direct adapter import | Yes |
| `oya-workflow-engine-rest` | `rest` | N/A | No direct adapter import | Yes |
| `oya-workflow-engine-app` | `app` | N/A | Unrestricted inward (wiring only) | No |
| `oya-workflow-approvals-kernel` | `kernel` | Yes — ApprovalDecisionStore, ApproverResolver | N/A | No |
| `oya-workflow-approvals-domain` | `domain` | N/A | N/A | No |
| `oya-workflow-approvals-application` | `application` | N/A | N/A | No |
| `oya-workflow-approvals-adapter` | `adapter` | N/A | Yes — PgApprovalDecisionAdapter | No |
| `oya-workflow-sla-kernel` | `kernel` | Yes — SlaTimerStore, SlaBreachPublisher | N/A | No |
| `oya-workflow-sla-application` | `application` | N/A | N/A | No |
| `oya-workflow-sla-adapter` | `adapter` | N/A | Yes — PgSlaTimerAdapter | No |
| `oya-workflow-automations-kernel` | `kernel` | Yes — AutomationBindingStore | N/A | No |
| `oya-workflow-automations-application` | `application` | N/A | N/A | No |
| `oya-workflow-automations-adapter` | `adapter` | N/A | Yes — PgAutomationBindingAdapter | No |
| `oya-workflow-triggers-kernel` | `kernel` | Yes — TriggerStore, TriggerFirer | N/A | No |
| `oya-workflow-triggers-application` | `application` | N/A | N/A | No |
| `oya-workflow-triggers-adapter` | `adapter` | N/A | Yes — PgTriggerAdapter, CronScheduler | No |
| `oya-workflow-integrations-kernel` | `kernel` | Yes — IntegrationRunStore, ConnectorGateway | N/A | No |
| `oya-workflow-integrations-application` | `application` | N/A | N/A | No |
| `oya-workflow-integrations-adapter` | `adapter` | N/A | Yes — PgIntegrationRunAdapter | No |
| `oya-workflow-transitions-kernel` | `kernel` | Yes — TransitionStore | N/A | No |
| `oya-workflow-transitions-adapter` | `adapter` | N/A | Yes — PgTransitionAdapter | No |

### Port traits declared in kernel (core set)

```rust
// oya-workflow-engine-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

#[async_trait::async_trait]
pub trait WorkflowStateStore: Send + Sync + sealed::Sealed {
    async fn create_run(&self, tenant_id: TenantId, def_id: WorkflowDefId, trigger: TriggerSource) -> Result<RunId, WorkflowError>;
    async fn record_step(&self, tenant_id: TenantId, run_id: RunId, event: StepEvent) -> Result<(), WorkflowError>;
    async fn replay_run(&self, tenant_id: TenantId, run_id: RunId) -> Result<RunState, WorkflowError>;
    async fn transition_run(&self, tenant_id: TenantId, run_id: RunId, to_state: WorkflowRunState) -> Result<(), WorkflowError>;
}

#[async_trait::async_trait]
pub trait TransitionEngine: Send + Sync + sealed::Sealed {
    async fn evaluate(&self, run: &RunState, def: &WorkflowDefinition, event: WorkflowEvent) -> Result<Vec<EngineAction>, WorkflowError>;
}

#[async_trait::async_trait]
pub trait EventBus: Send + Sync + sealed::Sealed {
    async fn publish(&self, topic: Topic, key: String, payload: bytes::Bytes) -> Result<(), WorkflowError>;
    async fn subscribe(&self, topic: Topic, handler: BoxedHandler) -> Result<SubscriptionId, WorkflowError>;
}

#[async_trait::async_trait]
pub trait AutomationRunner: Send + Sync + sealed::Sealed {
    async fn run(&self, run_id: RunId, step: AutomationStep) -> Result<StepOutput, WorkflowError>;
}

// WorkflowBridgePort: the audited boundary between Workflow executor and per-product engines
// Per Bominal ADR-0148 §"Composition with other modules" translated to oyatie clean-arch
#[async_trait::async_trait]
pub trait WorkflowBridgePort: Send + Sync + sealed::Sealed {
    async fn apply_ontology_action(&self, tenant_id: TenantId, action: TypedAction) -> Result<ActionResult, WorkflowError>;
    async fn read_ontology_object(&self, tenant_id: TenantId, object_id: ObjectId) -> Result<TypedObject, WorkflowError>;
}
```

### CI lanes that must green before phase exit gate

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P12-workflow-engine` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P12-workflow-engine` | exit 0 |
| `port-location` | `oya gate validate port-location --phase P12-workflow-engine` | exit 0 |
| `layer-correctness` | `oya gate validate layer-correctness --phase P12-workflow-engine` | exit 0 |
| `statelessness` | `oya gate validate statelessness --phase P12-workflow-engine` | exit 0 |
| `shardability` | `oya gate validate shardability --phase P12-workflow-engine` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `workflow-engine` | `workflow` | pending |
| `workflow-transitions` | `workflow` | pending |
| `workflow-approvals` | `workflow` | pending |
| `workflow-sla` | `workflow` | pending |
| `workflow-automations` | `workflow` | pending |
| `workflow-triggers` | `workflow` | pending |
| `workflow-integrations` | `workflow` | pending |

---

## Grit Claim Symbols

```
crates/oya-workflow-engine-kernel/src/lib.rs::WorkflowStateStore
crates/oya-workflow-engine-kernel/src/lib.rs::TransitionEngine
crates/oya-workflow-engine-kernel/src/lib.rs::EventBus
crates/oya-workflow-engine-kernel/src/lib.rs::AutomationRunner
crates/oya-workflow-engine-kernel/src/lib.rs::WorkflowBridgePort
crates/oya-workflow-engine-domain/src/lib.rs::WorkflowDefinition
crates/oya-workflow-engine-application/src/lib.rs::WorkflowEngineService
crates/oya-workflow-engine-adapter/src/lib.rs::PgWorkflowStateStore
crates/oya-workflow-approvals-kernel/src/lib.rs::ApprovalDecisionStore
crates/oya-workflow-sla-kernel/src/lib.rs::SlaTimerStore
crates/oya-workflow-automations-kernel/src/lib.rs::AutomationBindingStore
crates/oya-workflow-triggers-kernel/src/lib.rs::TriggerStore
crates/oya-workflow-integrations-kernel/src/lib.rs::ConnectorGateway
contracts/workflow.openapi.yaml::createRun
contracts/workflow.proto::WorkflowService
migrations/workflow/V001__workflow_schema.sql::workflow.definitions
```

TTL recommendation: `--ttl 3600` per IP. Fallback: ICM topic `scaffold-locks-oyatie`.

---

## ICM Rationale Fields

```bash
# At phase start
icm store \
  -t context-oyatie \
  -c "Phase P12-workflow-engine started; milestone M02b-substrate; scope: workflow µservice 7 BCs (engine/transitions/approvals/sla/automations/triggers/integrations); entry gate met: P02-ontology complete" \
  -i high \
  -k "M02,P12,phase-start,workflow"

# At phase complete
icm store \
  -t context-oyatie \
  -c "Phase P12-workflow-engine complete; IPs IP-001..007 merged; grit symbols released; lanes lean-a1/a2/a3/a4 green; next phase: P13-tenancy" \
  -i high \
  -k "M02,P12,phase-complete,workflow"
```

---

## References

- Milestone README: `../../README.md`
- Bominal ADRs inherited: ADR-0103 (workflow hexagonal migration), ADR-0148 (state-machine + DAG hybrid), ADR-0028 (audit-chain Ed25519)
- oyatie ADRs cited: ADR-0056 v4.1 (BNF), ADR-0035 (workflow engine)
- Memory files: `feedback_workflow_objectgraph_adapter_layer.md`, `feedback_workflow_is_shared.md`, `feedback_workflow_studio_scope.md`, `feedback_clean_architecture_requirements.md`
