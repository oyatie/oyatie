---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-substrate
phase: P12-workflow-engine
impl_plan_id: IP-001-workflow-engine-kernel-scaffold
status: pending
owner: council-architecture
blocked_by: []
acceptance_lanes:
  - cargo-check
  - cargo-build
  - cargo-clippy
  - cargo-nextest
  - cargo-deny
  - lean-a1
  - lean-a2
  - lean-a3
  - lean-a4
---

# IP-001-workflow-engine-kernel-scaffold: Scaffold Workflow Engine Kernel, Domain, Application, Adapter, Worker, gRPC, REST, App — Full DDL + Port Traits + Migrations

## Intent

Scaffolds all 25 workflow crates across 7 BCs (engine, transitions, approvals, sla,
automations, triggers, integrations), authors the complete Postgres DDL migrations
(expanding M02-substrate-schema-foundation §2), implements all kernel port traits with
sealed-trait markers, and wires the composition-root app binary. Establishes the
state-machine + DAG hybrid engine per Bominal ADR-0148 translated to oyatie BNF v4.1.
After this IP merges, every subsequent workflow IP (approvals, SLA, worker, etc.) has
a clean kernel to build against.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `Cargo.toml` | update | Add all 25 workflow crate workspace members |
| `crates/oya-workflow-engine-kernel/Cargo.toml` | create | Package manifest; zero framework deps |
| `crates/oya-workflow-engine-kernel/src/lib.rs` | create | Module declarations + pub use surface |
| `crates/oya-workflow-engine-kernel/src/types.rs` | create | WorkflowDefinition, RunId, RunState, WorkflowRunState (9-state), StepKind, TriggerSource, WorkflowEvent, EngineAction, StepEvent, StepOutput, Topic |
| `crates/oya-workflow-engine-kernel/src/ports.rs` | create | WorkflowStateStore, TransitionEngine, EventBus, AutomationRunner, WorkflowBridgePort — all sealed |
| `crates/oya-workflow-engine-kernel/src/errors.rs` | create | WorkflowError enum (thiserror) |
| `crates/oya-workflow-engine-domain/Cargo.toml` | create | Depends on oya-workflow-engine-kernel only |
| `crates/oya-workflow-engine-domain/src/lib.rs` | create | module declarations |
| `crates/oya-workflow-engine-domain/src/engine.rs` | create | WorkflowEngineLogic: pure business rules (evaluate_transition, compute_next_step) using WorkflowStateStore + TransitionEngine ports |
| `crates/oya-workflow-engine-application/Cargo.toml` | create | Depends on domain + kernel |
| `crates/oya-workflow-engine-application/src/lib.rs` | create | module declarations |
| `crates/oya-workflow-engine-application/src/use_cases.rs` | create | CreateRunUseCase, RecordStepUseCase, ReplayRunUseCase, TriggerWorkflowUseCase |
| `crates/oya-workflow-engine-adapter/Cargo.toml` | create | Depends on application + domain + kernel + sqlx + tokio |
| `crates/oya-workflow-engine-adapter/src/lib.rs` | create | module declarations |
| `crates/oya-workflow-engine-adapter/src/pg_state_store.rs` | create | PgWorkflowStateStore: impl WorkflowStateStore + sealed::Sealed; tenant_id RLS; outbox insert |
| `crates/oya-workflow-engine-adapter/src/pg_transition_engine.rs` | create | PgTransitionEngine: impl TransitionEngine; evaluates state-machine guard functions |
| `crates/oya-workflow-engine-adapter/src/kafka_event_bus.rs` | create | KafkaEventBus: impl EventBus; Redpanda KRaft publisher/subscriber |
| `crates/oya-workflow-engine-adapter/src/ontology_bridge.rs` | create | OntologyBridgeAdapter: impl WorkflowBridgePort; calls oya-ontology-entity-kernel ActionStore |
| `crates/oya-workflow-transitions-kernel/Cargo.toml` | create | Package manifest |
| `crates/oya-workflow-transitions-kernel/src/lib.rs` | create | TransitionStore port + Transition types |
| `crates/oya-workflow-transitions-adapter/Cargo.toml` | create | Depends on transitions-kernel + sqlx |
| `crates/oya-workflow-transitions-adapter/src/lib.rs` | create | PgTransitionAdapter: impl TransitionStore |
| `crates/oya-workflow-approvals-kernel/Cargo.toml` | create | Package manifest |
| `crates/oya-workflow-approvals-kernel/src/lib.rs` | create | ApprovalDecisionStore + ApproverResolver ports; ApprovalChain + ApprovalDecision + ApprovalSignature types |
| `crates/oya-workflow-approvals-domain/Cargo.toml` | create | Depends on approvals-kernel |
| `crates/oya-workflow-approvals-domain/src/lib.rs` | create | Multi-stage approval chain resolution logic |
| `crates/oya-workflow-approvals-application/Cargo.toml` | create | Depends on approvals-domain + approvals-kernel |
| `crates/oya-workflow-approvals-application/src/lib.rs` | create | SubmitApprovalUseCase, ResolveApproverUseCase |
| `crates/oya-workflow-approvals-adapter/Cargo.toml` | create | Depends on approvals-application + kernel + sqlx |
| `crates/oya-workflow-approvals-adapter/src/lib.rs` | create | PgApprovalDecisionAdapter: impl ApprovalDecisionStore with Ed25519 signature storage |
| `crates/oya-workflow-sla-kernel/Cargo.toml` | create | Package manifest |
| `crates/oya-workflow-sla-kernel/src/lib.rs` | create | SlaTimerStore + SlaBreachPublisher ports; SlaTimer + SlaConfig types |
| `crates/oya-workflow-sla-application/Cargo.toml` | create | Depends on sla-kernel |
| `crates/oya-workflow-sla-application/src/lib.rs` | create | SlaEnforcementUseCase: check due_at; escalate on breach |
| `crates/oya-workflow-sla-adapter/Cargo.toml` | create | Depends on sla-application + kernel + sqlx |
| `crates/oya-workflow-sla-adapter/src/lib.rs` | create | PgSlaTimerAdapter |
| `crates/oya-workflow-automations-kernel/Cargo.toml` | create | Package manifest |
| `crates/oya-workflow-automations-kernel/src/lib.rs` | create | AutomationBindingStore port; AutomationBinding + AutomationTrigger types |
| `crates/oya-workflow-automations-application/Cargo.toml` | create | Depends on automations-kernel |
| `crates/oya-workflow-automations-application/src/lib.rs` | create | RegisterAutomationUseCase, FireAutomationUseCase |
| `crates/oya-workflow-automations-adapter/Cargo.toml` | create | Depends on automations-application + kernel + sqlx |
| `crates/oya-workflow-automations-adapter/src/lib.rs` | create | PgAutomationBindingAdapter |
| `crates/oya-workflow-triggers-kernel/Cargo.toml` | create | Package manifest |
| `crates/oya-workflow-triggers-kernel/src/lib.rs` | create | TriggerStore + TriggerFirer ports; Trigger + TriggerType + TriggerConfig types |
| `crates/oya-workflow-triggers-application/Cargo.toml` | create | Depends on triggers-kernel |
| `crates/oya-workflow-triggers-application/src/lib.rs` | create | RegisterTriggerUseCase, FireTriggerUseCase |
| `crates/oya-workflow-triggers-adapter/Cargo.toml` | create | Depends on triggers-application + kernel + sqlx + tokio-cron-scheduler |
| `crates/oya-workflow-triggers-adapter/src/lib.rs` | create | PgTriggerAdapter + CronScheduler |
| `crates/oya-workflow-integrations-kernel/Cargo.toml` | create | Package manifest |
| `crates/oya-workflow-integrations-kernel/src/lib.rs` | create | IntegrationRunStore + ConnectorGateway ports; IntegrationRun + ConnectorConfig types |
| `crates/oya-workflow-integrations-application/Cargo.toml` | create | Depends on integrations-kernel |
| `crates/oya-workflow-integrations-application/src/lib.rs` | create | ExecuteIntegrationStepUseCase |
| `crates/oya-workflow-integrations-adapter/Cargo.toml` | create | Depends on integrations-application + kernel + sqlx + reqwest |
| `crates/oya-workflow-integrations-adapter/src/lib.rs` | create | PgIntegrationRunAdapter + HttpConnectorGateway |
| `crates/oya-workflow-engine-worker/Cargo.toml` | create | Depends on application + kernel (NOT adapter directly) |
| `crates/oya-workflow-engine-worker/src/lib.rs` | create | WorkflowWorker: consumes event bus; dispatches step kinds; stateless |
| `crates/oya-workflow-engine-grpc/Cargo.toml` | create | Depends on application + kernel; tonic |
| `crates/oya-workflow-engine-grpc/src/lib.rs` | create | gRPC service handlers wired to use-cases |
| `crates/oya-workflow-engine-rest/Cargo.toml` | create | Depends on application + kernel; axum |
| `crates/oya-workflow-engine-rest/src/lib.rs` | create | REST handlers: POST /runs, GET /runs/{id}, POST /runs/{id}/steps |
| `crates/oya-workflow-engine-app/Cargo.toml` | create | Composition root; depends on all layers |
| `crates/oya-workflow-engine-app/src/main.rs` | create | DI assembly; wires PgPool → adapters → services → gRPC + REST servers |
| `contracts/workflow.openapi.yaml` | create | Full OpenAPI 3.1 spec: createRun, getRun, recordStep, listRuns, triggerWorkflow |
| `contracts/workflow.proto` | create | Protobuf: WorkflowService rpc CreateRun / RecordStep / ReplayRun / TriggerWorkflow |
| `migrations/workflow/V001__workflow_schema.sql` | create | Full DDL from M02-substrate-schema-foundation §2 EXPANDED: definitions, runs, step_history, step_runs, triggers, approvals, approval_decisions, sla_timers, automation_bindings, integration_runs, outbox; all with RLS + indexes |
| `docs/standards/bounded-contexts.md` | update | Register 7 workflow BCs |

---

## Crate Naming

```
NAME: oya-workflow-engine-kernel
JUSTIFICATION:
- microservice = workflow: registered; flat BNF v4.1; override: shared not corporate
- bc-tokens = engine: multiple BCs in workflow µservice at same layer; engine BC is
  the core run/step/replay execution surface
- layer = kernel: pure types + sealed port traits; ZERO I/O; ADR-0056
- exemptions claimed: none

NAME: oya-workflow-engine-adapter
JUSTIFICATION:
- microservice = workflow, bc-tokens = engine: same rationale
- layer = adapter: Postgres + Kafka implementations of kernel ports; DTO mappers
- exemptions claimed: none

NAME: oya-workflow-engine-worker
JUSTIFICATION:
- microservice = workflow, bc-tokens = engine: same rationale
- layer = worker: long-running stateless step-dispatch loop; consumes event bus;
  no module-level mutable state (oya-check-statelessness passes)
- exemptions claimed: none

NAME: oya-workflow-approvals-kernel
JUSTIFICATION:
- microservice = workflow, bc-tokens = approvals: 전자결재 chain BC; distinct state
  machine from engine BC; Ed25519 approval signatures per ADR-0028 lineage
- layer = kernel: sealed port traits ApprovalDecisionStore + ApproverResolver
- exemptions claimed: none
```

---

## Code Shape

### `crates/oya-workflow-engine-kernel/src/types.rs`

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type TenantId = Uuid;
pub type RunId = Uuid;
pub type WorkflowDefId = Uuid;
pub type StepEventId = Uuid;

/// 9-state model per Bominal ADR-0148; maps to workflow.runs.state CHECK constraint
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowRunState {
    Draft,
    Simulated,
    Active,
    Blocked,
    Escalated,
    Reversed,
    Failed,
    Closed,
    Archived,
}

/// The sealed set of step kinds per Bominal ADR-0148 §"Inner layer"
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepKind {
    Form,
    Approval,
    Decision,
    Action,
    Integration,
    Wait,
    Branch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: WorkflowDefId,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: String,
    pub definition: serde_json::Value, // nodes + edges + triggers + state-machine spec
    pub version: i32,
    pub status: DefinitionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DefinitionStatus { Draft, Published, Archived }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunState {
    pub run_id: RunId,
    pub tenant_id: TenantId,
    pub def_id: WorkflowDefId,
    pub state: WorkflowRunState,
    pub current_step: Option<String>,
    pub step_state: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepEvent {
    pub step_name: String,
    pub kind: StepKind,
    pub event_type: StepEventType,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepEventType { Entered, Completed, Failed, Retry, SlaBreach }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerSource {
    pub kind: TriggerKind,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerKind { Cron, Webhook, Event, Ontology, Manual, Api }

#[derive(Debug, Clone)]
pub struct WorkflowEvent {
    pub tenant_id: TenantId,
    pub run_id: RunId,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct EngineAction {
    pub kind: EngineActionKind,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum EngineActionKind { RecordStep, TransitionRun, ScheduleSla, FireAutomation }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepOutput {
    pub success: bool,
    pub payload: serde_json::Value,
}

pub type Topic = String;
pub type BoxedHandler = Box<dyn Fn(bytes::Bytes) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), WorkflowError>> + Send>> + Send + Sync>;
pub type SubscriptionId = Uuid;

// Ontology bridge types (re-export from ontology kernel in app layer)
pub type ObjectId = Uuid;
pub type TypedAction = serde_json::Value;
pub type TypedObject = serde_json::Value;
pub type ActionResult = serde_json::Value;

use crate::errors::WorkflowError;
```

### `crates/oya-workflow-engine-kernel/src/ports.rs`

```rust
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

use async_trait::async_trait;
use crate::types::*;
use crate::errors::WorkflowError;

#[async_trait]
pub trait WorkflowStateStore: Send + Sync + sealed::Sealed {
    async fn create_run(&self, tenant_id: TenantId, def_id: WorkflowDefId, trigger: TriggerSource) -> Result<RunId, WorkflowError>;
    async fn record_step(&self, tenant_id: TenantId, run_id: RunId, event: StepEvent) -> Result<(), WorkflowError>;
    async fn replay_run(&self, tenant_id: TenantId, run_id: RunId) -> Result<RunState, WorkflowError>;
    async fn transition_run(&self, tenant_id: TenantId, run_id: RunId, to_state: WorkflowRunState) -> Result<(), WorkflowError>;
    async fn get_run(&self, tenant_id: TenantId, run_id: RunId) -> Result<Option<RunState>, WorkflowError>;
    async fn list_active_runs(&self, tenant_id: TenantId) -> Result<Vec<RunState>, WorkflowError>;
}

#[async_trait]
pub trait TransitionEngine: Send + Sync + sealed::Sealed {
    async fn evaluate(&self, run: &RunState, def: &WorkflowDefinition, event: WorkflowEvent) -> Result<Vec<EngineAction>, WorkflowError>;
}

#[async_trait]
pub trait EventBus: Send + Sync + sealed::Sealed {
    async fn publish(&self, topic: Topic, key: String, payload: bytes::Bytes) -> Result<(), WorkflowError>;
    async fn subscribe(&self, topic: Topic, handler: BoxedHandler) -> Result<SubscriptionId, WorkflowError>;
    async fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<(), WorkflowError>;
}

#[async_trait]
pub trait AutomationRunner: Send + Sync + sealed::Sealed {
    async fn run(&self, run_id: RunId, step: AutomationStep) -> Result<StepOutput, WorkflowError>;
}

#[async_trait]
pub trait WorkflowBridgePort: Send + Sync + sealed::Sealed {
    async fn apply_ontology_action(&self, tenant_id: TenantId, action: TypedAction) -> Result<ActionResult, WorkflowError>;
    async fn read_ontology_object(&self, tenant_id: TenantId, object_id: ObjectId) -> Result<TypedObject, WorkflowError>;
}

#[derive(Debug, Clone)]
pub struct AutomationStep {
    pub step_name: String,
    pub kind: StepKind,
    pub config: serde_json::Value,
}
```

### `migrations/workflow/V001__workflow_schema.sql` (shape)

```sql
CREATE SCHEMA IF NOT EXISTS workflow;

-- definitions (DAG + state machine spec)
CREATE TABLE workflow.definitions (
    workflow_def_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    name text NOT NULL,
    description text NOT NULL,
    definition jsonb NOT NULL,
    version int NOT NULL,
    status text NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'published', 'archived')),
    created_by uuid NOT NULL,
    published_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE workflow.definitions FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON workflow.definitions
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_workflow_def_name_version
    ON workflow.definitions (tenant_id, name, version);

-- runs (live executions; 9-state model per ADR-0148)
CREATE TABLE workflow.runs (
    run_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    workflow_def_id uuid NOT NULL REFERENCES workflow.definitions(workflow_def_id),
    state text NOT NULL DEFAULT 'draft' CHECK (state IN (
        'draft','simulated','active','blocked','escalated',
        'reversed','failed','closed','archived'
    )),
    current_step text NULL,
    step_state jsonb NOT NULL DEFAULT '{}'::jsonb,
    triggered_by_kind text NOT NULL CHECK (triggered_by_kind IN (
        'cron','webhook','event','ontology','manual','api'
    )),
    triggered_by_id text NOT NULL,
    started_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz NULL,
    sla_due_at timestamptz NULL,
    duration_ms int NULL
);
ALTER TABLE workflow.runs FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON workflow.runs
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_runs_active ON workflow.runs (tenant_id, state, sla_due_at)
    WHERE state IN ('active','blocked','escalated');
-- Citus distribution column declaration (sharding prep per ADR-0117 stage 2)
COMMENT ON TABLE workflow.runs IS 'distribution_column:tenant_id';

-- step_history (append-only; deterministic replay)
CREATE TABLE workflow.step_history (
    step_event_id bigserial PRIMARY KEY,
    tenant_id uuid NOT NULL,
    run_id uuid NOT NULL REFERENCES workflow.runs(run_id),
    step_name text NOT NULL,
    event_type text NOT NULL CHECK (event_type IN (
        'entered','completed','failed','retry','sla_breach'
    )),
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    occurred_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE workflow.step_history FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON workflow.step_history
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_step_history_run
    ON workflow.step_history (tenant_id, run_id, step_event_id);

-- step_runs (per-step execution records per ADR-0148)
CREATE TABLE workflow.step_runs (
    step_run_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    run_id uuid NOT NULL REFERENCES workflow.runs(run_id),
    step_key text NOT NULL,
    kind text NOT NULL CHECK (kind IN (
        'form','approval','decision','action','integration','wait','branch'
    )),
    status text NOT NULL DEFAULT 'pending' CHECK (status IN (
        'pending','running','completed','failed','skipped'
    )),
    input jsonb NULL,
    output jsonb NULL,
    started_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz NULL,
    error jsonb NULL
);
ALTER TABLE workflow.step_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON workflow.step_runs
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_step_runs_run ON workflow.step_runs (tenant_id, run_id, started_at);

-- triggers
CREATE TABLE workflow.triggers (
    trigger_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    workflow_def_id uuid NOT NULL REFERENCES workflow.definitions(workflow_def_id),
    trigger_type text NOT NULL CHECK (trigger_type IN (
        'cron','webhook','event','ontology','manual','api'
    )),
    config jsonb NOT NULL,
    enabled bool NOT NULL DEFAULT true,
    last_fired_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE workflow.triggers FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON workflow.triggers
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- approval_decisions (Ed25519 signed per ADR-0028 lineage)
CREATE TABLE workflow.approval_decisions (
    decision_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    step_run_id uuid NOT NULL REFERENCES workflow.step_runs(step_run_id),
    approver_user_id uuid NOT NULL,
    decision text NOT NULL CHECK (decision IN ('approved','rejected','delegated')),
    note text NULL,
    signature bytea NOT NULL,   -- Ed25519 over: tenant_id || step_run_id || decision || approver_user_id
    signing_key_id uuid NOT NULL,
    decided_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE workflow.approval_decisions FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON workflow.approval_decisions
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- sla_timers
CREATE TABLE workflow.sla_timers (
    timer_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    run_id uuid NOT NULL REFERENCES workflow.runs(run_id),
    due_at timestamptz NOT NULL,
    breached_at timestamptz NULL,
    escalated_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE workflow.sla_timers FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON workflow.sla_timers
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_sla_timers_due ON workflow.sla_timers (due_at)
    WHERE breached_at IS NULL;

-- automation_bindings
CREATE TABLE workflow.automation_bindings (
    binding_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    workflow_def_id uuid NOT NULL REFERENCES workflow.definitions(workflow_def_id),
    trigger_expr text NOT NULL,
    action_config jsonb NOT NULL,
    enabled bool NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE workflow.automation_bindings FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON workflow.automation_bindings
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- integration_runs
CREATE TABLE workflow.integration_runs (
    integration_run_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    step_run_id uuid NOT NULL REFERENCES workflow.step_runs(step_run_id),
    connector_id text NOT NULL,
    request_payload jsonb NOT NULL,
    response_payload jsonb NULL,
    status text NOT NULL DEFAULT 'pending' CHECK (status IN (
        'pending','running','success','failed','retrying'
    )),
    started_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz NULL,
    error_detail text NULL
);
ALTER TABLE workflow.integration_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON workflow.integration_runs
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- outbox (per outbox pattern; published to Kafka topic workflow.<event_type>)
CREATE TABLE workflow.outbox (
    outbox_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    topic text NOT NULL,
    key text NOT NULL,
    payload jsonb NOT NULL,
    published_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_workflow_outbox_unpublished ON workflow.outbox (created_at)
    WHERE published_at IS NULL;
```

---

## Acceptance Gates

```bash
# 1. Compile
cargo check -p oya-workflow-engine-kernel --all-features                 # exit 0
cargo check -p oya-workflow-engine-domain --all-features                 # exit 0
cargo check -p oya-workflow-engine-application --all-features            # exit 0
cargo check -p oya-workflow-engine-adapter --all-features                # exit 0
cargo check -p oya-workflow-approvals-kernel --all-features              # exit 0
cargo check -p oya-workflow-sla-kernel --all-features                    # exit 0
cargo check -p oya-workflow-automations-kernel --all-features            # exit 0
cargo check -p oya-workflow-triggers-kernel --all-features               # exit 0
cargo check -p oya-workflow-integrations-kernel --all-features           # exit 0
cargo check --workspace --all-features                                   # exit 0

# 2. Build
cargo build --workspace --all-features                                   # exit 0

# 3. Lint
cargo clippy --workspace --all-features -- -D warnings                  # exit 0; 0 warnings

# 4. Tests
cargo nextest run --workspace --all-features                             # exit 0; 0 failures

# 5. Supply chain
cargo deny check                                                         # exit 0

# 6. Docs
cargo doc --workspace --no-deps                                          # exit 0; 0 warnings

# 7. LEAN checks
oya gate validate lean-a1 --phase P12-workflow-engine                   # dependency-direction
oya gate validate lean-a2 --phase P12-workflow-engine                   # cross-product-refusal
oya gate validate lean-a3 --phase P12-workflow-engine                   # BC boundary
oya gate validate lean-a4 --phase P12-workflow-engine                   # naming conformance
oya gate validate port-location --phase P12-workflow-engine             # ports in kernel
oya gate validate statelessness --phase P12-workflow-engine             # worker stateless
oya gate validate shardability --phase P12-workflow-engine              # tenant_id partition key
```

---

## Test Plan

### Unit tests

Location: each `crates/oya-workflow-*-kernel/src/#[cfg(test)]`

| Test name | What it verifies |
|---|---|
| `test_workflow_run_state_transitions` | All 9 states; valid/invalid transitions per state machine guards |
| `test_step_kind_sealed` | StepKind enum covers exactly 7 kinds; no unknown variants at decode |
| `test_workflow_state_store_mock_create_run` | Mock WorkflowStateStore; create_run returns valid RunId |
| `test_workflow_state_store_mock_record_step` | Mock; record_step → replay_run includes the recorded step |
| `test_approval_decision_signature_preimage` | Signature preimage = tenant_id || step_run_id || decision || approver_user_id |
| `test_sla_timer_breach_detection` | SlaTimerStore mock; due_at in past → breach published |
| `test_automation_binding_fire` | AutomationRunner mock; trigger_expr match → StepOutput returned |
| `test_trigger_cron_parse` | CronScheduler parses cron expression; next_fire_at computed |
| `test_workflow_bridge_port_apply_action` | WorkflowBridgePort mock; apply_ontology_action returns ActionResult |

### Integration tests

Location: `crates/oya-workflow-engine-adapter/tests/`

| Test name | What it verifies |
|---|---|
| `integration_pg_workflow_state_store_create_replay` | PgWorkflowStateStore create_run → record_step → replay_run; RLS isolation between tenants |
| `integration_pg_approval_decision_with_signature` | PgApprovalDecisionAdapter; Ed25519 signature stored and verified |
| `integration_pg_sla_timer_breach` | PgSlaTimerAdapter; timer expires → SlaBreachPublisher called |
| `integration_outbox_publish` | After record_step, outbox row present; published_at NULL until worker fires |

### E2E / acceptance tests

| Scenario | Command | Expected output |
|---|---|---|
| Full workflow run end-to-end | `cargo nextest run --test e2e_workflow_run` | PASS; 0 failures |
| Cross-tenant isolation | `cargo nextest run --test isolation_workflow` | PASS; tenant B cannot read tenant A runs |

---

## Clean Architecture Compliance

### Dependency direction check

```
kernel  ←  domain  ←  application  ←  adapter  ←  {worker, grpc, rest}  ←  app
```

| Crate | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-workflow-engine-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `oya-workflow-engine-domain` | `domain` | `kernel` | `application`, `adapter`, `infrastructure`, presentation, `app` |
| `oya-workflow-engine-application` | `application` | `domain`, `kernel` | `adapter`, `infrastructure`, presentation, `app` |
| `oya-workflow-engine-adapter` | `adapter` | `application`, `domain`, `kernel` | `infrastructure`, presentation, `app` |
| `oya-workflow-engine-worker` | `worker` | `application`, `domain`, `kernel` | `adapter`, `infrastructure` directly |
| `oya-workflow-engine-app` | `app` | every layer | (none; unrestricted inward) |

### Cross-product integration check

This IP introduces NO direct imports between product µservices. Cross-product data flow:
- Workflow events (action/orchestration): `workflow.run.created`, `workflow.step.completed`, `workflow.run.state_changed`
- Ontology reads/writes (information): WorkflowBridgePort calls through `oya-ontology-entity-kernel` ActionStore port ONLY — never imports an ontology adapter directly

`oya gate validate lean-a2` passes by design: no product crate (medical, hr, payroll, etc.)
appears in any workflow kernel/domain/application/adapter `[dependencies]`.

---

## Load Test

### k6 smoke test (run in CI on every PR)

```javascript
// tests/load/smoke-workflow-engine.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  vus: 50,
  duration: '60s',
  thresholds: {
    http_req_duration: ['p(99)<200'],   // p99 ≤200ms for run creation (Action Type target)
    http_req_failed: ['rate<0.001'],
  },
};

export default function () {
  const payload = JSON.stringify({
    workflow_def_id: __ENV.WORKFLOW_DEF_ID,
    trigger: { kind: 'manual', id: 'load-test' },
  });
  const res = http.post(`${__ENV.BASE_URL}/api/v1/runs`, payload, {
    headers: { 'Content-Type': 'application/json', 'X-Tenant-Id': __ENV.TENANT_ID },
  });
  check(res, { 'status 201': (r) => r.status === 201 });
  sleep(0.1);
}
```

Run: `k6 run tests/load/smoke-workflow-engine.js --env BASE_URL=http://localhost:3020 --env TENANT_ID=<uuid> --env WORKFLOW_DEF_ID=<uuid>`

### Load test (staging before merge)

```bash
echo "POST http://staging.workflow/api/v1/runs" \
  | vegeta attack -rate=500/s -duration=60s -body tests/load/payload-create-run.json \
  | vegeta report
# Pass criteria: p99 ≤200ms; p999 ≤500ms; success rate ≥99.9%
```

### Throughput target verification

| Scenario | Tool | Target | Pass criterion |
|---|---|---|---|
| Create workflow run | k6 | p99 ≤200ms at 500 RPS | `http_req_duration{p(99)}<200` |
| Record step event | k6 | p99 ≤200ms at 1k RPS | `http_req_duration{p(99)}<200` |
| Replay run (read) | k6 | p99 ≤50ms at 2k RPS | `http_req_duration{p(99)}<50` |
| Sustained load | vegeta | 0 errors at 10k RPS (cell baseline) | success_rate=100% |

---

## Grit Symbol-Locks

```bash
grit session start m02-p12-workflow-engine-2026-05-13
# OR per-symbol:
grit claim \
  --agent council-architecture \
  --intent "IP-001-workflow-engine-kernel-scaffold: full workflow substrate scaffold" \
  --ttl 3600 \
  crates/oya-workflow-engine-kernel/src/lib.rs::WorkflowStateStore \
  crates/oya-workflow-engine-kernel/src/lib.rs::TransitionEngine \
  crates/oya-workflow-engine-kernel/src/lib.rs::EventBus \
  crates/oya-workflow-engine-kernel/src/lib.rs::AutomationRunner \
  crates/oya-workflow-engine-kernel/src/lib.rs::WorkflowBridgePort \
  contracts/workflow.proto::WorkflowService \
  contracts/workflow.openapi.yaml::createRun \
  migrations/workflow/V001__workflow_schema.sql::workflow.definitions
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-001-workflow-engine-kernel-scaffold merged; crates scaffolded: oya-workflow-engine-kernel/domain/application/adapter/worker/grpc/rest/app + 7 BC kernels; migrations: V001; grit symbols released; acceptance lanes green; next IP: IP-002-workflow-engine-adapter" \
  -i high \
  -k "M02,P12,IP-001,workflow"
```

---

## Halt Conditions

1. `cargo check` fails after 3 attempts with the same error.
2. LEAN-A2 cross-product-refusal violation: any workflow crate importing a product crate (medical, hr, payroll, etc.) — escalate; do NOT add the import.
3. Crate name cannot satisfy BNF v4.1 justification — escalate to architect agent.
4. Grit claim conflicts with another agent on any symbol.
5. Ed25519 signature preimage for approval_decisions deviates from ADR-0028 pattern — escalate; do NOT invent a new preimage.

---

## Next IP Pointer

`IP-002-workflow-engine-adapter.md` (same phase directory).

---

## Cross-References

- Phase spec: `phase-spec.md`
- Milestone README: `../../README.md`
- PRD: `docs/prds/workflow.md`
- ADR-0056 (BNF v4.1), ADR-0148 (state-machine + DAG), ADR-0103 (hexagonal migration), ADR-0028 (audit-chain)
- M02-substrate-schema-foundation.md §2 (expanded into V001 migration)
- Memory: `feedback_workflow_objectgraph_adapter_layer.md`, `feedback_workflow_is_shared.md`, `feedback_clean_architecture_requirements.md`, `feedback_quality_performance_scalability_bar.md`
